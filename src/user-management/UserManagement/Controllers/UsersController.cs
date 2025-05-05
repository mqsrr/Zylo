using Asp.Versioning;
using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.Mvc;
using UserManagement.Application.Common;
using UserManagement.Application.Contracts.Requests.Users;
using UserManagement.Application.Helpers;
using UserManagement.Application.Mappers;
using UserManagement.Application.Services.User;
using UserManagement.Domain.Users;
using UserManagement.Mappers;

namespace UserManagement.Controllers;

[Authorize]
[ApiController]
[ApiVersion(1.0)]
public sealed class UsersController : ControllerBase
{
    private readonly IUserService _userService;

    public UsersController(IUserService userService)
    {
        _userService = userService;
    }

    [HttpGet(ApiEndpoints.Users.GetById)]
    public async Task<IActionResult> GetById([FromRoute] string id, CancellationToken cancellationToken)
    {
        var user = await _userService.GetByIdAsync(UserId.Parse(id), cancellationToken);
        return user.Match<User, IActionResult>(
            success: u => Ok(u.ToResponse()),
            failure: e => e.ToProblemObjectResult(HttpContext.TraceIdentifier));
    }

    [HttpPut(ApiEndpoints.Users.Update)]
    public async Task<IActionResult> Update([FromRoute] string id, [FromForm] UpdateUserRequest request, CancellationToken cancellationToken)
    {
        var userUpdateResult = await _userService.UpdateAsync(request.ToUser(id), request.ProfileImage, request.BackgroundImage, cancellationToken);
        return userUpdateResult.Match<User,IActionResult>(
            success: u => Ok(u.ToResponse()),
            failure: e => e.ToProblemObjectResult(HttpContext.TraceIdentifier));
    }
}