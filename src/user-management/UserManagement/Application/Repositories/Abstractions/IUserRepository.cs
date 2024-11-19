using UserManagement.Application.Contracts.Requests.Users;
using UserManagement.Application.Models;

namespace UserManagement.Application.Repositories.Abstractions;

public interface IUserRepository
{
    Task<User?> GetByIdAsync(UserId id, CancellationToken cancellationToken);
    
    Task<bool> CreateAsync(User user, IFormFile profileImage, IFormFile backgroundImage, CancellationToken cancellationToken);
    
    Task<bool> UpdateAsync(UpdateUserRequest updatedUser, IFormFile profileImage, IFormFile backgroundImage,  CancellationToken cancellationToken);
    
    Task<bool> DeleteByIdAsync(UserId id, CancellationToken cancellationToken);
}