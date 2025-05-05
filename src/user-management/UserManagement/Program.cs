using Asp.Versioning;
using FluentValidation;
using FluentValidation.AspNetCore;
using Microsoft.AspNetCore.Server.Kestrel.Core;
using Serilog;
using UserManagement.Application.Extensions;
using UserManagement.Application.Middleware;
using UserManagement.Application.Validators;
using UserManagement.Infrastructure.Extensions;

var builder = WebApplication.CreateBuilder(args);
builder.Configuration.AddEnvFile();
builder.Configuration.AddAzureKeyVault();
builder.Configuration.AddJwtBearer(builder);

builder.Host.UseSerilog((context, configuration) => configuration.ReadFrom.Configuration(context.Configuration));
builder.Services.ConfigureOpenTelemetry(builder.Configuration["OTEL:CollectorAddress"]!, builder.Logging);

builder.Services.ConfigureJsonSerializer();
builder.Services.AddControllers();

builder.Services.AddApiVersioning(new HeaderApiVersionReader());
builder.Services.AddFluentValidationAutoValidation()
    .AddValidatorsFromAssemblyContaining<RegisterRequestValidator>(includeInternalTypes: true);

builder.Services.AddTransient<MetricsMiddleware>();
builder.Services.AddTransient<RequestIdMiddleware>();

builder.Services.AddInfrastructure(builder.Configuration);
builder.Services.RegisterRabbitMqPublishers();

builder.Services.AddCors(options =>
    options.AddDefaultPolicy(policyBuilder =>
        policyBuilder.AllowAnyHeader()
            .AllowAnyMethod()
            .WithOrigins("http://localhost:5173")
            .AllowCredentials()));

builder.WebHost.ConfigureKestrel(options =>
{
    options.ListenAnyIP(50051, listenOptions => { listenOptions.Protocols = HttpProtocols.Http2; });
    options.ListenAnyIP(8080, listenOptions => { listenOptions.Protocols = HttpProtocols.Http1; });
});

var app = builder.Build();

app.UseSerilogRequestLogging();

app.UseMiddleware<RequestIdMiddleware>();
app.UseMiddleware<MetricsMiddleware>();

app.UseAuthentication();
app.UseAuthorization();
app.MapControllers();

app.MapHealthChecks("/healthz").ExcludeFromDescription();

app.MapGrpcService<UserManagement.Infrastructure.Services.Users.UserProfileService>();
app.UseCors();
app.Run();