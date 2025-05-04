using System.Data;
using Dapper;
using NotificationService.Models;

namespace NotificationService.TypeHandlers;

public class NotificationIdTypeHandler : SqlMapper.TypeHandler<NotificationId>
{
    public override NotificationId Parse(object value)
    {
        return new NotificationId(new Guid((byte[])value));
    }

    public override void SetValue(IDbDataParameter parameter, NotificationId id)
    {
        parameter.DbType = DbType.Binary;
        parameter.Size = 16;
        parameter.Value = id.Value.ToByteArray();
    }
}

public class UserIdTypeHandler : SqlMapper.TypeHandler<UserId>
{
    public override UserId Parse(object value)
    {
        return new UserId(new Guid((byte[])value));
    }

    public override void SetValue(IDbDataParameter parameter, UserId id)
    {
        parameter.DbType = DbType.Binary;
        parameter.Size = 16;
        parameter.Value = id.Value.ToByteArray();
    }
}