using AutoFixture;

namespace UserManagement.Tests.Unit.Fixtures;

internal sealed class DateTimeCustomization : ICustomization
{

    public void Customize(IFixture fixture)
    {
        fixture.Customize<DateTime>(composer => composer.FromFactory(() => DateTime.UtcNow));
    }
}