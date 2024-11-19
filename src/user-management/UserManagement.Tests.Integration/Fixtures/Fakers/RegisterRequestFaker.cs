using AutoBogus;
using AutoFixture;
using Microsoft.AspNetCore.Http;
using UserManagement.Application.Contracts.Requests.Auth;
using UserManagement.Application.Models;
using UserManagement.Tests.Integration.Fixtures.Customizations;

namespace UserManagement.Tests.Integration.Fixtures.Fakers;

public sealed class RegisterRequestFaker : AutoFaker<RegisterRequest>
{
    public RegisterRequestFaker()
    {
        var fixture = new Fixture();
        fixture.Customize(new IFormFileCustomization());
        
        RuleFor(r => r.Id, IdentityId.NewId());
        RuleFor(r => r.ProfileImage, fixture.Create<IFormFile>());
        RuleFor(r => r.BackgroundImage, fixture.Create<IFormFile>());
        RuleFor(r => r.Password, f => f.Internet.Password());
        RuleFor(r => r.Name, f => f.Person.FirstName);
        RuleFor(r => r.Username, f => f.Person.UserName);
        RuleFor(r => r.Bio, fixture.Create<string>());
        RuleFor(r => r.Email, f => f.Person.Email);
        RuleFor(r => r.Location, fixture.Create<string>());
        RuleFor(r => r.BirthDate, f => DateOnly.FromDateTime(f.Date.Between(DateTime.UtcNow.AddYears(-1), DateTime.UtcNow).Date));
    }
}