using AutoFixture;

namespace UserManagement.Tests.Integration.Fixtures.Customizations;

internal sealed class DateTimeCustomization : ICustomization
{

    public void Customize(IFixture fixture)
    {
        fixture.Customize<DateTime>(composer => composer.FromFactory(() => DateTime.UtcNow));
    }
}