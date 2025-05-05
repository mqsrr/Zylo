using Npgsql;

namespace UserManagement.Infrastructure.Extensions;

public static class PostgresExceptionExtensions 
{
    public static bool IsUniqueViolation(this PostgresException exception, string columnName)
        => exception.SqlState == PostgresErrorCodes.UniqueViolation && 
           (exception.ConstraintName?.Contains(columnName, StringComparison.OrdinalIgnoreCase) ?? false);

    public static bool IsForeignKeyViolation(this PostgresException exception, string tableName)
        => exception.SqlState == PostgresErrorCodes.ForeignKeyViolation && 
           (exception.TableName?.Contains(tableName, StringComparison.OrdinalIgnoreCase) ?? false);
}