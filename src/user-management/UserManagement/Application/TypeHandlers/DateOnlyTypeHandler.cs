﻿using System.Data;
using Dapper;

namespace UserManagement.Application.TypeHandlers;

public partial class DateOnlyTypeHandler : SqlMapper.TypeHandler<DateOnly> 
{
    public override DateOnly Parse(object value)
    {
        return DateOnly.FromDateTime((DateTime)value);
    }

    public override void SetValue(IDbDataParameter parameter, DateOnly value)
    {
        parameter.DbType = DbType.Date;
        parameter.Value = value.ToDateTime(TimeOnly.MinValue);
    }
}