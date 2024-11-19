using AutoFixture;
using Microsoft.AspNetCore.Http;
using NSubstitute;

namespace UserManagement.Tests.Integration.Fixtures.Customizations;

internal sealed class IFormFileCustomization : ICustomization
{
    public void Customize(IFixture fixture)
    {
        fixture.Customize<IFormFile>(composer => composer.FromFactory(() =>
        {
            var file =  Substitute.For<IFormFile>();
            
            file.FileName.Returns(fixture.Create<string>());
            file.ContentType.Returns("image/jpeg");
            file.OpenReadStream().Returns(Stream.Null);
            return file;
        }).OmitAutoProperties());
    }
}