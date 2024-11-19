using AutoFixture;
using FluentAssertions;
using UserManagement.Application.Contracts.Requests.Auth;
using UserManagement.Application.Mappers;
using UserManagement.Application.Models;
using UserManagement.Tests.Unit.Fixtures;

namespace UserManagement.Tests.Unit.Context.Mappers;

public sealed class UserMapperTests
{
    private readonly Fixture _fixture;

    public UserMapperTests()
    {
        _fixture = new Fixture();
        _fixture.Customize(new UserCustomization())
            .Customize(new RegisterRequestCustomization());
    }

    [Fact]
    public void ToUser_ShouldMapRegisterRequestToUser()
    {
        // Arrange
        var registerRequest = _fixture.Create<RegisterRequest>();

        // Act
        var user = registerRequest.ToUser();

        // Assert
        user.Should().NotBeNull();
        user.Id.Value.Should().Be(registerRequest.Id.Value);
        user.Id.Should().BeOfType<UserId>();
        user.Username.Should().Be(registerRequest.Username);
        user.Name.Should().Be(registerRequest.Name);
        user.Bio.Should().Be(registerRequest.Bio);
        user.Location.Should().Be(registerRequest.Location);
        user.BirthDate.Should().Be(registerRequest.BirthDate);
    }

    [Fact]
    public void ToResponse_ShouldMapUserToUserResponse()
    {
        // Arrange
        var user = _fixture.Create<User>();

        // Act
        var response = user.ToResponse();

        // Assert
        response.Should().NotBeNull();
        response.Id.Should().Be(user.Id.Value);
        response.Username.Should().Be(user.Username);
        response.Name.Should().Be(user.Name);
        response.Bio.Should().Be(user.Bio);
        response.Location.Should().Be(user.Location);
        response.BirthDate.Should().Be(user.BirthDate);
    }
}