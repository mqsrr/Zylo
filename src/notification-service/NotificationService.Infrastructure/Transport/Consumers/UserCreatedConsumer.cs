using FluentEmail.Core;
using Microsoft.Extensions.Logging;
using NotificationService.Application.Abstractions;
using NotificationService.Application.Messages;
using NotificationService.Application.Transport;

namespace NotificationService.Infrastructure.Transport.Consumers;

internal sealed class UserCreatedConsumer : IConsumer<UserCreated>
{
    private readonly IEncryptionService _encryptionService;
    private readonly ILogger<UserCreatedConsumer> _logger;
    private readonly IFluentEmail _fluentEmail;

    public UserCreatedConsumer(IEncryptionService encryptionService, ILogger<UserCreatedConsumer> logger, IFluentEmail fluentEmail)
    {
        _encryptionService = encryptionService;
        _logger = logger;
        _fluentEmail = fluentEmail;
    }

    public async Task ConsumeAsync(UserCreated message, CancellationToken cancellationToken)
    {
        string email = _encryptionService.Decrypt(message.Email, message.EmailIv);
        string otpCode = _encryptionService.Decrypt(message.Otp, message.OtpIv);

        var response = await _fluentEmail
            .To(email)
            .Subject("Email Confirmation Code")
            .UsingTemplateFromFile("EmailTemplates/EmailConfirmation.liquid", new{otpCode})
            .Tag("test")
            .SendAsync(cancellationToken);

        if (response.Successful is false)
        {
            _logger.LogError("Error while sending email confirmation: {}", string.Join(',',response.ErrorMessages));
        }
    }
}