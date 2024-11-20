using System.Net;
using Amazon.S3.Model;
using AutoFixture;
using Dapper;
using FluentAssertions;
using MassTransit;
using MassTransit.Testing;
using Mediator;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.Options;
using Npgsql;
using NSubstitute;
using NSubstitute.ClearExtensions;
using UserManagement.Application.Contracts.Requests.Auth;
using UserManagement.Application.Factories.Abstractions;
using UserManagement.Application.Helpers;
using UserManagement.Application.Mappers;
using UserManagement.Application.Messaging.Users;
using UserManagement.Application.Models;
using UserManagement.Application.Services;
using UserManagement.Application.Services.Abstractions;
using UserManagement.Application.Settings;
using UserManagement.Tests.Integration.Fixtures;
using UserManagement.Tests.Integration.Fixtures.Customizations;

namespace UserManagement.Tests.Integration.Context.Services;

[Collection(nameof(UserManagementApiCollection))]
public sealed class AuthServiceTests : IAsyncDisposable
{
    private readonly UserManagementApiFactory _factory;
    private readonly IDbConnectionFactory _dbConnectionFactory;
    private readonly ITokenWriter _tokenWriter;
    private readonly IHashService _hashService;
    private readonly S3Settings _s3Settings;
    private readonly ITestHarness _testHarness;
    private readonly IOtpService _otpService;
    private readonly AuthService _authService;
    private readonly AsyncServiceScope _serviceScope;

    private readonly Fixture _fixture;

    public AuthServiceTests(UserManagementApiFactory factory)
    {
        _factory = factory;
        _fixture = new Fixture();
        _fixture.Customize(new RegisterRequestCustomization())
            .Customize(new DateTimeCustomization());

        _serviceScope = _factory.Services.CreateAsyncScope();
        _dbConnectionFactory = _serviceScope.ServiceProvider.GetRequiredService<IDbConnectionFactory>();
        _tokenWriter = _serviceScope.ServiceProvider.GetRequiredService<ITokenWriter>();
        _hashService = _serviceScope.ServiceProvider.GetRequiredService<IHashService>();
        _otpService = _serviceScope.ServiceProvider.GetRequiredService<IOtpService>();
        _testHarness = _serviceScope.ServiceProvider.GetTestHarness();
        _s3Settings = _serviceScope.ServiceProvider.GetRequiredService<IOptions<S3Settings>>().Value;

        var publisher = _serviceScope.ServiceProvider.GetRequiredService<IPublisher>();
        _authService = new AuthService(_dbConnectionFactory, _tokenWriter, publisher, _hashService);
    }

