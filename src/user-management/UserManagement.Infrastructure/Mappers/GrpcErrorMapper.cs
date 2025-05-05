using System.Net;
using Grpc.Core;
using UserManagement.Domain.Errors;

namespace UserManagement.Infrastructure.Mappers;

public static class GrpcErrorMapper
{
    public static Status ToGrpcStatus(this Error error)
    {
        return new Status(error.MapStatusCode(), error.Detail);
    }

    private static StatusCode MapStatusCode(this Error error)
    {
        return error.StatusCode switch
        {
            HttpStatusCode.BadRequest => StatusCode.InvalidArgument,
            HttpStatusCode.Unauthorized => StatusCode.Unauthenticated,
            HttpStatusCode.NotFound => StatusCode.NotFound,
            _ => StatusCode.Internal
        };
    }
}