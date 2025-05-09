﻿namespace UserManagement.Application.Contracts.Responses.Auth;

public sealed class FileMetadataResponse
{
    public required string Url { get; init; }

    public required string ContentType { get; init; }

    public required string FileName { get; init; }
}