    [Fact]
    public async Task RegisterAsync_ShouldReturnSuccessAuthResult_WhenUserIsRegistered()
    {
        // Arrange
        _factory.S3.ClearSubstitute();
        var registerRequest = _fixture.Create<RegisterRequest>();

        var expectedIdentity = registerRequest.ToIdentity(_hashService);
        var expectedUser = registerRequest.ToUser();

        var expectedAccessToken = _tokenWriter.GenerateAccessToken(expectedIdentity);

        string imageKey = $"profile_images/{registerRequest.Id}";
        string backgroundImageKey = $"background_images/{registerRequest.Id}";

        var cancellationToken = CancellationToken.None;
        var objectMetadataResponse = new PutObjectResponse
        {
            HttpStatusCode = HttpStatusCode.OK
        };

        _factory.S3.PutObjectAsync(Arg.Is<PutObjectRequest>(r =>
                    r.BucketName == _s3Settings.BucketName &&
                    (r.Key == imageKey || r.Key == backgroundImageKey) &&
                    (r.ContentType == registerRequest.ProfileImage.ContentType || r.ContentType == registerRequest.BackgroundImage.ContentType) &&
                    (r.Metadata["x-amz-meta-file-name"] == registerRequest.ProfileImage.FileName ||
                     r.Metadata["x-amz-meta-file-name"] == registerRequest.BackgroundImage.FileName) ||
                    r.Metadata["x-amz-meta-extension"] == Path.GetExtension(registerRequest.ProfileImage.FileName) || 
                    r.Metadata["x-amz-meta-extension"] == Path.GetExtension(registerRequest.BackgroundImage.FileName)),
                cancellationToken)
            .Returns(objectMetadataResponse);

        await using var connection = await _dbConnectionFactory.CreateAsync(cancellationToken);
        await using var transaction = await connection.BeginTransactionAsync(cancellationToken);

        // Act
        var (authResult, refreshToken) = await _authService.RegisterAsync(registerRequest, cancellationToken);

        var createdIdentity =
            await connection.QueryFirstOrDefaultAsync<Identity>(SqlQueries.Authentication.GetIdentityById, new {registerRequest.Id}, transaction);

        var createdUser = await connection.QueryFirstOrDefaultAsync<User>(SqlQueries.Users.GetById, new {registerRequest.Id}, transaction);

        // Assert
        authResult.Id.Should().BeEquivalentTo(registerRequest.Id.Value);
        refreshToken.Should().BeNull();
        
        authResult.AccessToken.Should().BeNull();
        authResult.Success.Should().BeTrue();

        createdIdentity.Should().BeEquivalentTo(expectedIdentity, options =>
            options.Excluding(identity => identity.PasswordHash)
                .Using<string>(ctx => ctx.Subject.Length.Should().Be(ctx.Expectation.Length))
                .When(info => info.Path.EndsWith("PasswordHash") || info.Path.EndsWith("EmailHash")||
                              info.Path.EndsWith("PasswordSalt") || info.Path.EndsWith("EmailSalt")));

        createdUser.Should().BeEquivalentTo(expectedUser);

        await _factory.S3.Received().PutObjectAsync(Arg.Is<PutObjectRequest>(r =>
                r.BucketName == _s3Settings.BucketName &&
                (r.Key == imageKey || r.Key == backgroundImageKey) &&
                (r.ContentType == registerRequest.ProfileImage.ContentType || r.ContentType == registerRequest.BackgroundImage.ContentType) &&
                (r.Metadata["x-amz-meta-file-name"] == registerRequest.ProfileImage.FileName ||
                 r.Metadata["x-amz-meta-file-name"] == registerRequest.BackgroundImage.FileName) ||
                r.Metadata["x-amz-meta-extension"] == Path.GetExtension(registerRequest.ProfileImage.FileName) || 
                r.Metadata["x-amz-meta-extension"] == Path.GetExtension(registerRequest.BackgroundImage.FileName)),
            cancellationToken);
        
        var publishedMessage = _testHarness.Published.Select<UserCreated>(cancellationToken)
            .FirstOrDefault(publishedMessage => publishedMessage.Context.Message.Id == createdUser!.Id.Value);

        publishedMessage.Should().NotBeNull($"{nameof(UserCreated)} message was not published");
        publishedMessage!.Context.Message.Should().BeEquivalentTo(createdUser, options => options.ExcludingMissingMembers());
        publishedMessage.Context.RoutingKey().Should().BeEquivalentTo("user.created");
        
        await connection.ExecuteAsync(SqlQueries.Authentication.DeleteById, new {Id = registerRequest.Id.Value});
        await connection.ExecuteAsync(SqlQueries.Users.DeleteById, new {Id = registerRequest.Id.Value});
    }

