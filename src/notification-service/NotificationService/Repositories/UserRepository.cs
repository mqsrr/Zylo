using Dapper;
using NotificationService.Factories.Abstractions;
using NotificationService.Helpers;
using NotificationService.Models;
using NotificationService.Models.Errors;
using NotificationService.Repositories.Abstractions;

namespace NotificationService.Repositories;

internal sealed class UserRepository : IUserRepository
{
    private readonly IDbConnectionFactory _dbConnectionFactory;

    public UserRepository(IDbConnectionFactory dbConnectionFactory)
    {
        _dbConnectionFactory = dbConnectionFactory;
    }

    public async Task<Result> CreateAsync(User user, CancellationToken cancellationToken)
    {
        await using var connection = await _dbConnectionFactory.CreateAsync(cancellationToken);
        int result = await connection.ExecuteAsync("INSERT INTO users (id, email, email_iv) VALUES ($Id, $Email, $EmailIv)", new
        {
            user.Id, user.Email, user.EmailIv
        });

        return result > 0
            ? Result.Success()
            : Result.Failure();
    }

    public async Task<Result> DeleteByIdAsync(UserId id, CancellationToken cancellationToken)
    {
        await using var connection = await _dbConnectionFactory.CreateAsync(cancellationToken);
        int result = await connection.ExecuteAsync("DELETE FROM users WHERE id = $Id", new {Id = id });

        return result > 0
            ? Result.Success()
            : new NotFoundError("User with given id not found");
    }
}