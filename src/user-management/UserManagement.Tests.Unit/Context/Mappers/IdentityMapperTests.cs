using AutoFixture;
using FluentAssertions;
using Microsoft.AspNetCore.Identity;
using NSubstitute;
using UserManagement.Application.Contracts.Requests.Auth;
using UserManagement.Application.Mappers;
using UserManagement.Application.Models;
using UserManagement.Application.Services.Abstractions;
using UserManagement.Tests.Unit.Fixtures;

namespace UserManagement.Tests.Unit.Context.Mappers;

public sealed class IdentityMapperTests
{
    private readonly Fixture _fixture;
    private readonly IHashService _hashService;

    public IdentityMapperTests()
    {
        _fixture = new Fixture();
        _fixture.Customize(new IFormFileCustomization())
            .Customize(new RegisterRequestCustomization());

        _hashService = Substitute.For<IHashService>();
    }

    [Fact]
    public void ToIdentity_ShouldMapRegisterRequestToIdentity_WithHashedPassword_AndEmail()
    {
        // Arrange
        var registerRequest = _fixture.Create<RegisterRequest>();
        const string hashedEmail = "HashedEmail";
        const string emailSalt = "EmailSalt";
        
        const string hashedPassword = "HashedPassword";
        const string passwordSalt = "PasswordSalt";
        
        _hashService.Hash(registerRequest.Email)
            .Returns((hashedEmail, emailSalt));
        
        _hashService.Hash(registerRequest.Password)
            .Returns((hashedPassword, passwordSalt));
        
        _hashService.VerifyHash(registerRequest.Password, hashedPassword, passwordSalt)
            .Returns(true);
        
        _hashService.VerifyHash(registerRequest.Email, hashedEmail, emailSalt)
            .Returns(true);
        
        // Act
        var identity = registerRequest.ToIdentity(_hashService);

        // Assert
        identity.Should().NotBeNull();
        identity.Id.Should().Be(registerRequest.Id);
        identity.Username.Should().Be(registerRequest.Username);
        
        bool passwordVerificationResult = _hashService.VerifyHash(registerRequest.Password, identity.PasswordHash, identity.PasswordSalt);
        bool emailVerificationResult = _hashService.VerifyHash(registerRequest.Email, identity.EmailHash, identity.EmailSalt);
        
        passwordVerificationResult.Should().BeTrue();
        emailVerificationResult.Should().BeTrue();
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