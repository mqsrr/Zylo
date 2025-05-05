using System.Net;

namespace NotificationService.Domain.Errors;

public sealed class UnexpectedError(Exception? innerException = null, string detail = "Please contact support if the issue persists.") : Error(detail, innerException)
{
    public override HttpStatusCode StatusCode => HttpStatusCode.InternalServerError;

    public override string Type => "https://datatracker.ietf.org/doc/html/rfc7231#section-6.6.1";

    public override string Title => "An unexpected error occurred";
}
