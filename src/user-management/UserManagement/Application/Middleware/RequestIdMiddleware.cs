namespace UserManagement.Application.Middleware;

public class RequestIdMiddleware : IMiddleware
{

    public async Task InvokeAsync(HttpContext context, RequestDelegate next)
    {
        if (context.Request.Headers.TryGetValue("x-request-id", out var requestId))
        {
            context.TraceIdentifier = requestId!;
        }

        context.Response.Headers["x-request-id"] = context.TraceIdentifier;
        await next(context);
    }
}