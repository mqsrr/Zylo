using NotificationService.Repositories.Abstractions;
using NotificationService.Services.Abstractions;

namespace NotificationService.Messages.User.Consumers;

internal sealed class UserDeletedConsumer : IConsumer<UserDeleted>
{
    private readonly IUserRepository _userRepository;
    private readonly ILogger<UserDeletedConsumer> _logger;

    public UserDeletedConsumer(IUserRepository userRepository, ILogger<UserDeletedConsumer> logger)
    {
        _userRepository = userRepository;
        _logger = logger;
    }

    public async Task ConsumeAsync(UserDeleted message, CancellationToken cancellationToken)
    {
        var deletionResult = await _userRepository.DeleteByIdAsync(message.Id, cancellationToken);
        if (deletionResult.IsSuccess is false)
        {
            _logger.LogError("Error while deleting user with id {Id} {Error}", message.Id, deletionResult.Error);
        }
    }
}