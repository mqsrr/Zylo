namespace ApiGateway.Models;

public sealed class PaginatedResponse<T>
{
    public required List<T> Data { get; init; }

    public required int PerPage { get; init; }
    
    public required bool HasNextPage { get; init; }
    
    public required string Next { get; init; }
}