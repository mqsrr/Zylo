using System.Net;

namespace NotificationService.Domain.Errors;

public abstract class Error(string detail, Exception? innerException = null)
{
    public abstract HttpStatusCode StatusCode { get; }

    public abstract string Type { get; }

    public abstract string Title { get; }

    public string Detail { get; } = detail;

    public Exception? CausedBy { get; } = innerException;
}