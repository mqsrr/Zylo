using FluentValidation;
using UserManagement.Application.Contracts.Requests.Auth;

namespace UserManagement.Application.Validators;

internal sealed class RegisterRequestValidator : AbstractValidator<RegisterRequest> 
{
    public RegisterRequestValidator()
    {
        RuleFor(request => request.Username)
            .NotEmpty()
            .MaximumLength(30);
        
        RuleFor(request => request.Password)
            .NotEmpty()
            .MaximumLength(30);
        
        RuleFor(request => request.Email)
            .NotEmpty()
            .MaximumLength(50);

        RuleFor(request => request.ProfileImage)
            .NotEmpty();
        
        RuleFor(request => request.BackgroundImage)
            .NotEmpty();
        
        RuleFor(request => request.Name)
            .NotEmpty()
            .MaximumLength(30);

        RuleFor(request => request.Bio)
            .MaximumLength(500);
        
        RuleFor(request => request.Location)
            .MaximumLength(100);
        
        RuleFor(request => request.BirthDate)
            .InclusiveBetween(DateOnly.FromDateTime(DateTime.Now.AddYears(-70)), DateOnly.FromDateTime(DateTime.Now));
    }
}