using System.Data;
using Dapper;
using MassTransit;
using Mediator;
using Npgsql;
using UserManagement.Application.Factories.Abstractions;
using UserManagement.Application.Helpers;
using UserManagement.Application.Services.Abstractions;

namespace UserManagement.Application.Messaging.Users.Handlers;

internal sealed class CreateOtpOnUserCreatedNotificationHandler : INotificationHandler<CreateUserNotification>
{
    private readonly IOtpService _otpService;
    private readonly IEncryptionService _encryptionService;
    private readonly IHashService _hashService;
    private readonly IDbConnectionFactory _connectionFactory;
    private readonly IPublishEndpoint _publisher;

    public CreateOtpOnUserCreatedNotificationHandler(
        IPublishEndpoint publisher,
        IDbConnectionFactory connectionFactory,
        IOtpService otpService,
        IEncryptionService encryptionService,
        IHashService hashService)
    {
        _publisher = publisher;
        _connectionFactory = connectionFactory;
        _otpService = otpService;
        _encryptionService = encryptionService;
        _hashService = hashService;
    }

    public async ValueTask Handle(CreateUserNotification notification, CancellationToken cancellationToken)
    {
        await using var connection = await _connectionFactory.CreateAsync(cancellationToken);
        await using var transaction = await connection.BeginTransactionAsync(IsolationLevel.ReadCommitted, cancellationToken);
        try
        {
            string code = _otpService.CreateOneTimePassword(6);
            (string hashedCode, string salt) = _hashService.Hash(code);

            int result = await connection.ExecuteAsync(SqlQueries.Authentication.CreateOtpCode, new
            {
                notification.Request.Id,
                CodeHash = hashedCode,
                Salt = salt,
                ExpiresAt = DateTime.UtcNow.AddMonths(1)
            });
            if (result < 1)
            {
                return;
            }
            
            await transaction.CommitAsync(cancellationToken);
            
            (string encryptedCode, string otpIv) = _encryptionService.Encrypt(code);
            (string encryptedEmail, string emailIv) = _encryptionService.Encrypt(notification.Request.Email);
            await _publisher.Publish<VerifyEmailAddress>(new
            {
                Otp = encryptedCode,
                OtpIv = otpIv,
                Email = encryptedEmail,
                EmailIv = emailIv
            }, context => context.SetRoutingKey("user.verify.email"), cancellationToken);
        }
        catch(PostgresException)
        {
            await transaction.RollbackAsync(cancellationToken);
            throw;
        }
    }
}