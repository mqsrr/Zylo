using System.Diagnostics;
using System.Net;
using Grpc.Core;
using Microsoft.AspNetCore.Mvc;
using UserManagement.Application.Models.Errors;

namespace UserManagement.Application.Mappers;

public static class ErrorMapper
{
    public static ObjectResult ToProblemObjectResult(this Error error, string traceId)
    {
        traceId = Activity.Current?.Id ?? traceId;
        var problemDetails = new ProblemDetails
        {
            Title = error.Title,
            Detail = error.Detail,
            Status = (int)error.StatusCode,
            Type = error.Type,
            Extensions =
            {
                ["traceId"] = traceId
            }
        };

        return new ObjectResult(problemDetails)
        {
            StatusCode = problemDetails.Status
        };
    }

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