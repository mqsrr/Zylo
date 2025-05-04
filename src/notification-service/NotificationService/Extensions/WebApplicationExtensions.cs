using Dapper;
using NotificationService.TypeHandlers;

namespace NotificationService.Extensions;

internal static class WebApplicationExtensions
{
    public static WebApplication UseTypeHandlers(this WebApplication app)
    {
        SqlMapper.AddTypeHandler(new NotificationIdTypeHandler());
        SqlMapper.AddTypeHandler(new UserIdTypeHandler());

        return app;
    }
}