using Grpc.Core;
using UserManagement.Application.Common;
using UserManagement.Application.Services.User;
using UserManagement.Domain.Users;
using UserManagement.Infrastructure.Mappers;
using UserProfileService;

namespace UserManagement.Infrastructure.Services.Users;

public sealed class UserProfileService : global::UserProfileService.UserProfileService.UserProfileServiceBase
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

    public override async Task<BatchUsersSummaryResponse> GetBatchUsersSummaryByIds(GetBatchUsersByIdsRequest request, ServerCallContext context)
    {
        var usersResult = await _userService.GetBatchUsersSummaryByIdsAsync(request.UserIds.Select(UserId.Parse), context.CancellationToken);
        return usersResult.Match(
            success: u => new BatchUsersSummaryResponse
            {
                Users = { u.Select(user => user.ToGrpcResponse()) }
            },
            failure: e => throw new RpcException(e.ToGrpcStatus()));
    }
}