    [Fact]
    public async Task RegisterAsync_ShouldReturnFailedAuthResult_WhenUserAlreadyExists()
    {
        // Arrange
        _factory.S3.ClearSubstitute();

        var registerRequest = _fixture.Create<RegisterRequest>();
        var identity = registerRequest.ToIdentity(_hashService);

        var cancellationToken = CancellationToken.None;

        await using var connection = await _dbConnectionFactory.CreateAsync(cancellationToken);
        await using var transaction = await connection.BeginTransactionAsync(cancellationToken);

        await connection.ExecuteAsync(SqlQueries.Authentication.Register, identity, transaction);
        await transaction.CommitAsync(cancellationToken);

        // Act
        var registerTask = async () => await _authService.RegisterAsync(registerRequest, cancellationToken);

        // Assert
        await registerTask.Should().ThrowAsync<NpgsqlException>();

        await _factory.S3.DidNotReceiveWithAnyArgs().PutObjectAsync(default, cancellationToken);
        await connection.ExecuteAsync(SqlQueries.Authentication.DeleteById, new {registerRequest.Id}, transaction);
        
        _testHarness.Published.Select<UserCreated>(cancellationToken)
            .FirstOrDefault(publishedMessage => publishedMessage.Context.Message.Id == identity.Id.Value)
            .Should()
            .BeNull();
    }

    [Fact]
    public async Task LoginAsync_ShouldReturnSuccessAuthResult_WhenUserExists_AndRefreshTokenExists()
    {
        // Arrange
        _factory.S3.ClearSubstitute();

        var registerRequest = _fixture.Create<RegisterRequest>();
        var loginRequest = _fixture.Build<LoginRequest>()
            .With(r => r.Username, registerRequest.Username)
            .With(r => r.Password, registerRequest.Password)
            .Create();

        var expectedIdentity = registerRequest.ToIdentity(_hashService);
        var expectedUser = registerRequest.ToUser();

        var expectedAccessToken = _tokenWriter.GenerateAccessToken(expectedIdentity);
        var expectedRefreshToken = _tokenWriter.GenerateRefreshToken(expectedIdentity.Id);
        
        var cancellationToken = CancellationToken.None;

        await using var connection = await _dbConnectionFactory.CreateAsync(cancellationToken);
        await using var transaction = await connection.BeginTransactionAsync(cancellationToken);

        await connection.ExecuteAsync(SqlQueries.Authentication.Register, expectedIdentity, transaction);
        await connection.ExecuteAsync(SqlQueries.Authentication.CreateRefreshToken,
            new {IdentityId = expectedIdentity.Id, expectedRefreshToken.Token, expectedRefreshToken.ExpirationDate}, transaction);

        string code = _otpService.CreateOneTimePassword(6);
        (string codeHash, string codeSalt) = _hashService.Hash(code);

        await connection.ExecuteAsync(SqlQueries.Authentication.CreateOtpCode, new
        {
            Id = registerRequest.Id,
            CodeHash = codeHash,
            Salt = codeSalt,
            ExpiresAt = DateTime.UtcNow.AddMinutes(10),
        }, transaction);
        
        await transaction.CommitAsync(cancellationToken);
        
        // Act
        await _authService.VerifyEmailAsync(registerRequest.Id, code, cancellationToken);
        var (authResult, refreshToken) = await _authService.LoginAsync(loginRequest, cancellationToken);

        // Assert
        authResult.Id.Should().BeEquivalentTo(registerRequest.Id.Value).And.BeEquivalentTo(expectedUser.Id.Value).And.BeEquivalentTo(expectedIdentity.Id.Value);
        refreshToken.Should().NotBeNull();

        authResult.AccessToken.Should().BeEquivalentTo(expectedAccessToken, options =>
        {
            options = options
                .Excluding(token => token.Value)
                .Using<DateTime>(ctx => ctx.Subject.Should().BeCloseTo(ctx.Expectation, TimeSpan.FromSeconds(1)))
                .When(info => info.Path.EndsWith("ExpirationDate"));

            return options.Using<string>(ctx => ctx.Subject.Length.Should().Be(ctx.Expectation.Length))
                .When(info => info.Path.EndsWith("Value"));
        });

        refreshToken.Should().BeEquivalentTo(expectedRefreshToken.ToResponse(), options =>
        {
            options = options
                .Using<DateTime>(ctx => ctx.Subject.Should().BeCloseTo(ctx.Expectation, TimeSpan.FromMilliseconds(500)))
                .When(info => info.Path.EndsWith("ExpirationDate"));

            return options.Using<string>(ctx => ctx.Subject.Length.Should().Be(ctx.Expectation.Length))
                .When(info => info.Path.EndsWith("Value"));
        });

        await connection.ExecuteAsync(SqlQueries.Authentication.DeleteById, new {registerRequest.Id}, transaction);
        await connection.ExecuteAsync(SqlQueries.Authentication.DeleteRefreshTokenById, new {expectedRefreshToken.Token}, transaction);
    }

