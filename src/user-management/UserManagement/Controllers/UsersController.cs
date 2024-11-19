using Asp.Versioning;
using MassTransit.Mediator;
using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.Mvc;
using UserManagement.Application.Contracts.Requests.Users;
using UserManagement.Application.Helpers;
using UserManagement.Application.Mappers;
using UserManagement.Application.Models;
using UserManagement.Application.Repositories.Abstractions;

namespace UserManagement.Controllers;

[Authorize]
[ApiController]
[ApiVersion(1.0)]
public sealed class UsersController : ControllerBase
{
    private readonly IUserRepository _userRepository;

    public UsersController(IUserRepository userRepository)
    {
        _userRepository = userRepository;
    }

    [HttpGet(ApiEndpoints.Users.GetById)]
    public async Task<IActionResult> GetById([FromRoute] string id, CancellationToken cancellationToken)
    {
        var user = await _userRepository.GetByIdAsync(UserId.Parse(id), cancellationToken);
        return user is not null
            ? Ok(user.ToResponse())
            : NotFound();
    }
    
    [HttpPut(ApiEndpoints.Users.Update)]
    public async Task<IActionResult> Update([FromRoute] string id, [FromForm] UpdateUserRequest request, CancellationToken cancellationToken)
    {
        request.Id = UserId.Parse(id);
        bool isUpdated = await _userRepository.UpdateAsync(request, request.ProfileImage, request.BackgroundImage , cancellationToken);
        
        return isUpdated 
            ? NoContent()
            : BadRequest();
    }
    
    [HttpDelete(ApiEndpoints.Users.DeleteById)]
    public async Task<IActionResult> DeleteById([FromRoute] string id, CancellationToken cancellationToken)
    {
        bool isDeleted = await _userRepository.DeleteByIdAsync(UserId.Parse(id), cancellationToken);
        return isDeleted
            ? NoContent()
            : NotFound();
    }
}