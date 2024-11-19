using System.IdentityModel.Tokens.Jwt;
using AutoFixture;
using FluentAssertions;
using Microsoft.Extensions.Options;
using NSubstitute;
using UserManagement.Application.Models;
using UserManagement.Application.Services;
using UserManagement.Application.Settings;

namespace UserManagement.Tests.Unit.Context.Services;

public sealed class TokenWriterTests
{
    private readonly JwtSettings _jwtSettings;
    private readonly Fixture _fixture;
    private readonly TimeProvider _timeProvider;
    private readonly TokenWriter _sut;

    public TokenWriterTests()
    {
        _fixture = new Fixture();
        _jwtSettings = new JwtSettings
        {
            Secret = "supersecretkey1234567890sdkljfhjsdlfjsdl",
            Issuer = "TestIssuer",
            Audience = "TestAudience",
            Expire = 60
        };
        
        var options = Substitute.For<IOptions<JwtSettings>>();
        options.Value.Returns(_jwtSettings);

        _timeProvider = Substitute.For<TimeProvider>();
        _timeProvider.GetUtcNow().Returns(DateTimeOffset.UtcNow);
        
        _sut = new TokenWriter(options, _timeProvider);
    }

    [Fact]
    public void ParseRefreshToken_ShouldReturnByteArray_WhenTokenIsValidBase64()
    {
        // Arrange
        string tokenString = Convert.ToBase64String(_fixture.Create<byte[]>());

        // Act
        byte[]? result = _sut.ParseRefreshToken(tokenString);

        // Assert
        result.Should().NotBeNull();
        result.Length.Should().BeGreaterThan(0);
    }

    [Fact]
    public void ParseRefreshToken_ShouldReturnNull_WhenTokenIsInvalidBase64()
    {
        // Arrange
        string largeInvalidToken = Convert.ToBase64String(new byte[1025]);
        string invalidToken = "invalid_token";

        // Act
        byte[]? firstResult = _sut.ParseRefreshToken(largeInvalidToken);
        byte[]? secondResult = _sut.ParseRefreshToken(invalidToken);

        // Assert
        firstResult.Should().BeNull();
        secondResult.Should().BeNull();
    }

    [Fact]
    public void ParseRefreshToken_ShouldReturnNull_WhenTokenIsNull()
    {

        // Act
        byte[]? result = _sut.ParseRefreshToken(string.Empty);

        // Assert
        result.Should().BeNull();
    }

    [Fact]
    public void GenerateAccessToken_ShouldReturnAccessToken_WithExpectedClaimsAndExpiration()
    {
        // Arrange
        var identity = _fixture.Build<Identity>()
            .With(i => i.EmailVerified, true)
            .With(i => i.Id, IdentityId.NewId())
            .Create();

        // Act
        var result = _sut.GenerateAccessToken(identity);

        // Assert
        result.Should().NotBeNull();
        result.ExpirationDate.Should().BeCloseTo(_timeProvider.GetUtcNow().DateTime.AddMinutes(_jwtSettings.Expire), TimeSpan.FromHours(2));

        var handler = new JwtSecurityTokenHandler();
        var token = handler.ReadJwtToken(result.Value);

        token.Claims.Should().Contain(c => c.Type == JwtRegisteredClaimNames.Sub && c.Value == identity.Id.ToString());
        token.Claims.Should().Contain(c => c.Type == JwtRegisteredClaimNames.Email && c.Value == identity.Email);
        token.Claims.Should().Contain(c => c.Type == "email-verified" && c.Value == identity.EmailVerified.ToString());
        token.Issuer.Should().Be(_jwtSettings.Issuer);
        token.Audiences.Should().Contain(_jwtSettings.Audience);
    }

    [Fact]
    public void GenerateRefreshToken_ShouldReturnRefreshToken_WithExpectedProperties()
    {
        // Arrange
        var identityId = IdentityId.NewId();

        // Act
        var result = _sut.GenerateRefreshToken(identityId);

        // Assert
        result.Should().NotBeNull();
        result.Token.Should().NotBeNullOrEmpty();
        result.IdentityId.Should().Be(identityId.Value);
        result.ExpirationDate.Should().BeCloseTo(DateTime.UtcNow.AddDays(30), TimeSpan.FromSeconds(5));
    }
}
