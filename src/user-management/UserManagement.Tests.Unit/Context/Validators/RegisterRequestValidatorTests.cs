using AutoFixture;
using FluentValidation.TestHelper;
using Microsoft.AspNetCore.Http;
using UserManagement.Application.Contracts.Requests.Auth;
using UserManagement.Application.Validators;
using UserManagement.Tests.Unit.Fixtures;

namespace UserManagement.Tests.Unit.Context.Validators;

public sealed class RegisterRequestValidatorTests
{
    private readonly RegisterRequestValidator _sut;
    private readonly Fixture _fixture;

    public RegisterRequestValidatorTests()
    {
        _sut = new RegisterRequestValidator();
        _fixture = new Fixture();
        _fixture.Customize(new IFormFileCustomization())
            .Customize(new DateOnlyCustomization());
    }

    [Theory]
    [InlineData("ValidUsername")]
    [InlineData("Short")]
    public void Username_ShouldNotHaveValidationError_ForValidUsernames(string username)
    {
        // Arrange
        var request = _fixture.Build<RegisterRequest>()
            .With(x => x.Username, username)
            .Create();

        // Act
        var result = _sut.TestValidate(request);

        // Assert
        result.ShouldNotHaveValidationErrorFor(r => r.Username);
    }

    [Theory]
    [InlineData(null)]
    [InlineData("")]
    [InlineData("ThisUsernameIsWayTooLongAndExceedsTheMaximumAllowedLengthOf100Characters")]
    public void Username_ShouldHaveValidationError_ForInvalidUsernames(string username)
    {
        var request = _fixture.Build<RegisterRequest>()
            .With(x => x.Username, username)
            .Create();

        var result = _sut.TestValidate(request);

        result.ShouldHaveValidationErrorFor(r => r.Username);
    }

    [Theory]
    [InlineData("ValidPassword")]
    [InlineData("ShortPwd")]
    public void Password_ShouldNotHaveValidationError_ForValidPasswords(string password)
    {
        var request = _fixture.Build<RegisterRequest>()
            .With(x => x.Password, password)
            .Create();

        var result = _sut.TestValidate(request);

        result.ShouldNotHaveValidationErrorFor(r => r.Password);
    }

    [Theory]
    [InlineData(null)]
    [InlineData("")]
    [InlineData("ThisIsAVeryLongPasswordThatExceedsTheMaxLengthOf30Characters")]
    public void Password_ShouldHaveValidationError_ForInvalidPasswords(string password)
    {
        var request = _fixture.Build<RegisterRequest>()
            .With(x => x.Password, password)
            .Create();

        var result = _sut.TestValidate(request);

        result.ShouldHaveValidationErrorFor(r => r.Password);
    }

    [Theory]
    [InlineData("email@example.com")]
    public void Email_ShouldNotHaveValidationError_ForValidEmails(string email)
    {
        var request = _fixture.Build<RegisterRequest>()
            .With(x => x.Email, email)
            .Create();

        var result = _sut.TestValidate(request);

        result.ShouldNotHaveValidationErrorFor(r => r.Email);
    }

    [Theory]
    [InlineData(null)]
    [InlineData("")]
    [InlineData("ThisEmailIsWayTooLongAndExceedsTheMaximumLengthOf50Characters@example.com")]
    public void Email_ShouldHaveValidationError_ForInvalidEmails(string email)
    {
        var request = _fixture.Build<RegisterRequest>()
            .With(x => x.Email, email)
            .Create();

        var result = _sut.TestValidate(request);

        result.ShouldHaveValidationErrorFor(r => r.Email);
    }

    [Fact]
    public void ProfileImage_ShouldHaveValidationError_WhenNullOrEmpty()
    {
        var request = _fixture.Build<RegisterRequest>()
            .With(x => x.ProfileImage, (IFormFile)null!)
            .Create();

        var result = _sut.TestValidate(request);

        result.ShouldHaveValidationErrorFor(r => r.ProfileImage);
    }