    [Fact]
    public async Task LoginAsync_ShouldReturnSuccessAuthResult_WhenUserExists_AndRefreshTokenDoesNotExist()
    {
        // Arrange
        var registerRequest = _fixture.Create<RegisterRequest>();
        var loginRequest = _fixture.Build<LoginRequest>()
            .With(r => r.Username, registerRequest.Username)
            .With(r => r.Password, registerRequest.Password)
            .Create();

        var expectedIdentity = registerRequest.ToIdentity(_hashService);
        var expectedUser = registerRequest.ToUser();

        var expectedAccessToken = _tokenWriter.GenerateAccessToken(expectedIdentity);
        var expectedRefreshToken = _tokenWriter.GenerateRefreshToken(expectedIdentity.Id);

        var cancellationToken = CancellationToken.None;

        await using var connection = await _dbConnectionFactory.CreateAsync(cancellationToken);
        await using var transaction = await connection.BeginTransactionAsync(cancellationToken);

        await connection.ExecuteAsync(SqlQueries.Authentication.Register, expectedIdentity, transaction);
        string code = _otpService.CreateOneTimePassword(6);
        (string codeHash, string codeSalt) = _hashService.Hash(code);

        await connection.ExecuteAsync(SqlQueries.Authentication.CreateOtpCode, new
        {
            Id = registerRequest.Id,
            CodeHash = codeHash,
            Salt = codeSalt,
            ExpiresAt = DateTime.UtcNow.AddMinutes(10),
        });
        
        await transaction.CommitAsync(cancellationToken);
        
        // Act
        await _authService.VerifyEmailAsync(registerRequest.Id, code, cancellationToken);
        
        var (authResult, refreshToken) = await _authService.LoginAsync(loginRequest, cancellationToken);

        // Assert
        authResult.Id.Should().BeEquivalentTo(registerRequest.Id.Value).And.BeEquivalentTo(expectedUser.Id.Value).And.BeEquivalentTo(expectedIdentity.Id.Value);
        refreshToken.Should().NotBeNull();

        authResult.AccessToken.Should().BeEquivalentTo(expectedAccessToken, options =>
        {
            options = options
                .Excluding(token => token.Value)
                .Using<DateTime>(ctx => ctx.Subject.Should().BeCloseTo(ctx.Expectation, TimeSpan.FromSeconds(1)))
                .When(info => info.Path.EndsWith("ExpirationDate"));

            return options.Using<string>(ctx => ctx.Subject.Length.Should().Be(ctx.Expectation.Length))
                .When(info => info.Path.EndsWith("Value"));
        });

        refreshToken.Should().BeEquivalentTo(expectedRefreshToken.ToResponse(), options =>
        {
            options = options
                .Using<DateTime>(ctx => ctx.Subject.Should().BeCloseTo(ctx.Expectation, TimeSpan.FromMilliseconds(500)))
                .When(info => info.Path.EndsWith("ExpirationDate"));

            return options.Using<string>(ctx => ctx.Subject.Length.Should().Be(ctx.Expectation.Length))
                .When(info => info.Path.EndsWith("Value"));
        });

        byte[]? token = _tokenWriter.ParseRefreshToken(refreshToken!.Value);
        await connection.ExecuteAsync(SqlQueries.Authentication.DeleteById, new {registerRequest.Id}, transaction);
        await connection.ExecuteAsync(SqlQueries.Authentication.DeleteRefreshTokenById, new {Token = token}, transaction);
    }

