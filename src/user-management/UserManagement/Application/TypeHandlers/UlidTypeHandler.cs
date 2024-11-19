using System.Data;
using Dapper;
using UserManagement.Application.Models;

namespace UserManagement.Application.TypeHandlers;

public class IdentityIdTypeHandler : SqlMapper.TypeHandler<IdentityId>
{

    public override IdentityId Parse(object value)
    {
        return new IdentityId(new Ulid((byte[])value));
    }

    public override void SetValue(IDbDataParameter parameter, IdentityId id)
    {
        parameter.DbType = DbType.Binary;
        parameter.Size = 16;
        parameter.Value = id.Value.ToByteArray();
    }
}

public class UlidTypeHandler : SqlMapper.TypeHandler<Ulid>
{
    public override Ulid Parse(object value)
    {
        return new Ulid((byte[])value);
    }

    public override void SetValue(IDbDataParameter parameter, Ulid value)
    {
        parameter.DbType = DbType.Binary;
        parameter.Size = 16;
        parameter.Value = value.ToByteArray();    }

}

public class UserIdTypeHandler : SqlMapper.TypeHandler<UserId>
{
    public override UserId Parse(object value)
    {
        return new UserId(new Ulid((byte[])value));
    }

    public override void SetValue(IDbDataParameter parameter, UserId id)
    {
        parameter.DbType = DbType.Binary;
        parameter.Size = 16;
        parameter.Value = id.Value.ToByteArray();
    }
}