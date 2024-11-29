using Microsoft.AspNetCore.SignalR;
using NotificationService.Extensions;
using NotificationService.Factories.Abstractions;
using NotificationService.HostedServices;
using NotificationService.Hubs;
using NotificationService.Services.Abstractions;
using NotificationService.Settings;
using Serilog;

var builder = WebApplication.CreateBuilder(args);

builder.Host.UseSerilog((context, configuration) => 
    configuration.ReadFrom.Configuration(context.Configuration));

builder.Configuration.AddEnvFile();
builder.Configuration.AddAzureKeyVault();
builder.Configuration.AddJwtBearer(builder);

builder.Services.AddOptionsWithValidateOnStart<RabbitMqSettings>()
    .Bind(builder.Configuration.GetRequiredSection(RabbitMqSettings.SectionName));

builder.Services.AddOptionsWithValidateOnStart<EncryptionSettings>()
    .Bind(builder.Configuration.GetRequiredSection(EncryptionSettings.SectionName));

builder.Services.AddSignalR().AddAzureSignalR(builder.Configuration["SignalR:ConnectionString"]);

builder.Services.AddSingleton<IUserIdProvider, UserIdProvider>();

builder.Services.AddApplicationService<IRabbitMqConnectionFactory>(ServiceLifetime.Singleton);
builder.Services.AddApplicationService<IConsumer>(ServiceLifetime.Singleton);

builder.Services.AddApplicationService<IEncryptionService>(ServiceLifetime.Transient);
builder.Services.AddHostedService<RabbitMqConsumerHostedService>();


var app = builder.Build();

app.UseSerilogRequestLogging();
app.UseAuthentication();
app.UseAuthorization();

app.MapHub<NotificationHub>("/notifications");

app.Run();