using UserManagement.Application.Contracts.Requests.Auth;
using UserManagement.Application.Models;
using UserManagement.Application.Services.Abstractions;

namespace UserManagement.Application.Mappers;

internal static class IdentityMapper
{
   internal static Identity ToIdentity(this RegisterRequest request, IHashService hashService)
   {
      (string emailHash, string emailSalt) = hashService.Hash(request.Email);
      (string passwordHash, string passwordSalt) = hashService.Hash(request.Password);
      
      return new Identity
      {
         Id = request.Id,
         Username = request.Username,
         EmailHash = emailHash,
         EmailSalt = emailSalt,
         EmailVerified = false,
         PasswordHash = passwordHash,
         PasswordSalt = passwordSalt
      };
   }
   
   public static RefreshTokenResponse ToResponse(this RefreshToken refreshToken)
   {
      return new RefreshTokenResponse
      {
         Value = Convert.ToBase64String(refreshToken.Token),
         ExpirationDate = refreshToken.ExpirationDate,
         Revoked = refreshToken.Revoked
      };
   }
}