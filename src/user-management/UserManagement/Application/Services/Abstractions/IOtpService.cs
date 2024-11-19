namespace UserManagement.Application.Services.Abstractions;

public interface IOtpService
{
    string CreateOneTimePassword(int length);
}