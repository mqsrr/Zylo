using Asp.Versioning;
using Microsoft.AspNetCore.Authorization;
using Microsoft.AspNetCore.Mvc;
using NotificationService.Contracts;
using NotificationService.Extensions;
using NotificationService.Helpers;
using NotificationService.Mappers;
using NotificationService.Models;
using NotificationService.Repositories.Abstractions;

namespace NotificationService.Controllers;

[ApiController]
[Authorize]
[ApiVersion(1.0)]
public sealed class NotificationsController : ControllerBase
{
    private readonly INotificationRepository _notificationRepository;

    public NotificationsController(INotificationRepository notificationRepository)
    {
        _notificationRepository = notificationRepository;
    }

    [HttpGet(ApiEndpoints.Notifications.GetAll)]
    public async Task<IActionResult> GetAll(UserId id, CancellationToken cancellationToken)
    {
        var notificationsResult = await _notificationRepository.GetAllAsync(id, cancellationToken);
        return notificationsResult.Match<IEnumerable<Notification>, IActionResult>(
            success: Ok,
            failure: error => error.ToProblemObjectResult(HttpContext.TraceIdentifier));
    }

    [HttpPut(ApiEndpoints.Notifications.UpdateManySeen)]
    public async Task<IActionResult> UpdateManySeen([FromBody] UpdateManySeenRequest request, CancellationToken cancellationToken)
    {
        var updateResult = await _notificationRepository.UpdateSeenAsync(request.NotificationsIds, request.IsSeen, cancellationToken);
        return updateResult.Match<IActionResult>(
            success: NoContent,
            failure: error => error.ToProblemObjectResult(HttpContext.TraceIdentifier));
    }

    [HttpDelete(ApiEndpoints.Notifications.DeleteManyById)]
    public async Task<IActionResult> DeleteManyById([FromBody] DeleteManyByIdRequest request, CancellationToken cancellationToken)
    {
        var updateResult = await _notificationRepository.DeleteByIdsAsync(request.NotificationsIds, cancellationToken);
        return updateResult.Match<IActionResult>(
            success: NoContent,
            failure: error => error.ToProblemObjectResult(HttpContext.TraceIdentifier));
    }
}