    [Fact]
    public async Task LoginAsync_ShouldReturnFailedAuthResult_WhenUserExists_AndPasswordIsIncorrect()
    {
        // Arrange
        _factory.S3.ClearSubstitute();

        var registerRequest = _fixture.Create<RegisterRequest>();
        var loginRequest = _fixture.Build<LoginRequest>()
            .With(r => r.Username, registerRequest.Username)
            .WithAutoProperties()
            .Create();

        var identityToCreate = registerRequest.ToIdentity(_hashService);
        var cancellationToken = CancellationToken.None;

        await using var connection = await _dbConnectionFactory.CreateAsync(cancellationToken);
        await using var transaction = await connection.BeginTransactionAsync(cancellationToken);

        await connection.ExecuteAsync(SqlQueries.Authentication.Register, identityToCreate, transaction);
        await transaction.CommitAsync(cancellationToken);

        // Act
        var (authResult, refreshToken) = await _authService.LoginAsync(loginRequest, cancellationToken);

        // Assert
        authResult.Success.Should().Be(false);
        authResult.Error.Should().BeEquivalentTo("Incorrect credentials or the email address was not verified!");
        refreshToken.Should().BeNull();

        await connection.ExecuteAsync(SqlQueries.Authentication.DeleteById, new {registerRequest.Id}, transaction);
    }

    [Fact]
    public async Task LoginAsync_ShouldReturnFailedAuthResult_WhenUserDoesNotExist()
    {
        // Arrange
        _factory.S3.ClearSubstitute();

        var loginRequest = _fixture.Create<LoginRequest>();
        var cancellationToken = CancellationToken.None;

        // Act
        var (authResult, refreshToken) = await _authService.LoginAsync(loginRequest, cancellationToken);

        // Assert
        authResult.Success.Should().Be(false);
        authResult.Error.Should().BeEquivalentTo("Could not find user with given credentials");

        refreshToken.Should().BeNull();
    }

    [Fact]
    public async Task RefreshAccessToken_ShouldReturnNewAccessToken_WhenRefreshTokenIsValid()
    {
        // Arrange
        _factory.S3.ClearSubstitute();

        var registerRequest = _fixture.Create<RegisterRequest>();
        var cancellationToken = CancellationToken.None;

        var expectedIdentity = registerRequest.ToIdentity(_hashService);
        var expectedAccessToken = _tokenWriter.GenerateAccessToken(expectedIdentity);
        var expectedRefreshToken = _tokenWriter.GenerateRefreshToken(expectedIdentity.Id);

        await using var connection = await _dbConnectionFactory.CreateAsync(cancellationToken);
        await using var transaction = await connection.BeginTransactionAsync(cancellationToken);

        await connection.ExecuteAsync(SqlQueries.Authentication.Register, expectedIdentity, transaction);
        await connection.ExecuteAsync(SqlQueries.Authentication.CreateRefreshToken,
            new {IdentityId = expectedIdentity.Id, expectedRefreshToken.Token, expectedRefreshToken.ExpirationDate}, transaction);

        await transaction.CommitAsync(cancellationToken);
        string refreshTokenString = Convert.ToBase64String(expectedRefreshToken.Token);

        // Act
        var (authResult, refreshToken) = await _authService.RefreshAccessToken(refreshTokenString, cancellationToken);

        // Assert
        authResult.Success.Should().BeTrue();
        authResult.AccessToken.Should().BeEquivalentTo(expectedAccessToken, options =>
        {
            options = options
                .Excluding(token => token.Value)
                .Using<DateTime>(ctx => ctx.Subject.Should().BeCloseTo(ctx.Expectation, TimeSpan.FromSeconds(1)))
                .When(info => info.Path.EndsWith("ExpirationDate"));

            return options.Using<string>(ctx => ctx.Subject.Length.Should().Be(ctx.Expectation.Length))
                .When(info => info.Path.EndsWith("Value"));
        });

        refreshToken.Should().BeEquivalentTo(expectedRefreshToken.ToResponse(), options =>
        {
            options = options
                .Using<DateTime>(ctx => ctx.Subject.Should().BeCloseTo(ctx.Expectation, TimeSpan.FromMilliseconds(500)))
                .When(info => info.Path.EndsWith("ExpirationDate"));

            return options.Using<string>(ctx => ctx.Subject.Length.Should().Be(ctx.Expectation.Length))
                .When(info => info.Path.EndsWith("Value"));
        });

        await connection.ExecuteAsync(SqlQueries.Authentication.DeleteById, new {registerRequest.Id}, transaction);
        await connection.ExecuteAsync(SqlQueries.Authentication.DeleteRefreshTokenById, new {expectedRefreshToken.Token}, transaction);
    }

