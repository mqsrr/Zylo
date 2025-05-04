using Dapper;
using NotificationService.Factories.Abstractions;
using NotificationService.Helpers;
using NotificationService.Models;
using NotificationService.Repositories.Abstractions;

namespace NotificationService.Repositories;

internal sealed class NotificationRepository : INotificationRepository
{
    private readonly IDbConnectionFactory _dbConnectionFactory;

    public NotificationRepository(IDbConnectionFactory dbConnectionFactory)
    {
        _dbConnectionFactory = dbConnectionFactory;
    }

    public async Task<Result<IEnumerable<Notification>>> GetAllAsync(UserId userId, CancellationToken cancellationToken)
    {
        await using var connection = await _dbConnectionFactory.CreateAsync(cancellationToken);
        var notifications = await connection.QueryAsync<Notification>("SELECT * FROM notifications WHERE user_id = @UserId", new { UserId = userId });

        return Result.Success(notifications);
    }

    public async Task<Result> CreateAsync(Notification notification, CancellationToken cancellationToken)
    {
        await using var connection = await _dbConnectionFactory.CreateAsync(cancellationToken);
        int result = await connection.ExecuteAsync("INSERT INTO notifications (id, user_id, message, is_seen) VALUES ($Id, $UserId, $Message, $IsSeen)", notification);

        return result > 0
            ? Result.Success()
            : Result.Failure();
    }

    public async Task<Result> UpdateSeenAsync(IEnumerable<NotificationId> notificationIds, bool isSeen, CancellationToken cancellationToken)
    {
        await using var connection = await _dbConnectionFactory.CreateAsync(cancellationToken);
        int result = await connection.ExecuteAsync("UPDATE notifications SET is_seen = @IsSeen WHERE id = ANY(@Ids)",
            new { Ids = notificationIds.ToArray(), IsSeen = isSeen });

        return result > 0
            ? Result.Success()
            : Result.Failure();
    }

    public async Task<Result> DeleteByIdsAsync(IEnumerable<NotificationId> notificationIds, CancellationToken cancellationToken)
    {
        await using var connection = await _dbConnectionFactory.CreateAsync(cancellationToken);
        int result = await connection.ExecuteAsync("DELETE FROM notifications WHERE id = ANY(@Ids)", notificationIds.ToArray());

        return result > 0
            ? Result.Success()
            : Result.Failure();
    }
}