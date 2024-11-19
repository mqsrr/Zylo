using AutoFixture;

namespace UserManagement.Tests.Unit.Fixtures;

internal sealed class DateOnlyCustomization : ICustomization
{

    public void Customize(IFixture fixture)
    {
        fixture.Customize<DateOnly>(composer => composer.FromFactory(() => DateOnly.MinValue).OmitAutoProperties());
    }
}