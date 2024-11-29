using Riok.Mapperly.Abstractions;
using UserManagement.Application.Contracts.Responses;
using UserManagement.Application.Models;

namespace UserManagement.Application.Mappers;

[Mapper]
internal static partial class FileMetadataMapper
{
    [MapProperty([nameof(FileMetadata.AccessUrl), nameof(FileMetadata.AccessUrl.Url)], [nameof(FileMetadataResponse.Url)])]
    internal static partial FileMetadataResponse ToResponse(this FileMetadata metadata);
}