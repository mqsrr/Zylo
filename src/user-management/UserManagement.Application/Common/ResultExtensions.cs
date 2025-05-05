using UserManagement.Domain.Errors;

namespace UserManagement.Application.Common;

public static class ResultExtensions
{
    public static TOut Match<TOut>(this Result result, Func<TOut> success, Func<Error, TOut> failure)
    {
        return result.IsSuccess 
            ? success()
            : failure(result.Error!);
    }

    public static TOut Match<TIn, TOut>(this Result<TIn> result, Func<TIn, TOut> success, Func<Error, TOut> failure)
    {
        return result.IsSuccess
            ? success(result.Value!) 
            : failure(result.Error!);
    }
}
