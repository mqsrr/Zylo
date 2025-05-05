using System.Net;

namespace NotificationService.Domain.Errors;

public sealed class UnauthorizedError(string detail, Exception? innerException = null) : Error(detail, innerException)
{
    public override HttpStatusCode StatusCode => HttpStatusCode.Unauthorized;

    public override string Type => "https://datatracker.ietf.org/doc/html/rfc7235#section-3.1";

    public override string Title => "Unauthorized";
}
