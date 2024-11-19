using FluentValidation;
using UserManagement.Application.Contracts.Requests.Auth;

namespace UserManagement.Application.Validators;

internal sealed class LoginRequestValidator : AbstractValidator<LoginRequest>
{
    public LoginRequestValidator()
    {
        RuleFor(request => request.Username)
            .NotEmpty()
            .MaximumLength(30);
        
        RuleFor(request => request.Password)
            .NotEmpty()
            .MaximumLength(30);
    }
}