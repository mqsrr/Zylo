namespace UserManagement.Application.Contracts.Requests.Auth;

public sealed class VerifyEmailAddressRequest
{
    public required string Otp { get; init; }
}