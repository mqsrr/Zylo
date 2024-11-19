using AutoBogus;
using AutoFixture;
using Microsoft.AspNetCore.Http;
using UserManagement.Application.Contracts.Requests.Users;
using UserManagement.Application.Models;
using UserManagement.Tests.Integration.Fixtures.Customizations;

namespace UserManagement.Tests.Integration.Fixtures.Fakers;

internal sealed class UpdateUserRequestFaker : AutoFaker<UpdateUserRequest>
{
    public UpdateUserRequestFaker()
    {
        var fixture = new Fixture();
        fixture.Customize(new IFormFileCustomization());
        
        RuleFor(r => r.Id, UserId.NewId());
        RuleFor(r => r.ProfileImage, fixture.Create<IFormFile>());
        RuleFor(r => r.BackgroundImage, fixture.Create<IFormFile>());
        RuleFor(r => r.Name, f => f.Person.FirstName);
        RuleFor(r => r.Bio, fixture.Create<string>());
        RuleFor(r => r.Location, fixture.Create<string>());
        RuleFor(r => r.BirthDate, f => DateOnly.FromDateTime(f.Date.Between(DateTime.UtcNow.AddYears(-1), DateTime.UtcNow).Date));
    }
}