    [Fact]
    public void BackgroundImage_ShouldHaveValidationError_WhenNullOrEmpty()
    {
        var request = _fixture.Build<RegisterRequest>()
            .With(x => x.BackgroundImage, (IFormFile)null!)
            .Create();

        var result = _sut.TestValidate(request);

        result.ShouldHaveValidationErrorFor(r => r.BackgroundImage);
    }

    [Theory]
    [InlineData("Valid Name")]
    [InlineData("Another Valid Name")]
    public void Name_ShouldNotHaveValidationError_ForValidNames(string name)
    {
        var request = _fixture.Build<RegisterRequest>()
            .With(x => x.Name, name)
            .Create();

        var result = _sut.TestValidate(request);

        result.ShouldNotHaveValidationErrorFor(r => r.Name);
    }

    [Theory]
    [InlineData(null)]
    [InlineData("")]
    [InlineData("ThisNameIsWayTooLongAndExceedsTheMaximumAllowedLengthOf100Characters")]
    public void Name_ShouldHaveValidationError_ForInvalidNames(string name)
    {
        var request = _fixture.Build<RegisterRequest>()
            .With(x => x.Name, name)
            .Create();

        var result = _sut.TestValidate(request);

        result.ShouldHaveValidationErrorFor(r => r.Name);
    }

    [Theory]
    [InlineData("Short Bio")]
    [InlineData("A bio that is within the maximum length of 500 characters.")]
    public void Bio_ShouldNotHaveValidationError_ForValidBios(string bio)
    {
        var request = _fixture.Build<RegisterRequest>()
            .With(x => x.Bio, bio)
            .Create();

        var result = _sut.TestValidate(request);

        result.ShouldNotHaveValidationErrorFor(r => r.Bio);
    }

    [Fact]
    public void Bio_ShouldHaveValidationError_ForInvalidBios()
    {
        var request = _fixture.Build<RegisterRequest>()
            .With(x => x.Bio, string.Join(",", Enumerable.Repeat("d", 500)))
            .Create();

        var result = _sut.TestValidate(request);

        result.ShouldHaveValidationErrorFor(r => r.Bio);
    }

    [Theory]
    [InlineData("Location")]
    [InlineData("Another Valid Location")]
    public void Location_ShouldNotHaveValidationError_ForValidLocations(string location)
    {
        var request = _fixture.Build<RegisterRequest>()
            .With(x => x.Location, location)
            .Create();

        var result = _sut.TestValidate(request);

        result.ShouldNotHaveValidationErrorFor(r => r.Location);
    }

    [Theory]
    [InlineData("ThisLocationNameIsWayTooLongAndExceedsTheMaximumAllowedLengthOf100Characters" +
                "ThisLocationNameIsWayTooLongAndExceedsTheMaximumAllowedLengthOf100Characters")]
    public void Location_ShouldHaveValidationError_ForInvalidLocations(string location)
    {
        var request = _fixture.Build<RegisterRequest>()
            .With(x => x.Location, location)
            .Create();

        var result = _sut.TestValidate(request);

        result.ShouldHaveValidationErrorFor(r => r.Location);
    }

    [Theory]
    [InlineData("2000-01-01")]
    [InlineData("1989-12-31")] 
    public void BirthDate_ShouldNotHaveValidationError_ForValidBirthDates(string birthDate)
    {
        var request = _fixture.Build<RegisterRequest>()
            .With(x => x.BirthDate, DateOnly.Parse(birthDate))
            .Create();

        var result = _sut.TestValidate(request);

        result.ShouldNotHaveValidationErrorFor(r => r.BirthDate);
    }

    [Theory]
    [InlineData("1800-01-01")] // Too old
    [InlineData("2100-01-01")] // Too young (future date)
    public void BirthDate_ShouldHaveValidationError_ForInvalidBirthDates(string birthDate)
    {
        var request = _fixture.Build<RegisterRequest>()
            .With(x => x.BirthDate, DateOnly.Parse(birthDate))
            .Create();

        var result = _sut.TestValidate(request);

        result.ShouldHaveValidationErrorFor(r => r.BirthDate);
    }
}