using Azure.Security.KeyVault.Secrets;
using AutoFixture;
using FluentAssertions;
using UserManagement.Application.Services;

namespace UserManagement.Tests.Unit.Context.Services;

public sealed class PrefixKeyVaultSecretManagerTests
{
    private readonly PrefixKeyVaultSecretManager _sut;
    private readonly Fixture _fixture;

    public PrefixKeyVaultSecretManagerTests()
    {
        _fixture = new Fixture();
        
        var prefixes = new List<string> { "TestPrefix", "ProdPrefix" };
        _sut = new PrefixKeyVaultSecretManager(prefixes);
    }

    [Fact]
    public void Load_ShouldReturnTrue_WhenSecretNameStartsWithAnyPrefix()
    {
        // Arrange
        var secretName = "TestPrefix-MySecret";
        var secretProperties = new SecretProperties(secretName);

        // Act
        var result = _sut.Load(secretProperties);

        // Assert
        result.Should().BeTrue();
    }

    [Fact]
    public void Load_ShouldReturnFalse_WhenSecretNameDoesNotStartWithAnyPrefix()
    {
        // Arrange
        var secretName = "UnknownPrefix-MySecret";
        var secretProperties = new SecretProperties(secretName);

        // Act
        var result = _sut.Load(secretProperties);

        // Assert
        result.Should().BeFalse();
    }

    [Fact]
    public void GetKey_ShouldReturnKeyWithoutPrefix_WhenSecretNameStartsWithMatchingPrefix()
    {
        // Arrange
        var secretName = "TestPrefix-Section--Key";
        var expectedKey = "Section:Key";
        var keyVaultSecret = new KeyVaultSecret(secretName, _fixture.Create<string>());

        // Act
        var result = _sut.GetKey(keyVaultSecret);

        // Assert
        result.Should().Be(expectedKey);
    }

    [Fact]
    public void GetKey_ShouldReturnCorrectKey_WhenSecretNameContainsDoubleDashes()
    {
        // Arrange
        var secretName = "ProdPrefix-AnotherSection--AnotherKey";
        var expectedKey = "AnotherSection:AnotherKey";
        var keyVaultSecret = new KeyVaultSecret(secretName, _fixture.Create<string>());

        // Act
        var result = _sut.GetKey(keyVaultSecret);

        // Assert
        result.Should().Be(expectedKey);
    }
}
