using NotificationService.Services.Abstractions;
using NotificationService.Settings;

namespace NotificationService.Builders;

public sealed class RabbitMqBuilder
{
    private readonly List<PublisherSettings> _publisherSettings = [];
    private readonly List<ConsumerSettings> _consumerSettings = [];

    public RabbitMqBuilder AddPublisher<TEntity>(string exchangeName, string routingKey) where TEntity : class
    {
        _publisherSettings.Add(new PublisherSettings
        {
            ExchangeName = exchangeName,
            RoutingKey = routingKey,
            AttachedType = typeof(TEntity)
        });

        return this;
    }

    public RabbitMqBuilder AddConsumer<TMessage, TConsumer>(
        string exchangeName,
        string queueName,
        string routingKey) where TMessage : class where TConsumer : IConsumer<TMessage>
    {
        _consumerSettings.Add(new ConsumerSettings
        {
            MessageType = typeof(TMessage),
            ExchangeName = exchangeName,
            QueueName = queueName,
            RoutingKey = routingKey,
            ConsumerType = typeof(TConsumer),
        });

        return this;
    }

    public RabbitMqBusSettings Build()
    {
        return new RabbitMqBusSettings
        {
            Publishers = _publisherSettings,
            Consumers = _consumerSettings
        };
    }
}