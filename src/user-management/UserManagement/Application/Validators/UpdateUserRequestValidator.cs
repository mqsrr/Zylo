using FluentValidation;
using UserManagement.Application.Contracts.Requests.Users;

namespace UserManagement.Application.Validators;

internal sealed class UpdateUserRequestValidator : AbstractValidator<UpdateUserRequest>
{
    public UpdateUserRequestValidator()
    {
        RuleFor(request => request.ProfileImage)
            .NotEmpty();
        
        RuleFor(request => request.BackgroundImage)
            .NotEmpty();

        RuleFor(request => request.Name)
            .NotEmpty()
            .MaximumLength(30);
        
        RuleFor(request => request.Bio)
            .NotEmpty()
            .MaximumLength(500);
        
        RuleFor(request => request.Location)
            .NotEmpty()
            .MaximumLength(100);
        
        RuleFor(request => request.BirthDate)
            .InclusiveBetween(DateOnly.FromDateTime(DateTime.Now.AddYears(-70)), DateOnly.FromDateTime(DateTime.Now));
    }
}