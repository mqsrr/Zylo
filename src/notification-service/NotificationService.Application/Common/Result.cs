using NotificationService.Domain.Errors;

namespace NotificationService.Application.Common;

public class Result(bool isSuccess, Error? error)
{
    public bool IsSuccess { get; } = isSuccess;
    public Error? Error { get;} = error;

    public static Result Success()
    {
        return new Result(true, null);
    }

    public static Result Failure(Error? error = null)
    {
        return new Result(false, error);
    }

    public static Result<TValue> Success<TValue>(TValue value)
    {
        return new Result<TValue>(value, true, null);
    }

    protected static Result<TValue> Failure<TValue>(Error? error)
    {
        return new Result<TValue>(default, false, error);
    }

    public static implicit operator Result(Error? error)
    {
        return Failure(error);
    }
}

public sealed class Result<TValue>(TValue? value, bool isSuccess, Error? message) : Result(isSuccess, message)
{
    public TValue? Value { get; } = value;

    public static implicit operator Result<TValue>(TValue value)
    {
        return Success(value);
    }

    public static implicit operator Result<TValue>(Error? error)
    {
        return Failure<TValue>(error);
    }
}