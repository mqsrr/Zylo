using AutoFixture;
using Microsoft.AspNetCore.Http;
using UserManagement.Application.Contracts.Requests.Users;
using UserManagement.Application.Models;

namespace UserManagement.Tests.Integration.Fixtures.Customizations;

internal sealed class UpdateUserRequestCustomization : ICustomization
{
    public void Customize(IFixture fixture)
    {
        fixture.Customize(new IFormFileCustomization());
        fixture.Customize<UpdateUserRequest>(composer => composer.FromFactory(() => new UpdateUserRequest()
        {
            Id = UserId.NewId(),
            ProfileImage = fixture.Create<IFormFile>(),
            BackgroundImage = fixture.Create<IFormFile>(),
            Name = fixture.Create<string>(),
            Bio = fixture.Create<string>(),
            Location = fixture.Create<string>(),
            BirthDate = fixture.Create<DateOnly>(),
        }).OmitAutoProperties());
    }
}