using UserManagement.Application.Common;
using UserManagement.Application.Repositories.Auth;
using UserManagement.Application.Services.Auth;
using UserManagement.Domain.Auth;
using UserManagement.Domain.Errors;

namespace UserManagement.Infrastructure.Services.Auth;

public sealed class TokenService : ITokenService
{
    private readonly IRefreshTokenRepository _refreshTokenRepository;
    private readonly ITokenWriter _tokenWriter;

    public TokenService(ITokenWriter tokenWriter, IRefreshTokenRepository refreshTokenRepository)
    {
        _tokenWriter = tokenWriter;
        _refreshTokenRepository = refreshTokenRepository;
    }

    public AccessToken GenerateToken(Identity identity)
    {
        return _tokenWriter.GenerateAccessToken(identity);
    }

    public Task<Result<RefreshToken>> GetRefreshTokenByIdentityIdAsync(IdentityId id, CancellationToken cancellationToken)
    {
        return _refreshTokenRepository.GetByIdentityIdAsync(id, cancellationToken);
    }

    public Task<Result<RefreshToken>> GetRefreshTokenAsync(string refreshToken, CancellationToken cancellationToken)
    {
        byte[]? tokenBytes = _tokenWriter.ParseRefreshToken(refreshToken);
        return tokenBytes is not null
            ? _refreshTokenRepository.GetRefreshTokenAsync(tokenBytes, cancellationToken)
            : Task.FromResult<Result<RefreshToken>>(new BadRequestError("Invalid refresh token"));
    }

    public async Task<Result<RefreshToken>> CreateRefreshTokenAsync(IdentityId id, CancellationToken cancellationToken)
    {
        var refreshToken = _tokenWriter.GenerateRefreshToken(id);
        var result = await _refreshTokenRepository.CreateAsync(refreshToken, cancellationToken);

        return result.IsSuccess
            ? refreshToken
            : result.Error;
    }

    public Task<Result> DeleteRefreshTokenAsync(string refreshToken, CancellationToken cancellationToken)
    {
        byte[]? tokenBytes = _tokenWriter.ParseRefreshToken(refreshToken);
        return tokenBytes is not null
            ? _refreshTokenRepository.DeleteAsync(tokenBytes, cancellationToken)
            : Task.FromResult<Result>(new BadRequestError("Invalid refresh token"));
    }

    public Task<Result> DeleteRefreshTokenByIdentityIdAsync(IdentityId id, CancellationToken cancellationToken)
    {
        return _refreshTokenRepository.DeleteAllByIdAsync(id,  cancellationToken);
    }
}