using UserManagement.Application.Contracts.Requests.Auth;
using UserManagement.Application.Contracts.Responses.Auth;
using UserManagement.Application.Services.Common;
using UserManagement.Domain.Auth;

namespace UserManagement.Application.Mappers;

public static class IdentityMapper
{
   internal static Identity ToIdentity(this RegisterRequest request, IHashService hashService)
   {
      (string emailHash, string emailSalt) = hashService.Hash(request.Email);
      string uniqueEmailHash = hashService.CreateUniqueHash(request.Email);
      (string passwordHash, string passwordSalt) = hashService.Hash(request.Password);
      
      return new Identity
      {
         Id = request.Id,
         Username = request.Username,
         EmailHash = emailHash,
         EmailSalt = emailSalt,
         EmailUniqueHash = uniqueEmailHash,
         EmailVerified = false,
         PasswordHash = passwordHash,
         PasswordSalt = passwordSalt,
      };
   }

   public static RefreshTokenResponse ToResponse(this RefreshToken refreshToken)
   {
      return new RefreshTokenResponse
      {
         Value = Convert.ToBase64String(refreshToken.Token),
         ExpiresAt = refreshToken.ExpiresAt,
      };
   }

   public static AuthenticationResultResponse ToResponse(this AuthenticationResult result)
   {
      return new AuthenticationResultResponse
      {
         Id = result.Id.Value.ToString(),
         EmailVerified = result.EmailVerified,
         AccessToken = result.AccessToken,
      };
   }
}