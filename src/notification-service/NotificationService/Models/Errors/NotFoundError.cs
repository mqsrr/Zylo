using System.Net;

namespace NotificationService.Models.Errors;

public sealed class NotFoundError(string detail, Exception? innerException = null) : Error(detail, innerException)
{
    public override HttpStatusCode StatusCode => HttpStatusCode.NotFound;

    public override string Type => "https://datatracker.ietf.org/doc/html/rfc7231#section-6.5.4";

    public override string Title => "Not Found";
}