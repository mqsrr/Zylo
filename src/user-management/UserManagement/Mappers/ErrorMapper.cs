using System.Diagnostics;
using Microsoft.AspNetCore.Mvc;
using UserManagement.Domain.Errors;

namespace UserManagement.Mappers;

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
}