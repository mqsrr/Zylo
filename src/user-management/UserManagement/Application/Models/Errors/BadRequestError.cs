using System.Net;

namespace UserManagement.Application.Models.Errors;

public sealed class BadRequestError(string detail, Exception? innerException = null) : Error(detail, innerException)
{
    public override HttpStatusCode StatusCode { get; } = HttpStatusCode.BadRequest;

    public override string Type => "https://datatracker.ietf.org/doc/html/rfc7231#section-6.5.1";

    public override string Title => "Bad Request";
}