    [Fact]
    public async Task RefreshAccessToken_ShouldReturnFailedAuthResult_WhenRefreshTokenIsInvalid()
    {
        // Arrange
        _factory.S3.ClearSubstitute();

        var cancellationToken = CancellationToken.None;

        // Act
        var (authResult, refreshToken) = await _authService.RefreshAccessToken(string.Empty, cancellationToken);

        // Assert
        authResult.Success.Should().BeFalse();
        authResult.Error.Should().Be("Refresh token is not valid");
        refreshToken.Should().BeNull();
    }

    [Fact]
    public async Task RefreshAccessToken_ShouldReturnFailedAuthResult_WhenRefreshTokenIsValid_AndDoesNotExist()
    {
        // Arrange
        _factory.S3.ClearSubstitute();

        var cancellationToken = CancellationToken.None;
        var identityId = IdentityId.NewId();
        var refreshTokenToCreate = _tokenWriter.GenerateRefreshToken(identityId);
        
        string refreshTokenString = Convert.ToBase64String(refreshTokenToCreate.Token);

        // Act
        var (authResult, refreshToken) = await _authService.RefreshAccessToken(refreshTokenString, cancellationToken);

        // Assert
        authResult.Success.Should().BeFalse();
        authResult.Error.Should().Be("Refresh token is not valid");
        refreshToken.Should().BeNull();
    }

    [Fact]
    public async Task RefreshAccessToken_ShouldReturnFailedAuthResult_WhenRefreshTokenIsValid_AndRefreshTokenHasExpired()
    {
        // Arrange
        _factory.S3.ClearSubstitute();

        var cancellationToken = CancellationToken.None;
        var identity = _fixture.Create<RegisterRequest>().ToIdentity(_hashService);
        var refreshTokenToCreate = _tokenWriter.GenerateRefreshToken(identity.Id);

        await using var connection = await _dbConnectionFactory.CreateAsync(cancellationToken);
        await using var transaction = await connection.BeginTransactionAsync(cancellationToken);

        await transaction.CommitAsync(cancellationToken);
        string refreshTokenString = Convert.ToBase64String(refreshTokenToCreate.Token);

        await connection.ExecuteAsync(SqlQueries.Authentication.Register, identity, transaction);
        await connection.ExecuteAsync(SqlQueries.Authentication.CreateRefreshToken,
            new {IdentityId = identity.Id, refreshTokenToCreate.Token, ExpirationDate = refreshTokenToCreate.ExpirationDate.AddDays(-30).Date}, transaction);

        // Act
        var (authResult, refreshToken) = await _authService.RefreshAccessToken(refreshTokenString, cancellationToken);

        // Assert
        authResult.Success.Should().BeFalse();
        authResult.Error.Should().Be("Refresh token is not valid");
        refreshToken.Should().BeNull();
        
        await connection.ExecuteAsync(SqlQueries.Authentication.DeleteById, new {identity.Id}, transaction);
        await connection.ExecuteAsync(SqlQueries.Authentication.DeleteRefreshTokenById, new {refreshTokenToCreate.Token}, transaction);
    }

