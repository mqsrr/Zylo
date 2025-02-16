using Npgsql;

namespace UserManagement.Application.Extensions;

public static class PostgresExceptionExtensions 
{
    public static bool IsUniqueViolation(this PostgresException exception, string columnName)
        => exception.SqlState == PostgresErrorCodes.UniqueViolation && 
           (exception.ConstraintName?.Contains(columnName, StringComparison.OrdinalIgnoreCase) ?? false);

    public static bool IsForeignKeyViolation(this PostgresException exception, string tableName)
        => exception.SqlState == PostgresErrorCodes.ForeignKeyViolation && 
           (exception.TableName?.Contains(tableName, StringComparison.OrdinalIgnoreCase) ?? false);

    public static bool IsAnyUniqueViolation(this PostgresException exception, params string[] columnNames)
        => exception.SqlState == PostgresErrorCodes.UniqueViolation && 
           columnNames.Any(column => exception.ConstraintName?.Contains(column, StringComparison.OrdinalIgnoreCase) ?? false);

    public static bool IsAnyForeignKeyViolation(this PostgresException exception, params string[] tableNames)
        => exception.SqlState == PostgresErrorCodes.ForeignKeyViolation && 
           tableNames.Any(table => exception.TableName?.Contains(table, StringComparison.OrdinalIgnoreCase) ?? false);
}