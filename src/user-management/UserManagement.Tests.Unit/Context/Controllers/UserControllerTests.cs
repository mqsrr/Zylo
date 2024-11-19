using AutoFixture;
using FluentAssertions;
using Microsoft.AspNetCore.Mvc;
using NSubstitute;
using NSubstitute.ReturnsExtensions;
using UserManagement.Application.Contracts.Requests.Users;
using UserManagement.Application.Mappers;
using UserManagement.Application.Models;
using UserManagement.Application.Repositories.Abstractions;
using UserManagement.Controllers;
using UserManagement.Tests.Unit.Fixtures;

namespace UserManagement.Tests.Unit.Context.Controllers;

public sealed class UsersControllerTests
{
    private readonly UsersController _sut;
    private readonly IUserRepository _userRepository;
    private readonly Fixture _fixture;

    public UsersControllerTests()
    {
        _fixture = new Fixture();
        _fixture.Customize(new UpdateUserRequestCustomization())
            .Customize(new UserCustomization());

        _userRepository = Substitute.For<IUserRepository>();
        _sut = new UsersController(_userRepository);
    }

    [Fact]
    public async Task GetById_ShouldReturnOk_WhenUserExists()
    {
        // Arrange
        var user = _fixture.Create<User>();
        var cancellationToken = CancellationToken.None;

        _userRepository.GetByIdAsync(Arg.Is<UserId>(id => id == user.Id), cancellationToken)
            .Returns(user);

        // Act
        var result = await _sut.GetById(user.Id.ToString(), cancellationToken);

        // Assert
        result.Should().BeOfType<OkObjectResult>()
            .Which.Value.Should().BeEquivalentTo(user.ToResponse());

        await _userRepository.Received().GetByIdAsync(Arg.Is<UserId>(id => id == user.Id), cancellationToken);
    }

    [Fact]
    public async Task GetById_ShouldReturnNotFound_WhenUserDoesNotExist()
    {
        // Arrange
        var userId = UserId.NewId();
        var cancellationToken = CancellationToken.None;

        _userRepository.GetByIdAsync(Arg.Is<UserId>(id => id == userId), cancellationToken)
            .ReturnsNull();

        // Act
        var result = await _sut.GetById(userId.ToString(), cancellationToken);

        // Assert
        result.Should().BeOfType<NotFoundResult>();
        
        await _userRepository.Received().GetByIdAsync(Arg.Is<UserId>(id => id == userId), cancellationToken);
    }

    [Fact]
    public async Task Update_ShouldReturnNoContent_WhenUpdateIsSuccessful()
    {
        // Arrange
        var request = _fixture.Create<UpdateUserRequest>();
        var cancellationToken = CancellationToken.None;

        _userRepository.UpdateAsync(
                Arg.Is<UpdateUserRequest>(r => r.Id == request.Id),
                request.ProfileImage,
                request.BackgroundImage,
                cancellationToken)
            .Returns(true);

        // Act
        var result = await _sut.Update(request.Id.ToString(), request, cancellationToken);

        // Assert
        result.Should().BeOfType<NoContentResult>();
        
        await _userRepository.Received().UpdateAsync(
            Arg.Is<UpdateUserRequest>(r => r.Id == request.Id),
            request.ProfileImage,
            request.BackgroundImage,
            cancellationToken);
    }

    [Fact]
    public async Task Update_ShouldReturnBadRequest_WhenUpdateFails()
    {
        // Arrange
        var request = _fixture.Create<UpdateUserRequest>();
        var cancellationToken = CancellationToken.None;
        
        _userRepository.UpdateAsync(
                Arg.Is<UpdateUserRequest>(r => r.Id == request.Id),
                request.ProfileImage,
                request.BackgroundImage,
                cancellationToken)
            .Returns(false);

        // Act
        var result = await _sut.Update(request.Id.ToString(), request, cancellationToken);

        // Assert
        result.Should().BeOfType<BadRequestResult>();

        await _userRepository.Received().UpdateAsync(
            Arg.Is<UpdateUserRequest>(r => r.Id == request.Id),
            request.ProfileImage,
            request.BackgroundImage,
            cancellationToken);
    }

    [Fact]
    public async Task DeleteById_ShouldReturnNoContent_WhenDeletionIsSuccessful()
    {
        // Arrange
        var userId = UserId.NewId();
        var cancellationToken = CancellationToken.None;
        
        _userRepository.DeleteByIdAsync(Arg.Is<UserId>(id => id == userId), cancellationToken)
            .Returns(true);

        // Act
        var result = await _sut.DeleteById(userId.ToString(), cancellationToken);

        // Assert
        result.Should().BeOfType<NoContentResult>();

        await _userRepository.Received().DeleteByIdAsync(Arg.Is<UserId>(id => id == userId), cancellationToken);
    }

    [Fact]
    public async Task DeleteById_ShouldReturnNotFound_WhenDeletionFails()
    {
        // Arrange
        var userId = UserId.NewId();
        var cancellationToken = CancellationToken.None;
        
        _userRepository.DeleteByIdAsync(Arg.Is<UserId>(id => id == userId), cancellationToken)
            .Returns(false);

        // Act
        var result = await _sut.DeleteById(userId.ToString(),cancellationToken);

        // Assert
        result.Should().BeOfType<NotFoundResult>();

        await _userRepository.Received().DeleteByIdAsync(Arg.Is<UserId>(id => id == userId), cancellationToken);
    }
}