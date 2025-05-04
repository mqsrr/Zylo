using Asp.Versioning;
using Microsoft.AspNetCore.SignalR;
using Microsoft.Extensions.FileProviders;
using NotificationService.Extensions;
using NotificationService.Factories;
using NotificationService.Factories.Abstractions;
using NotificationService.Hubs;
using NotificationService.Messages.Block;
using NotificationService.Messages.Block.Consumers;
using NotificationService.Messages.Follower;
using NotificationService.Messages.Follower.Consumers;
using NotificationService.Messages.Friend;
using NotificationService.Messages.Friend.Consumers;
using NotificationService.Messages.User;
using NotificationService.Messages.User.Consumers;
using NotificationService.Middleware;
using NotificationService.Repositories;
using NotificationService.Repositories.Abstractions;
using NotificationService.Services;
using NotificationService.Services.Abstractions;
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
builder.Services.AddScoped<IEncryptionService, EncryptionService>();
builder.Services.AddScoped<INotificationRepository, NotificationRepository>();
builder.Services.AddScoped<IUserRepository, UserRepository>();

builder.Services.AddSingleton<IUserIdProvider, UserIdProvider>();
builder.Services.AddApplicationSettings(builder.Configuration);

builder.Services.AddFluentEmail(builder.Configuration["Mailgun:Sender"])
    .AddMailGunSender(builder.Configuration["MailGun:Domain"], builder.Configuration["Mailgun:ApiKey"])
    .AddLiquidRenderer(options => options.FileProvider = new PhysicalFileProvider(Directory.GetCurrentDirectory()));

builder.Services.AddRabbitMqBus(mqBuilder =>
    mqBuilder
        .AddConsumer<UserCreated, UserCreatedConsumer>("user-exchange", "user-created-notification-service", "user.created")
        .AddConsumer<UserDeleted, UserDeletedConsumer>("user-exchange", "user-deleted-notification-service", "user.deleted")
        .AddConsumer<UserAcceptedFriendRequest, UserAcceptedFriendRequestConsumer>("user-exchange", "user-accepted-friend-request-service", "user.add.friend")
        .AddConsumer<UserSentFriendRequest, UserSentFriendRequestConsumer>("user-exchange", "user-sent-friend-request-notification-service", "user.sent.friend")
        .AddConsumer<UserRemovedFriend, UserRemovedFriendConsumer>("user-exchange", "user-friend-removed-notification-service", "user.friends.remove")
        .AddConsumer<UserDeclinedFriendRequest, UserDeclinedFriendRequestConsumer>("user-exchange", "user-declined-friend-request-notification-service", "user.remove.friend")
        .AddConsumer<UserFollowed, UserFollowedConsumer>("user-exchange", "user-followed-notification-service", "user.followed")
        .AddConsumer<UserUnfollowed, UserUnfollowedConsumer>("user-exchange", "user-unfollowed-notification-service", "user.unfollowed")
        .AddConsumer<UserBlocked, UserBlockedConsumer>("user-exchange", "user-blocked-notification-service", "user.blocked")
        .AddConsumer<UserUnblocked, UserUnblockedConsumer>("user-exchange", "user-unblocked-notification-service", "user.unblocked"));

builder.Services.AddServiceHealthChecks(builder);
builder.Services.AddCors(options =>
    options.AddDefaultPolicy(policyBuilder =>
        policyBuilder.AllowAnyHeader()
            .AllowAnyMethod()
            .WithOrigins("http://localhost:5173")
            .AllowCredentials()));

var app = builder.Build();

app.UseTypeHandlers();
app.UseSerilogRequestLogging();

app.UseMiddleware<RequestIdMiddleware>();
app.UseAuthentication();
app.UseAuthorization();

app.MapControllers();
app.MapHealthChecks("/healthz").ExcludeFromDescription();

app.MapHub<NotificationHub>("/notifications");
app.UseCors();
app.Run();