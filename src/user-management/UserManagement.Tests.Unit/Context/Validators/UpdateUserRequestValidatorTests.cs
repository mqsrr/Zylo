using AutoFixture;
using FluentValidation.TestHelper;
using Microsoft.AspNetCore.Http;
using UserManagement.Application.Contracts.Requests.Users;
using UserManagement.Application.Validators;
using UserManagement.Tests.Unit.Fixtures;

namespace UserManagement.Tests.Unit.Context.Validators;

public sealed class UpdateUserRequestValidatorTests
{
    private readonly UpdateUserRequestValidator _sut;
    private readonly Fixture _fixture;

    public UpdateUserRequestValidatorTests()
    {
        _sut = new UpdateUserRequestValidator();
        _fixture = new Fixture();
        _fixture.Customize(new IFormFileCustomization())
            .Customize(new DateOnlyCustomization());
    }

    [Fact]
    public void ProfileImage_ShouldHaveValidationError_WhenNullOrEmpty()
    {
        // Arrange
        var request = _fixture.Build<UpdateUserRequest>()
            .With(x => x.ProfileImage, (IFormFile)null!)
            .Create();

        // Act
        var result = _sut.TestValidate(request);

        // Assert
        result.ShouldHaveValidationErrorFor(r => r.ProfileImage);
    }

    [Fact]
    public void BackgroundImage_ShouldHaveValidationError_WhenNullOrEmpty()
    {
        // Arrange
        var request = _fixture.Build<UpdateUserRequest>()
            .With(x => x.BackgroundImage, (IFormFile)null!)
            .Create();

        // Act
        var result = _sut.TestValidate(request);

        // Assert
        result.ShouldHaveValidationErrorFor(r => r.BackgroundImage);
    }

    [Theory]
    [InlineData("Valid Name")]
    [InlineData("ShortName")]
    public void Name_ShouldNotHaveValidationError_ForValidNames(string name)
    {
        // Arrange
        var request = _fixture.Build<UpdateUserRequest>()
            .With(x => x.Name, name)
            .Create();

        // Act
        var result = _sut.TestValidate(request);

        // Assert
        result.ShouldNotHaveValidationErrorFor(r => r.Name);
    }

    [Theory]
    [InlineData(null)]
    [InlineData("")]
    [InlineData("ThisNameIsWayTooLongAndExceedsTheMaximumAllowedLengthOf30Characters")]
    public void Name_ShouldHaveValidationError_ForInvalidNames(string name)
    {
        // Arrange
        var request = _fixture.Build<UpdateUserRequest>()
            .With(x => x.Name, name)
            .Create();

        // Act
        var result = _sut.TestValidate(request);

        // Assert
        result.ShouldHaveValidationErrorFor(r => r.Name);
    }

    [Theory]
    [InlineData("A short bio")]
    [InlineData("This is a bio within the limit of 500 characters.")]
    public void Bio_ShouldNotHaveValidationError_ForValidBios(string bio)
    {
        // Arrange
        var request = _fixture.Build<UpdateUserRequest>()
            .With(x => x.Bio, bio)
            .Create();

        // Act
        var result = _sut.TestValidate(request);

        // Assert
        result.ShouldNotHaveValidationErrorFor(r => r.Bio);
    }

    [Fact]
    public void Bio_ShouldHaveValidationError_ForInvalidBios()
    {
        // Arrange
        var request = _fixture.Build<UpdateUserRequest>()
            .With(x => x.Bio, string.Join(",", Enumerable.Repeat(" ", 500)))
            .Create();

        // Act
        var result = _sut.TestValidate(request);

        // Assert
        result.ShouldHaveValidationErrorFor(r => r.Bio);
    }

    [Theory]
    [InlineData("New York")]
    [InlineData("Another Valid Location")]
    public void Location_ShouldNotHaveValidationError_ForValidLocations(string location)
    {
        // Arrange
        var request = _fixture.Build<UpdateUserRequest>()
            .With(x => x.Location, location)
            .Create();

        // Act
        var result = _sut.TestValidate(request);

        // Assert
        result.ShouldNotHaveValidationErrorFor(r => r.Location);
    }

    [Theory]
    [InlineData(null)]
    [InlineData("")]
    [InlineData("ThisLocationNameIsWayTooLongAndExceedsTheMaximumAllowedLengthOf100Characters" +
                "ThisLocationNameIsWayTooLongAndExceedsTheMaximumAllowedLengthOf100Characters")]
    public void Location_ShouldHaveValidationError_ForInvalidLocations(string location)
    {
        // Arrange
        var request = _fixture.Build<UpdateUserRequest>()
            .With(x => x.Location, location)
            .Create();

        // Act
        var result = _sut.TestValidate(request);

        // Assert
        result.ShouldHaveValidationErrorFor(r => r.Location);
    }

    [Theory]
    [InlineData("2000-01-01")]
    [InlineData("1978-12-31")] 
    public void BirthDate_ShouldNotHaveValidationError_ForValidBirthDates(string birthDate)
    {
        // Arrange
        var request = _fixture.Build<UpdateUserRequest>()
            .With(x => x.BirthDate, DateOnly.Parse(birthDate))
            .Create();

        // Act
        var result = _sut.TestValidate(request);

        // Assert
        result.ShouldNotHaveValidationErrorFor(r => r.BirthDate);
    }

    [Theory]
    [InlineData("1800-01-01")]
    [InlineData("2100-01-01")] 
    public void BirthDate_ShouldHaveValidationError_ForInvalidBirthDates(string birthDate)
    {
        // Arrange
        var request = _fixture.Build<UpdateUserRequest>()
            .With(x => x.BirthDate, DateOnly.Parse(birthDate))
            .Create();

        // Act
        var result = _sut.TestValidate(request);

        // Assert
        result.ShouldHaveValidationErrorFor(r => r.BirthDate);
    }
}
