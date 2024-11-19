using AutoFixture;
using FluentAssertions;
using Microsoft.AspNetCore.Http;
using Microsoft.Extensions.Options;
using NSubstitute;
using UserManagement.Application.Contracts.Requests.Users;
using UserManagement.Application.Models;
using UserManagement.Application.Repositories;
using UserManagement.Application.Repositories.Abstractions;
using UserManagement.Application.Services.Abstractions;
using UserManagement.Application.Settings;
using UserManagement.Tests.Unit.Fixtures;

namespace UserManagement.Tests.Unit.Context.Repositories;

public sealed class CachedUserRepositoryFacts
{
    private const string CacheKey = "user-management";

    private readonly IUserRepository _userRepository;
    private readonly ICacheService _cacheService;
    private readonly IOptions<S3Settings> _s3Settings;
    private readonly Fixture _fixture;

    private readonly CachedUserRepository _sut;

    public CachedUserRepositoryFacts()
    {
        _fixture = new Fixture();
        _fixture.Customize(new UserCustomization())
            .Customize(new UpdateUserRequestCustomization());
        
        _userRepository = Substitute.For<IUserRepository>();
        _cacheService = Substitute.For<ICacheService>();

        _s3Settings = Options.Create(_fixture.Build<S3Settings>()
            .With(s => s.PresignedUrlExpire, TimeSpan.FromSeconds(3600).TotalSeconds)
            .Create());

        _sut = new CachedUserRepository(_userRepository, _cacheService, _s3Settings);
    }

    [Fact]
    public async Task GetById_ShouldReturnUser_FromCache_WhenUserCacheExists()
    {
        // Arrange
        var userId = UserId.NewId();
        var cachedUser = _fixture.Create<User>();
        var cancellationToken = CancellationToken.None;

        _cacheService.GetOrCreateAsync(CacheKey,
                Arg.Is<string>(s => s.Equals(userId.ToString(), StringComparison.Ordinal)),
                Arg.Any<Func<Task<User?>>>(),
                TimeSpan.FromMinutes(_s3Settings.Value.PresignedUrlExpire))
            .Returns(cachedUser);

        // Act
        var result = await _sut.GetByIdAsync(userId, cancellationToken);

        // Assert
        result.Should().BeEquivalentTo(cachedUser);

        await _cacheService.Received().GetOrCreateAsync(CacheKey,
            Arg.Is<string>(s => s.Equals(userId.ToString(), StringComparison.Ordinal)),
            Arg.Any<Func<Task<User?>>>(),
            TimeSpan.FromMinutes(_s3Settings.Value.PresignedUrlExpire));

        await _userRepository.DidNotReceiveWithAnyArgs().GetByIdAsync(default, cancellationToken);
    }

    [Fact]
    public async Task GetById_ShouldReturnUser_FromWrappedService_WhenUserCacheDoesNotExist()
    {
        // Arrange
        var userId = UserId.NewId();
        var expectedUser = _fixture.Create<User>();
        var cancellationToken = CancellationToken.None;

        _cacheService.GetOrCreateAsync(CacheKey,
                Arg.Is<string>(s => s.Equals(userId.ToString(), StringComparison.Ordinal)),
                Arg.Any<Func<Task<User?>>>(),
                TimeSpan.FromMinutes(_s3Settings.Value.PresignedUrlExpire))
            .Returns(callInfo => callInfo.Arg<Func<Task<User?>>>().Invoke());
        
        _userRepository.GetByIdAsync(userId, cancellationToken).Returns(expectedUser);

        // Act
        var result = await _sut.GetByIdAsync(userId, cancellationToken);

        // Assert
        result.Should().BeEquivalentTo(expectedUser);

        await _cacheService.Received().GetOrCreateAsync(CacheKey,
            Arg.Is<string>(s => s.Equals(userId.ToString(), StringComparison.Ordinal)),
            Arg.Any<Func<Task<User?>>>(),
            TimeSpan.FromMinutes(_s3Settings.Value.PresignedUrlExpire));

        await _userRepository.Received().GetByIdAsync(userId, cancellationToken);
    }

