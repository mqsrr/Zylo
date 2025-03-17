using NotificationService.Settings;

namespace NotificationService.Builders;

public sealed class RabbitMqBuilder
{
    private readonly List<PublisherSettings> _publisherSettings = [];

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

    internal List<PublisherSettings> Build() => _publisherSettings;
}