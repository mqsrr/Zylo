using AutoFixture;
using FluentAssertions;
using Microsoft.Extensions.Logging;
using Npgsql;
using NSubstitute;
using NSubstitute.ExceptionExtensions;
using UserManagement.Application.Contracts.Requests.Auth;
using UserManagement.Application.Models;
using UserManagement.Application.Services;
using UserManagement.Application.Services.Abstractions;
using UserManagement.Tests.Unit.Fixtures;

namespace UserManagement.Tests.Unit.Context.Services;

public class ExceptionlessAuthServiceFacts
{
    private readonly IAuthService _authService;
    private readonly Fixture _fixture;
    private readonly ExceptionlessAuthService _sut;

    public ExceptionlessAuthServiceFacts()
    {
        _fixture = new Fixture();
        _fixture.Customize(new RegisterRequestCustomization())
            .Customize(new AuthenticationSuccessResultCustomization())
            .Customize(new DateOnlyCustomization());

        _authService = Substitute.For<IAuthService>();
        var logger = LoggerFactory.Create(builder => builder.AddConsole()).CreateLogger<ExceptionlessAuthService>();

        _sut = new ExceptionlessAuthService(_authService, logger);
    }

    [Fact]
    public async Task RegisterAsync_ShouldReturnsAuthResult_WhenThereIsNoException()
    {
        // Arrange
        var request = _fixture.Create<RegisterRequest>();
        var expectedAuthResult = _fixture.Create<AuthenticationResult>();
        var expectedRefreshToken = _fixture.Create<RefreshTokenResponse>();
        var cancellationToken = CancellationToken.None;
        
        _authService.RegisterAsync(request, cancellationToken)
            .Returns((expectedAuthResult, expectedRefreshToken));

        // Act
        var (authResult, refreshTokenResponse) = await _sut.RegisterAsync(request, cancellationToken);

        // Assert
        authResult.Should().BeEquivalentTo(expectedAuthResult);
        refreshTokenResponse.Should().BeEquivalentTo(expectedRefreshToken);
        
        await _authService.Received().RegisterAsync(request, cancellationToken);
    }

    [Fact]
    public async Task RegisterAsync_ShouldReturnsAuthFailure_WhenThereIsPostgresException()
    {
        // Arrange
        var request = _fixture.Create<RegisterRequest>();
        var postgresException = _fixture.Create<PostgresException>();
        
        var cancellationToken = CancellationToken.None;
        var expected = Tuple.Create<AuthenticationResult, RefreshTokenResponse?>(new AuthenticationResult
        {
            Success = false,
            Id = null,
            AccessToken = null,
            Error = postgresException.MessageText
        }, null);
        
        _authService.RegisterAsync(request, cancellationToken).ThrowsAsync(postgresException);

        // Act
        var result = await _sut.RegisterAsync(request, cancellationToken);

        // Assert
        result.Should().BeEquivalentTo(expected);
        
        await _authService.Received().RegisterAsync(request, cancellationToken);
    }

    [Fact]
    public async Task LoginAsync_ShouldReturnAuthResult_WhenThereIsNoException()
    {
        // Arrange
        var request = _fixture.Create<LoginRequest>();
        var expectedAuthResult = _fixture.Create<AuthenticationResult>();
        
        var expectedRefreshToken = _fixture.Create<RefreshTokenResponse>();
        var cancellationToken = CancellationToken.None;

        _authService.LoginAsync(request, cancellationToken)
            .Returns((expectedAuthResult, expectedRefreshToken));

        // Act
        var (authResult, refreshTokenResponse) = await _sut.LoginAsync(request, cancellationToken);

        // Assert
        authResult.Should().BeEquivalentTo(expectedAuthResult);
        refreshTokenResponse.Should().BeEquivalentTo(expectedRefreshToken);
        
        await _authService.Received().LoginAsync(request, cancellationToken);
    }

    [Fact]
    public async Task LoginAsync_ShouldReturnAuthFailure_WhenThereIsPostgresException()
    {
        // Arrange
        var request = _fixture.Create<LoginRequest>();
        var postgresException = _fixture.Create<PostgresException>();
        
        var cancellationToken = CancellationToken.None;
        var expected = Tuple.Create<AuthenticationResult, RefreshTokenResponse?>(new AuthenticationResult
        {
            Success = false,
            Id = null,
            AccessToken = null,
            Error = postgresException.MessageText
        }, null);
        
        _authService.LoginAsync(request, cancellationToken).ThrowsAsync(postgresException);

        // Act
        var result = await _sut.LoginAsync(request, cancellationToken);

        // Assert
        result.Should().BeEquivalentTo(expected);
        await _authService.Received().LoginAsync(request, cancellationToken);
    }

