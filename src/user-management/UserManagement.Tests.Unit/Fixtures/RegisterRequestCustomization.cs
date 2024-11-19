using AutoFixture;
using Microsoft.AspNetCore.Http;
using UserManagement.Application.Contracts.Requests.Auth;

namespace UserManagement.Tests.Unit.Fixtures;

internal sealed class RegisterRequestCustomization : ICustomization
{
    public void Customize(IFixture fixture)
    {
        fixture.Customize(new IFormFileCustomization());
        fixture.Customize(new DateOnlyCustomization());
        
        fixture.Customize<RegisterRequest>(composer => composer.FromFactory(() => new RegisterRequest
        {
            ProfileImage = fixture.Create<IFormFile>(),
            BackgroundImage = fixture.Create<IFormFile>(),
            Name = fixture.Create<string>(),
            Bio = fixture.Create<string>(),
            Location = fixture.Create<string>(),
            BirthDate = fixture.Create<DateOnly>(),
            Username = fixture.Create<string>(),
            Password = fixture.Create<string>(),
            Email = fixture.Create<string>()
        }).OmitAutoProperties());
    }
}