    [Fact]
    public async Task RevokeRefreshToken_ShouldReturnTrue_WhenTokenIsSuccessfullyRevoked()
    {
        // Arrange
        var registerRequest = _fixture.Create<RegisterRequest>();
        var identity = registerRequest.ToIdentity(_hashService);
        var refreshToken = _tokenWriter.GenerateRefreshToken(identity.Id);

        await using var connection = await _dbConnectionFactory.CreateAsync(CancellationToken.None);
        await using var transaction = await connection.BeginTransactionAsync(CancellationToken.None);

        await connection.ExecuteAsync(SqlQueries.Authentication.Register, identity, transaction);
        await connection.ExecuteAsync(SqlQueries.Authentication.CreateRefreshToken,
            new {IdentityId = identity.Id, refreshToken.Token, refreshToken.ExpirationDate}, transaction);

        await transaction.CommitAsync(CancellationToken.None);
        string refreshTokenString = Convert.ToBase64String(refreshToken.Token);

        // Act
        bool result = await _authService.RevokeRefreshToken(refreshTokenString, CancellationToken.None);

        // Assert
        result.Should().BeTrue();
        var revokedToken = await connection.QueryFirstOrDefaultAsync<RefreshToken>(SqlQueries.Authentication.GetRefreshToken, 
            new {refreshToken.Token});
        revokedToken.Should().BeNull();
        
        await connection.ExecuteAsync(SqlQueries.Authentication.DeleteById, new {identity.Id}, transaction);
        await connection.ExecuteAsync(SqlQueries.Authentication.DeleteRefreshTokenById, new {refreshToken.Token}, transaction);
    }

    [Fact]
    public async Task RevokeRefreshToken_ShouldReturnFalse_WhenTokenDoesNotExist()
    {
        // Arrange
        string invalidToken = Convert.ToBase64String(new byte[32]);
        var cancellationToken = CancellationToken.None;

        // Act
        bool result = await _authService.RevokeRefreshToken(invalidToken, cancellationToken);

        // Assert
        result.Should().BeFalse();
    }
    
    [Fact]
    public async Task DeleteByIdAsync_ShouldReturnTrue_WhenIdentityIsSuccessfullyDeleted()
    {
        // Arrange
        var registerRequest = _fixture.Create<RegisterRequest>();
        var cancellationToken = CancellationToken.None;
        
        var identity = registerRequest.ToIdentity(_hashService);
        var refreshToken = _tokenWriter.GenerateRefreshToken(identity.Id);
        
        await using var connection = await _dbConnectionFactory.CreateAsync(cancellationToken);
        await using var transaction = await connection.BeginTransactionAsync(cancellationToken);

        await connection.ExecuteAsync(SqlQueries.Authentication.Register, identity, transaction);
        await connection.ExecuteAsync(SqlQueries.Authentication.CreateRefreshToken,
            new {IdentityId = identity.Id, refreshToken.Token, refreshToken.ExpirationDate}, transaction);

        await transaction.CommitAsync(cancellationToken);
        string refreshTokenString = Convert.ToBase64String(refreshToken.Token);

        // Act
        bool result = await _authService.RevokeRefreshToken(refreshTokenString,cancellationToken);

        // Assert
        result.Should().BeTrue();
        var revokedToken = await connection.QueryFirstOrDefaultAsync<RefreshToken>(SqlQueries.Authentication.GetRefreshToken, new {refreshToken.Token});
        revokedToken.Should().BeNull();
        
        await connection.ExecuteAsync(SqlQueries.Authentication.DeleteById, new {identity.Id}, transaction);
        await connection.ExecuteAsync(SqlQueries.Authentication.DeleteRefreshTokenById, new {refreshToken.Token}, transaction);
    }

    [Fact]
    public async Task DeleteByIdAsync_ShouldReturnFalse_WhenIdentityDoesNotExist()
    {
        // Arrange
        var identityId = IdentityId.NewId();
        var cancellationToken = CancellationToken.None;

        // Act
        bool result = await _authService.DeleteByIdAsync(identityId, cancellationToken);

        // Assert
        result.Should().BeFalse();
    }

    public async ValueTask DisposeAsync()
    {
        await _serviceScope.DisposeAsync();
    }
}