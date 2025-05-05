using System.Collections.Concurrent;
using System.Reflection;
using Microsoft.AspNetCore.SignalR;
using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.FileProviders;
using Newtonsoft.Json;
using Newtonsoft.Json.Serialization;
using NotificationService.Application.Abstractions;
using NotificationService.Application.Builders;
using NotificationService.Application.Converters;
using NotificationService.Application.Messages;
using NotificationService.Application.Settings;
using NotificationService.Application.Transport;
using NotificationService.Infrastructure.Hubs;
using NotificationService.Infrastructure.Services;
using NotificationService.Infrastructure.Transport.Bus;
using NotificationService.Infrastructure.Transport.Consumers;
using NotificationService.Infrastructure.Transport.Factories;
using NotificationService.Infrastructure.Transport.HostedServices;
using RabbitMQ.Client;

namespace NotificationService.Infrastructure.Extensions;

public static class ServiceCollectionExtensions
{
    private static IServiceCollection AddOptionsSettingsWithValidation<TOptions>(
        this IServiceCollection services,
        IConfiguration configuration)
        where TOptions : BaseSettings
    {
        if (Activator.CreateInstance<TOptions>() is not BaseSettings instance)
        {
            throw new InvalidOperationException($"Could not create instance of {typeof(TOptions).Name}");
        }

        return services
            .AddOptionsWithValidateOnStart<TOptions>()
            .Bind(configuration.GetRequiredSection(instance.SectionName))
            .Services;
    }

    private static IServiceCollection AddApplicationSettings(
        this IServiceCollection services,
        IConfiguration configuration)
    {
        var baseSettingsType = typeof(BaseSettings);
        var allSettings = Assembly.GetAssembly(baseSettingsType)!
            .ExportedTypes
            .Where(t => t is { IsInterface: false, IsAbstract: false } && t.BaseType == baseSettingsType);

        foreach (var settings in allSettings)
        {
            var method = typeof(ServiceCollectionExtensions)
                .GetMethod(nameof(AddOptionsSettingsWithValidation), BindingFlags.Static | BindingFlags.NonPublic)!
                .MakeGenericMethod(settings);

            method.Invoke(null, [services, configuration]);
        }

        return services;
    }

    private static IServiceCollection AddRabbitMqBus(
        this IServiceCollection services,
        Action<RabbitMqBuilder> configure)
    {
        services.AddSingleton<ConcurrentDictionary<Type, IChannel>>();
        services.AddSingleton<IRabbitMqConnectionFactory, RabbitMqConnectionFactory>();
        services.AddSingleton<IBus, RabbitMqBus>();

        services.AddOptions<RabbitMqBusSettings>()
            .Configure<IServiceProvider>((settings, _) =>
            {
                var builder = new RabbitMqBuilder();
                configure(builder);

                var busSettings = builder.Build();
                settings.Publishers = busSettings.Publishers;
                settings.Consumers = busSettings.Consumers;
            });

        var consumerType = typeof(IConsumer<>);
        var consumerMap = Assembly.GetExecutingAssembly()
            .GetTypes()
            .Where(t => t is { IsInterface: false, IsAbstract: false })
            .SelectMany(t => t.GetInterfaces()
                .Where(i => i.IsGenericType && i.GetGenericTypeDefinition() == consumerType)
                .Select(i => new { Interface = i, Implementation = t }))
            .ToDictionary(x => x.Interface, x => x.Implementation);

        foreach (var (interfaceType, implementationType) in consumerMap)
        {
            services.AddScoped(interfaceType, implementationType);
        }

        services.AddHostedService<RabbitMqBusHostedService>();
        return services;
    }

    public static IServiceCollection ConfigureJsonSerializer(this IServiceCollection services)
    {
        JsonConvert.DefaultSettings = () => new JsonSerializerSettings
        {
            Formatting = Formatting.Indented,
            TypeNameHandling = TypeNameHandling.None,
            ContractResolver = new CamelCasePropertyNamesContractResolver(),
            Converters = [new UserIdConverter()]
        };

        return services;
    }

    public static IServiceCollection AddInfrastructure(this IServiceCollection services, IConfiguration configuration)
    {
        services.AddSingleton<IUserIdProvider, UserIdProvider>();
        services.AddSignalR()
            .AddAzureSignalR(configuration["SignalR:ConnectionString"]);

        services.AddApplicationSettings(configuration);

        services.AddFluentEmail(configuration["Mailgun:Sender"])
            .AddMailGunSender(configuration["MailGun:Domain"], configuration["Mailgun:ApiKey"])
            .AddLiquidRenderer(options => options.FileProvider = new PhysicalFileProvider(Directory.GetCurrentDirectory()));

        services.AddScoped<IEncryptionService, EncryptionService>();
        return services;
    }

    public static IServiceCollection RegisterRabbitMqBusConsumers(this IServiceCollection services)
    {
        return services.AddRabbitMqBus(mqBuilder =>
            mqBuilder
                .AddConsumer<UserCreated, UserCreatedConsumer>("user-exchange", "user-created-notification-service", "user.created")
                .AddConsumer<UserAcceptedFriendRequest, UserAcceptedFriendRequestConsumer>("user-exchange", "user-accepted-friend-request-service", "user.add.friend")
                .AddConsumer<UserSentFriendRequest, UserSentFriendRequestConsumer>("user-exchange", "user-sent-friend-request-notification-service", "user.sent.friend")
                .AddConsumer<UserRemovedFriend, UserRemovedFriendConsumer>("user-exchange", "user-friend-removed-notification-service", "user.friends.remove")
                .AddConsumer<UserDeclinedFriendRequest, UserDeclinedFriendRequestConsumer>("user-exchange", "user-declined-friend-request-notification-service",
                    "user.remove.friend")
                .AddConsumer<UserFollowed, UserFollowedConsumer>("user-exchange", "user-followed-notification-service", "user.followed")
                .AddConsumer<UserUnfollowed, UserUnfollowedConsumer>("user-exchange", "user-unfollowed-notification-service", "user.unfollowed")
                .AddConsumer<UserBlocked, UserBlockedConsumer>("user-exchange", "user-blocked-notification-service", "user.blocked")
                .AddConsumer<UserUnblocked, UserUnblockedConsumer>("user-exchange", "user-unblocked-notification-service", "user.unblocked"));
    }
}