    [Theory]
    [InlineData(true)]
    [InlineData(false)]
    public async Task CreateAsync_ShouldReturnWrappedServiceResponse(bool isSuccessful)
    {
        // Arrange
        var user = _fixture.Create<User>();
        var profileImage = Substitute.For<IFormFile>();
        var backgroundImage = Substitute.For<IFormFile>();
        var cancellationToken = CancellationToken.None;

        _userRepository.CreateAsync(user, profileImage, backgroundImage, cancellationToken)
            .Returns(isSuccessful);

        // Act
        bool result = await _sut.CreateAsync(user, profileImage, backgroundImage, cancellationToken);

        // Assert
        result.Should().Be(isSuccessful);
        await _userRepository.Received().CreateAsync(user, profileImage, backgroundImage, cancellationToken);
    }

    [Fact]
    public async Task UpdateAsync_ShouldReturnTrue_WhenUserIsUpdated_AndRemoveCache()
    {
        // Arrange
        var updateUserRequest = _fixture.Create<UpdateUserRequest>();
        var cancellationToken = CancellationToken.None;

        var profileImage = Substitute.For<IFormFile>();
        var backgroundImage = Substitute.For<IFormFile>();

        _userRepository.UpdateAsync(updateUserRequest, profileImage, backgroundImage, cancellationToken)
            .Returns(true);

        _cacheService.HRemoveAsync(CacheKey, Arg.Is<string>(s => s.Equals(updateUserRequest.Id.ToString(), StringComparison.Ordinal)))
            .Returns(Task.CompletedTask);

        // Act
        bool result = await _sut.UpdateAsync(updateUserRequest, profileImage, backgroundImage, cancellationToken);

        // Assert
        result.Should().Be(true);

        await _userRepository.Received().UpdateAsync(updateUserRequest, profileImage, backgroundImage, cancellationToken);
        await _cacheService.Received().HRemoveAsync(CacheKey, Arg.Is<string>(s => s.Equals(updateUserRequest.Id.ToString(), StringComparison.Ordinal)));
    }

    [Fact]
    public async Task UpdateAsync_ShouldReturnFalse_WhenUserIsNotUpdated()
    {
        // Arrange
        var updateUserRequest = _fixture.Create<UpdateUserRequest>();
        var profileImage = Substitute.For<IFormFile>();
        var backgroundImage = Substitute.For<IFormFile>();
        var cancellationToken = CancellationToken.None;

        _userRepository.UpdateAsync(updateUserRequest, profileImage, backgroundImage, cancellationToken)
            .Returns(false);

        // Act
        bool result = await _sut.UpdateAsync(updateUserRequest, profileImage, backgroundImage, cancellationToken);

        // Assert
        result.Should().Be(false);

        await _userRepository.Received().UpdateAsync(updateUserRequest, profileImage, backgroundImage, cancellationToken);
        await _cacheService.DidNotReceiveWithAnyArgs().HRemoveAsync(default!, default!);
    }

    [Fact]
    public async Task DeleteByIdAsync_ShouldReturnTrue_WhenUserIsDeleted_AndRemoveCache()
    {
        // Arrange
        var userId = UserId.NewId();
        var cancellationToken = CancellationToken.None;

        _userRepository.DeleteByIdAsync(userId, cancellationToken)
            .Returns(true);

        _cacheService.HRemoveAsync(CacheKey, Arg.Is<string>(s => s.Equals(userId.ToString(), StringComparison.Ordinal)))
            .Returns(Task.CompletedTask);

        // Act
        bool result = await _sut.DeleteByIdAsync(userId, cancellationToken);

        // Assert
        result.Should().Be(true);

        await _userRepository.Received().DeleteByIdAsync(userId, cancellationToken);
        await _cacheService.Received().HRemoveAsync(CacheKey, Arg.Is<string>(s => s.Equals(userId.ToString(), StringComparison.Ordinal)));
    }
    
    [Fact]
    public async Task DeleteByIdAsync_ShouldReturnFalse_WhenUserIsNotDeleted()
    {
        // Arrange
        var userId = UserId.NewId();
        var cancellationToken = CancellationToken.None;

        _userRepository.DeleteByIdAsync(userId, cancellationToken)
            .Returns(false);
        
        // Act
        bool result = await _sut.DeleteByIdAsync(userId, cancellationToken);

        // Assert
        result.Should().Be(false);

        await _userRepository.Received().DeleteByIdAsync(userId, cancellationToken);
        await _cacheService.DidNotReceiveWithAnyArgs().HRemoveAsync(default!, default!);
    }
}