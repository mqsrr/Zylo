using AutoFixture;
using FluentAssertions;
using Microsoft.AspNetCore.Identity;
using UserManagement.Application.Contracts.Requests.Auth;
using UserManagement.Application.Mappers;
using UserManagement.Application.Models;
using UserManagement.Tests.Unit.Fixtures;

namespace UserManagement.Tests.Unit.Context.Mappers;

public sealed class IdentityMapperTests
{
    private readonly Fixture _fixture;
    private readonly PasswordHasher<Identity> _passwordHasher;

    public IdentityMapperTests()
    {
        _fixture = new Fixture();
        _fixture.Customize(new IFormFileCustomization())
            .Customize(new RegisterRequestCustomization());
        
        _passwordHasher = new PasswordHasher<Identity>();
    }

    [Fact]
    public void ToIdentity_ShouldMapRegisterRequestToIdentity_WithHashedPassword()
    {
        // Arrange
        var registerRequest = _fixture.Create<RegisterRequest>();

        // Act
        var identity = registerRequest.ToIdentity();

        // Assert
        identity.Should().NotBeNull();
        identity.Id.Should().Be(registerRequest.Id);
        identity.Username.Should().Be(registerRequest.Username);
        identity.Email.Should().Be(registerRequest.Email);
        
        var passwordVerificationResult = _passwordHasher.VerifyHashedPassword(identity, identity.PasswordHash, registerRequest.Password);
        passwordVerificationResult.Should().Be(PasswordVerificationResult.Success);
    }

    [Fact]
    public void ToResponse_ShouldMapRefreshTokenToRefreshTokenResponse()
    {
        // Arrange
        var refreshToken = _fixture.Build<RefreshToken>()
            .With(t => t.IdentityId, Ulid.NewUlid())
            .WithAutoProperties()
            .Create();

        // Act
        var response = refreshToken.ToResponse();

        // Assert
        response.Should().NotBeNull();
        response.Value.Should().Be(Convert.ToBase64String(refreshToken.Token));
        response.ExpirationDate.Should().Be(refreshToken.ExpirationDate);
        response.Revoked.Should().Be(refreshToken.Revoked);
    }
}