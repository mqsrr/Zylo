
namespace NotificationService.Application.Transport;

public interface IBus
{
    Task StartAsync(CancellationToken cancellationToken);

    Task StopAsync(CancellationToken cancellationToken);
}