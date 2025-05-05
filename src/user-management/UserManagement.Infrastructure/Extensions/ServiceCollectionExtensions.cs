using System.Collections.Concurrent;
using System.IO.Compression;
using System.Reflection;
using Amazon.S3;
using Dapper;
using Microsoft.Extensions.Configuration;
using Microsoft.Extensions.DependencyInjection;
using Newtonsoft.Json;
using Newtonsoft.Json.Serialization;
using RabbitMQ.Client;
using StackExchange.Redis;
using UserManagement.Application.Builders;
using UserManagement.Application.Converters;
using UserManagement.Application.Messages;
using UserManagement.Application.Repositories.Auth;
using UserManagement.Application.Repositories.Users;
using UserManagement.Application.Services.Auth;
using UserManagement.Application.Services.Common;
using UserManagement.Application.Services.User;
using UserManagement.Application.Settings;
using UserManagement.Application.Transport;
using UserManagement.Infrastructure.Persistence.Factories;
using UserManagement.Infrastructure.Repositories.Auth;
using UserManagement.Infrastructure.Repositories.Users;
using UserManagement.Infrastructure.Services.Auth;
using UserManagement.Infrastructure.Services.Common;
using UserManagement.Infrastructure.Services.Users;
using UserManagement.Infrastructure.Transport.Bus;
using UserManagement.Infrastructure.Transport.Factories;
using UserManagement.Infrastructure.Transport.HostedServices;
using UserManagement.Infrastructure.Transport.Producers;
using UserManagement.Infrastructure.TypeHandlers;

namespace UserManagement.Infrastructure.Extensions;

public static class ServiceCollectionExtensions
{
    private static IHealthChecksBuilder AddServiceHealthChecks(this IServiceCollection services, IConfiguration configuration)
    {
        return services.AddHealthChecks()
            .AddRedis(configuration["Redis:ConnectionString"]!, name: "Redis", tags: ["Redis"])
            .AddNpgSql(configuration["Postgres:ConnectionString"]!, name: "Postgres", tags: ["Database"]);
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


    private static IServiceCollection AddConnectionMultiplexer(this IServiceCollection services, string connectionString)
    {
        var connection = ConnectionMultiplexer.Connect(connectionString);
        var response = connection.GetDatabase().Execute("PING");
        if (response.IsNull)
        {
            throw new Exception("Redis connection failed");
        }

        services.AddSingleton<IConnectionMultiplexer>(connection);
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
                settings.Publishers = builder.Build();
            });

        services.AddScoped(typeof(IProducer<>), typeof(RabbitMqProducer<>));
        services.AddHostedService<RabbitMqBusHostedService>();

        return services;
    }

    private static IServiceCollection AddTypeHandlers(this IServiceCollection services)
    {
        SqlMapper.AddTypeHandler(new IdentityIdTypeHandler());
        SqlMapper.AddTypeHandler(new UserIdTypeHandler());
        SqlMapper.AddTypeHandler(new UlidTypeHandler());
        SqlMapper.AddTypeHandler(new DateOnlyTypeHandler());
        SqlMapper.AddTypeHandler(new DateTimeTypeHandler());

        return services;
    }

    public static IServiceCollection AddInfrastructure(this IServiceCollection services, IConfiguration configuration)
    {

        services.AddConnectionMultiplexer(configuration["Redis:ConnectionString"]!);

        services.AddScoped<IDbConnectionFactory, PostgresDbConnectionFactory>();

        services.AddScoped<IUserRepository, UserRepository>();
        services.Decorate<IUserRepository, ExceptionlessUserRepository>();
        services.Decorate<IUserRepository>((repository, provider) => new ObservableUserRepository(repository, provider.GetRequiredService<Instrumentation>()));
        services.Decorate<IUserRepository>((repository, provider) => new CachedUserRepository(repository, provider.GetRequiredService<ICacheService>()));

        services.AddScoped<IIdentityRepository, IdentityRepository>();
        services.Decorate<IIdentityRepository, ExceptionlessIdentityRepository>();
        services.Decorate<IIdentityRepository>((repository, provider) => new ObservableIdentityRepository(repository, provider.GetRequiredService<Instrumentation>()));
        services.Decorate<IIdentityRepository>((repository, provider) => new CachedIdentityRepository(repository, provider.GetRequiredService<ICacheService>()));

        services.AddScoped<IRefreshTokenRepository, RefreshTokenRepository>();
        services.Decorate<IRefreshTokenRepository, ExceptionlessRefreshTokenRepository>();
        services.Decorate<IRefreshTokenRepository>((repository, provider) =>
            new ObservableRefreshTokenRepository(repository, provider.GetRequiredService<Instrumentation>()));

        services.AddScoped<IOtpRepository, OtpRepository>();
        services.Decorate<IOtpRepository, ExceptionlessOtpRepository>();
        services.Decorate<IOtpRepository>((repository, provider) => new ObservableOtpRepository(repository, provider.GetRequiredService<Instrumentation>()));

        services.AddScoped<IUserService, UserService>();
        services.AddScoped<IIdentityService, IdentityService>();

        services.AddScoped<ICacheService, CacheService>();
        services.Decorate<ICacheService, ObservableCacheService>();

        services.AddScoped<IImageService, ImageService>();
        services.Decorate<IImageService, ObservableImageService>();
        services.Decorate<IImageService>((repository, provider) => new CachedImageService(repository, provider.GetRequiredService<ICacheService>()));

        services.AddScoped<IOtpService, OtpService>();
        services.AddScoped<IEncryptionService, EncryptionService>();
        services.AddScoped<IHashService, HashService>();

        services.AddScoped<ITokenWriter, TokenWriter>();
        services.AddScoped<ITokenService, TokenService>();

        services.AddScoped<IAuthService, AuthService>();

        services.AddGrpc(options =>
        {
            const short kilobyte = 1024;
            options.EnableDetailedErrors = Environment.GetEnvironmentVariable("ASPNETCORE_ENVIRONMENT") == "Development";

            options.MaxReceiveMessageSize = 2 * kilobyte * kilobyte;
            options.MaxSendMessageSize = 2 * kilobyte * kilobyte;

            options.ResponseCompressionLevel = CompressionLevel.Fastest;
        });

        services.AddSingleton<IAmazonS3, AmazonS3Client>();
        services.AddApplicationSettings(configuration);


        services.AddRabbitMqBus(mqBuilder =>
            mqBuilder
                .AddPublisher<UserCreated>("user-exchange", "user.created")
                .AddPublisher<UserDeleted>("user-exchange", "user.deleted")
                .AddPublisher<VerifyEmailAddress>("user-exchange", "user.verify.email"));

        services.AddServiceHealthChecks(configuration);
        services.AddTypeHandlers();
        return services;
    }

    public static IServiceCollection RegisterRabbitMqPublishers(this IServiceCollection services)
    {
        return services.AddRabbitMqBus(mqBuilder =>
            mqBuilder
                .AddPublisher<UserCreated>("user-exchange", "user.created")
                .AddPublisher<UserDeleted>("user-exchange", "user.deleted")
                .AddPublisher<VerifyEmailAddress>("user-exchange", "user.verify.email"));

    }
}