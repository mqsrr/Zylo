using AutoFixture;
using UserManagement.Application.Models;

namespace UserManagement.Tests.Unit.Fixtures;

internal sealed class AuthenticationSuccessResultCustomization : ICustomization
{
    public void Customize(IFixture fixture)
    {
        fixture.Customize(new DateTimeCustomization());
        fixture.Customize<AuthenticationResult>(composer => composer.FromFactory(() => new AuthenticationResult
        {
            Success = true,
            Id = Ulid.NewUlid(),
            AccessToken = fixture.Create<AccessToken>(),
            Error = null
        }).OmitAutoProperties());
    }
}

internal sealed class AuthenticationFailureResultCustomization : ICustomization
{
    public void Customize(IFixture fixture)
    {
        fixture.Customize(new DateTimeCustomization());
        fixture.Customize<AuthenticationResult>(composer => composer.FromFactory(() => new AuthenticationResult
        {
            Success = false,
            Error = "Invalid token"
        }).OmitAutoProperties());
    }
}