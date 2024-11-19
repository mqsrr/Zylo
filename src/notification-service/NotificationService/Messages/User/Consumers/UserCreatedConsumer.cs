using System.Text;
using Newtonsoft.Json;
using NotificationService.Factories.Abstractions;
using NotificationService.Services.Abstractions;
using RabbitMQ.Client;
using RabbitMQ.Client.Events;

namespace NotificationService.Messages.User.Consumers;

internal sealed class UserCreatedConsumer : IConsumer
{
    private readonly IRabbitMqConnectionFactory _connectionFactory;
    private readonly IEncryptionService _encryptionService;
    private readonly ILogger<UserCreatedConsumer> _logger;
    private IChannel? _channel;

    public UserCreatedConsumer(IRabbitMqConnectionFactory factory, IEncryptionService encryptionService, ILogger<UserCreatedConsumer> logger)
    {
        _connectionFactory = factory;
        _encryptionService = encryptionService;
        _logger = logger;
    }
    
    public async Task ConsumeAsync(CancellationToken cancellationToken)
    {
        var connection = await _connectionFactory.GetConnectionAsync(cancellationToken);
        _channel = await  connection.CreateChannelAsync(null, cancellationToken);

        await _channel.ExchangeDeclareAsync("user-exchange", "direct", true, false, cancellationToken: cancellationToken);
        var queueDeclare= await _channel.QueueDeclareAsync("user-verify-email-notification-service", true, false, false, cancellationToken: cancellationToken);
        
        await _channel.QueueBindAsync(queueDeclare.QueueName, "user-exchange", "user.verify.email", cancellationToken: cancellationToken);
        
        var consumer = new AsyncEventingBasicConsumer(_channel);
        consumer.ReceivedAsync += async (model, ea) =>
        {
            try
            {
                var userCreated = JsonConvert.DeserializeObject<UserCreated>(Encoding.UTF8.GetString(ea.Body.ToArray()));
                if (userCreated is null)
                {
                    _logger.LogError("The message was not received");
                    await _channel.BasicNackAsync(ea.DeliveryTag, false, false, cancellationToken);
                    return;
                }

                string email = _encryptionService.Decrypt(userCreated.Email, userCreated.EmailIv);
                string otpCode = _encryptionService.Decrypt(userCreated.Otp, userCreated.OtpIv);

                _logger.LogInformation("Email:{Email}, Code:{OtpCode}", email, otpCode);
                await _channel.BasicAckAsync(ea.DeliveryTag, false, ea.CancellationToken);
            }
            catch (Exception ex)
            {
                await _channel.BasicNackAsync(ea.DeliveryTag, false, false, cancellationToken);
            }
           
        };

        await _channel.BasicConsumeAsync(queueDeclare.QueueName, false, consumer, cancellationToken);
    }

    public async ValueTask DisposeAsync()
    {
        if (_channel is not null)
        {
            await _channel.DisposeAsync();
        }

        await _connectionFactory.DisposeAsync();
    }
}