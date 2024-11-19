using AutoFixture;
using UserManagement.Application.Models;

namespace UserManagement.Tests.Integration.Fixtures.Customizations;

internal sealed class UserCustomization : ICustomization
{

    public void Customize(IFixture fixture)
    {
        fixture.Customize(new DateOnlyCustomization());
        fixture.Customize<User>(composer => composer.FromFactory(() => new User
        {
            Id = UserId.NewId(),
            ProfileImage = fixture.Create<FileMetadata>(),
            BackgroundImage = fixture.Create<FileMetadata>(),
            Name = fixture.Create<string>(),
            Bio = fixture.Create<string>(),
            Location = fixture.Create<string>(),
            BirthDate = fixture.Create<DateOnly>(),
            Username = fixture.Create<string>()
        }).OmitAutoProperties());
    }
}