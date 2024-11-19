using AutoFixture;
using FluentValidation.TestHelper;
using UserManagement.Application.Contracts.Requests.Auth;
using UserManagement.Application.Validators;

namespace UserManagement.Tests.Unit.Context.Validators;

public sealed class LoginRequestValidatorTests
{
    private readonly LoginRequestValidator _sut;
    private readonly Fixture _fixture;

    public LoginRequestValidatorTests()
    {
        _sut = new LoginRequestValidator();
        _fixture = new Fixture();
    }

    [Theory]
    [InlineData("ValidUsername")]
    [InlineData("Short")]
    [InlineData("ValidUsernameWithinLimit")]
    public void Username_ShouldNotHaveValidationError_ForValidUsernames(string username)
    {
        // Arrange
        var request = _fixture.Build<LoginRequest>()
            .With(x => x.Username, username)
            .With(x => x.Password, "ValidPassword")
            .Create();

        // Act
        var result = _sut.TestValidate(request);

        // Assert
        result.ShouldNotHaveValidationErrorFor(r => r.Username);
    }

    [Theory]
    [InlineData(null)]
    [InlineData("")]
    [InlineData("ThisIsAReallyLongUsernameThatExceedsTheMaximumLengthAllowedForUsernames")]
    public void Username_ShouldHaveValidationError_ForInvalidUsernames(string username)
    {
        // Arrange
        var request = _fixture.Build<LoginRequest>()
            .With(x => x.Username, username)
            .With(x => x.Password, "ValidPassword")
            .Create();

        // Act
        var result = _sut.TestValidate(request);

        // Assert
        result.ShouldHaveValidationErrorFor(r => r.Username);
    }

    [Theory]
    [InlineData("ValidPassword")]
    [InlineData("ShortPwd")]
    [InlineData("ValidPasswordWithinLimit")]
    public void Password_ShouldNotHaveValidationError_ForValidPasswords(string password)
    {
        // Arrange
        var request = _fixture.Build<LoginRequest>()
            .With(x => x.Username, "ValidUsername")
            .With(x => x.Password, password)
            .Create();

        // Act
        var result = _sut.TestValidate(request);

        // Assert
        result.ShouldNotHaveValidationErrorFor(r => r.Password);
    }

    [Theory]
    [InlineData(null)]
    [InlineData("")]
    [InlineData("ThisIsAReallyLongPasswordThatExceedsTheAllowedMaximumLength")]
    public void Password_ShouldHaveValidationError_ForInvalidPasswords(string password)
    {
        // Arrange
        var request = _fixture.Build<LoginRequest>()
            .With(x => x.Username, "ValidUsername")
            .With(x => x.Password, password)
            .Create();

        // Act
        var result = _sut.TestValidate(request);

        // Assert
        result.ShouldHaveValidationErrorFor(r => r.Password);
    }
}