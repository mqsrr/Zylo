using AutoFixture;

namespace UserManagement.Tests.Integration.Fixtures.Customizations;

internal sealed class DateOnlyCustomization : ICustomization
{

    public void Customize(IFixture fixture)
    {
        fixture.Customize<DateOnly>(composer => composer.FromFactory(() => DateOnly.MinValue).OmitAutoProperties());
    }
}