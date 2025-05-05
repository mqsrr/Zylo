using Asp.Versioning;
using NotificationService.Extensions;
using NotificationService.Infrastructure.Extensions;
using NotificationService.Infrastructure.Hubs;
using NotificationService.Middleware;
using Serilog;

var builder = WebApplication.CreateBuilder(args);

builder.Configuration.AddEnvFile();
builder.Configuration.AddAzureKeyVault();
builder.Configuration.AddJwtBearer(builder);

builder.Host.UseSerilog((context, configuration) =>
    configuration.ReadFrom.Configuration(context.Configuration));

builder.Services.ConfigureOpenTelemetry(builder.Configuration["OTEL:CollectorAddress"]!, builder.Logging);
builder.Services.ConfigureJsonSerializer();

builder.Services.AddControllers();
builder.Services.AddApiVersioning(new HeaderApiVersionReader());
builder.Services.AddInfrastructure(builder.Configuration);

builder.Services.RegisterRabbitMqBusConsumers();
builder.Services.AddCors(options =>
    options.AddDefaultPolicy(policyBuilder =>
        policyBuilder.AllowAnyHeader()
            .AllowAnyMethod()
            .WithOrigins("http://localhost:5173")
            .AllowCredentials()));

var app = builder.Build();

app.UseSerilogRequestLogging();
app.UseMiddleware<RequestIdMiddleware>();

app.UseAuthentication();
app.UseAuthorization();

app.MapControllers();
app.MapHub<NotificationHub>("/notifications");

app.UseCors();
app.Run();