using AutoFixture;
using Microsoft.AspNetCore.Http;
using NSubstitute;

namespace UserManagement.Tests.Unit.Fixtures;

internal sealed class IFormFileCustomization : ICustomization
{

    public void Customize(IFixture fixture)
    {
        fixture.Customize<IFormFile>(composer => composer.FromFactory(() => Substitute.For<IFormFile>()).OmitAutoProperties());
    }
}