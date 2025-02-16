using Grpc.Core;
using UserManagement.Application.Extensions;
using UserManagement.Application.Mappers;
using UserManagement.Application.Models;
using UserManagement.Application.Services.Abstractions;

namespace UserManagement.Application.Services;

internal sealed class UserProfileService : global::UserProfileService.UserProfileServiceBase
{
    private readonly IUserService _userService;

    public UserProfileService(IUserService userService)
    {
        _userService = userService;
    }

    public override async Task<GrpcUserResponse> GetUserById(GetUserByIdRequest request, ServerCallContext context)
    {
        var userResult = await _userService.GetByIdAsync(UserId.Parse(request.UserId), context.CancellationToken);
        return userResult.Match(
            success: u => u.ToGrpcResponse(),
            failure: e => throw new RpcException(e.ToGrpcStatus()));
    }

    public override async Task<BatchUsersResponse> GetBatchUsersByIds(GetBatchUsersByIdsRequest request, ServerCallContext context)
    {
        var usersResult = await _userService.GetBatchUsersByIdsAsync(request.UserIds.Select(UserId.Parse), context.CancellationToken);
        return usersResult.Match(
            success: u => new BatchUsersResponse
            {
                Users = { u.Select(user => user.ToGrpcResponse()) }
            },
            failure: e => throw new RpcException(e.ToGrpcStatus()));
    }
}