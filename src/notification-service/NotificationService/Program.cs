using Asp.Versioning;
using Microsoft.AspNetCore.SignalR;
using NotificationService.Extensions;
using NotificationService.Factories;
using NotificationService.Factories.Abstractions;
using NotificationService.Hubs;
using NotificationService.Messages.User;
using NotificationService.Middleware;
using Serilog;

var builder = WebApplication.CreateBuilder(args);

builder.Configuration.AddEnvFile();
builder.Configuration.AddAzureKeyVault();
builder.Configuration.AddJwtBearer(builder);

builder.Host.UseSerilog((context, configuration) => 
    configuration.ReadFrom.Configuration(context.Configuration));

builder.ConfigureOpenTelemetry(builder.Configuration["OTEL:CollectorAddress"]!);
builder.Services.ConfigureJsonSerializer();

builder.Services.AddControllers();

builder.Services.AddApiVersioning(new HeaderApiVersionReader());
builder.Services.AddSignalR().AddAzureSignalR(builder.Configuration["SignalR:ConnectionString"]);
builder.Services.AddConnectionMultiplexer(builder.Configuration["Redis:ConnectionString"]!);


builder.Services.AddScoped<IDbConnectionFactory, PostgresDbConnectionFactory>();

builder.Services.AddSingleton<IUserIdProvider, UserIdProvider>();
builder.Services.AddApplicationSettings(builder.Configuration);

builder.Services.AddRabbitMqBus(mqBuilder =>
    mqBuilder
        .AddPublisher<UserCreated>("user-exchange", "user.created")
        .AddPublisher<UserDeleted>("user-exchange", "user.deleted")
        .AddPublisher<VerifyEmailAddress>("user-exchange", "user.verify.email"));


builder.Services.AddServiceHealthChecks(builder);
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
app.MapHealthChecks("/healthz").ExcludeFromDescription();

app.MapHub<NotificationHub>("/notifications");
app.UseCors();
app.Run();