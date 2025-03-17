namespace NotificationService.Middleware;

public class RequestIdMiddleware
{
    private readonly RequestDelegate _next;

    public RequestIdMiddleware(RequestDelegate next)
    {
        _next = next;
    }

    public async Task Invoke(HttpContext context)
    {
        if (context.Request.Headers.TryGetValue("x-request-id", out var requestId))
        {
            context.TraceIdentifier = requestId!;
        }
        
        context.Response.Headers["x-request-id"] = context.TraceIdentifier;
        await _next(context);
    }
}