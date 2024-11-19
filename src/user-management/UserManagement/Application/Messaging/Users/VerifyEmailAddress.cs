namespace UserManagement.Application.Messaging.Users;
public interface VerifyEmailAddress
{
    string Otp { get; }

    string OtpIv { get; }

    string Email { get;}

    string EmailIv { get;}
}