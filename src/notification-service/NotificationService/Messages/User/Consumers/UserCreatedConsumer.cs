using FluentEmail.Core;
using NotificationService.Mappers;
using NotificationService.Repositories.Abstractions;
using NotificationService.Services.Abstractions;

namespace NotificationService.Messages.User.Consumers;

internal sealed class UserCreatedConsumer : IConsumer<UserCreated>
{
    private readonly IEncryptionService _encryptionService;
    private readonly IUserRepository _userRepository;
    private readonly ILogger<UserCreatedConsumer> _logger;
    private readonly IFluentEmail _fluentEmail;

    public UserCreatedConsumer(IEncryptionService encryptionService, ILogger<UserCreatedConsumer> logger, IUserRepository userRepository, IFluentEmail fluentEmail)
    {
        _encryptionService = encryptionService;
        _logger = logger;
        _userRepository = userRepository;
        _fluentEmail = fluentEmail;
    }

    public async Task ConsumeAsync(UserCreated message, CancellationToken cancellationToken)
    {
        var creationResult = await _userRepository.CreateAsync(message.ToUser(), cancellationToken);
        if (creationResult.IsSuccess is false)
        {
            _logger.LogError("Error while creating user with {UserId} id", message.Id);
            return;
        }

        string email = _encryptionService.Decrypt(message.Email, message.EmailIv);
        string otpCode = _encryptionService.Decrypt(message.Otp, message.OtpIv);

        var response = await _fluentEmail
            .To(email)
            .Subject("Email Confirmation Code")
            .UsingTemplateFromFile("EmailTemplates/EmailConfirmation.liquid", otpCode)
            .Tag("test")
            .SendAsync(cancellationToken);

        if (response.Successful is false)
        {
            _logger.LogError("Error while sending email confirmation: {}", string.Join(',',response.ErrorMessages));
        }
    }
}