    [Fact]
    public async Task RefreshAccessToken_ShouldReturnAuthResult_WhenThereIsNoException()
    {
        // Arrange
        const string token = "refreshToken";
        var expectedAuthResult = _fixture.Create<AuthenticationResult>();
        
        var expectedRefreshToken = _fixture.Create<RefreshTokenResponse>();
        var cancellationToken = CancellationToken.None;

        _authService.RefreshAccessToken(token, cancellationToken)
            .Returns((expectedAuthResult, expectedRefreshToken));

        // Act
        var (authResult, refreshTokenResponse) = await _sut.RefreshAccessToken(token, cancellationToken);

        // Assert
        authResult.Should().BeEquivalentTo(expectedAuthResult);
        refreshTokenResponse.Should().BeEquivalentTo(expectedRefreshToken);

        await _authService.Received().RefreshAccessToken(token, cancellationToken);
    }

    [Fact]
    public async Task RefreshAccessToken_ShouldReturnAuthFailure_WhenThereIsPostgresException()
    {
        // Arrange
        const string token = "refreshToken";
        var postgresException = _fixture.Create<PostgresException>();
        
        var cancellationToken = CancellationToken.None;
        var expected = Tuple.Create<AuthenticationResult, RefreshTokenResponse?>(new AuthenticationResult
        {
            Success = false,
            Id = null,
            AccessToken = null,
            Error = postgresException.MessageText
        }, null);
        
        _authService.RefreshAccessToken(token, cancellationToken)
            .ThrowsAsync(postgresException);

        // Act
        var result = await _sut.RefreshAccessToken(token, cancellationToken);

        // Assert
        result.Should().BeEquivalentTo(expected);
        await _authService.Received().RefreshAccessToken(token, cancellationToken);
    }

    [Fact]
    public async Task RevokeRefreshToken_ShouldReturnTrue_WhenThereIsNoException()
    {
        // Arrange
        const string token = "refreshToken";
        var cancellationToken = CancellationToken.None;

        _authService.RevokeRefreshToken(token, cancellationToken).Returns(true);

        // Act
        bool result = await _sut.RevokeRefreshToken(token, cancellationToken);

        // Assert
        result.Should().BeTrue();
        await _authService.Received().RevokeRefreshToken(token, cancellationToken);
    }

    [Fact]
    public async Task RevokeRefreshToken_ShouldReturnFalse_WhenThereIsPostgresException()
    {
        // Arrange
        const string token = "refreshToken";
        var postgresException = _fixture.Create<PostgresException>();
        var cancellationToken = CancellationToken.None;

        _authService.RevokeRefreshToken(token, cancellationToken).ThrowsAsync(postgresException);

        // Act
        bool result = await _sut.RevokeRefreshToken(token, cancellationToken);

        // Assert
        result.Should().BeFalse();
        await _authService.Received().RevokeRefreshToken(token, cancellationToken);
    }

    [Fact]
    public async Task DeleteByIdAsync_ShouldReturnTrue_WhenThereIsNoException()
    {
        // Arrange
        var id = IdentityId.NewId();
        var cancellationToken = CancellationToken.None;

        _authService.DeleteByIdAsync(id, cancellationToken).Returns(true);

        // Act
        bool result = await _sut.DeleteByIdAsync(id, cancellationToken);

        // Assert
        result.Should().BeTrue();
        await _authService.Received().DeleteByIdAsync(id, cancellationToken);
    }

    [Fact]
    public async Task DeleteByIdAsync_ShouldReturnFalse_WhenThereIsPostgresException()
    {
        // Arrange
        var id = IdentityId.NewId();
        var postgresException = _fixture.Create<PostgresException>();
        var cancellationToken = CancellationToken.None;

        _authService.DeleteByIdAsync(id, cancellationToken).ThrowsAsync(postgresException);

        // Act
        bool result = await _sut.DeleteByIdAsync(id, cancellationToken);

        // Assert
        result.Should().BeFalse();
        await _authService.Received().DeleteByIdAsync(id, cancellationToken);
    }
}