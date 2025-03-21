using System.IO.Compression;
using Amazon.S3;
using Asp.Versioning;
using FluentValidation;
using FluentValidation.AspNetCore;
using Microsoft.AspNetCore.Server.Kestrel.Core;
using Serilog;
using UserManagement.Application.Extensions;
using UserManagement.Application.Factories;
using UserManagement.Application.Factories.Abstractions;
using UserManagement.Application.Helpers;
using UserManagement.Application.Messaging.Users;
using UserManagement.Application.Middleware;
using UserManagement.Application.Repositories;
using UserManagement.Application.Repositories.Abstractions;
using UserManagement.Application.Services;
using UserManagement.Application.Services.Abstractions;
using UserManagement.Application.Validators;
using UserManagement.Decorators;

var builder = WebApplication.CreateBuilder(args);
builder.Configuration.AddEnvFile();
builder.Configuration.AddAzureKeyVault();

builder.Host.UseSerilog((context, configuration) => configuration.ReadFrom.Configuration(context.Configuration));
builder.ConfigureOpenTelemetry(builder.Configuration["OTEL:CollectorAddress"]!);

builder.Configuration.AddJwtBearer(builder);
builder.Services.ConfigureJsonSerializer();

builder.Services.AddControllers();
builder.Services.AddApiVersioning(new HeaderApiVersionReader());
builder.Services.AddConnectionMultiplexer(builder.Configuration["Redis:ConnectionString"]!);

builder.Services.AddScoped<IDbConnectionFactory, PostgresDbConnectionFactory>();

builder.Services.AddScoped<IUserRepository, UserRepository>();
builder.Services.Decorate<IUserRepository, ExceptionlessUserRepository>();
builder.Services.Decorate<IUserRepository>((repository, provider) => new ObservableUserRepository(repository, provider.GetRequiredService<Instrumentation>()));
builder.Services.Decorate<IUserRepository>((repository, provider) => new CachedUserRepository(repository, provider.GetRequiredService<ICacheService>()));

builder.Services.AddScoped<IIdentityRepository, IdentityRepository>();
builder.Services.Decorate<IIdentityRepository, ExceptionlessIdentityRepository>();
builder.Services.Decorate<IIdentityRepository>((repository, provider) => new ObservableIdentityRepository(repository, provider.GetRequiredService<Instrumentation>()));
builder.Services.Decorate<IIdentityRepository>((repository, provider) => new CachedIdentityRepository(repository, provider.GetRequiredService<ICacheService>()));

builder.Services.AddScoped<IRefreshTokenRepository, RefreshTokenRepository>();
builder.Services.Decorate<IRefreshTokenRepository, ExceptionlessRefreshTokenRepository>();
builder.Services.Decorate<IRefreshTokenRepository>((repository, provider) => new ObservableRefreshTokenRepository(repository, provider.GetRequiredService<Instrumentation>()));

builder.Services.AddScoped<IOtpRepository, OtpRepository>();
builder.Services.Decorate<IOtpRepository, ExceptionlessOtpRepository>();
builder.Services.Decorate<IOtpRepository>((repository, provider) => new ObservableOtpRepository(repository, provider.GetRequiredService<Instrumentation>()));

builder.Services.AddScoped<IUserService, UserService>();
builder.Services.AddScoped<IIdentityService, IdentityService>();

builder.Services.AddScoped<ICacheService, CacheService>();
builder.Services.Decorate<ICacheService, ObservableCacheService>();

builder.Services.AddScoped<IImageService, ImageService>();
builder.Services.Decorate<IImageService, ObservableImageService>();
builder.Services.Decorate<IImageService>((repository, provider) => new CachedImageService(repository, provider.GetRequiredService<ICacheService>()));

builder.Services.AddScoped<IOtpService, OtpService>();
builder.Services.AddScoped<IEncryptionService, EncryptionService>();
builder.Services.AddScoped<IHashService, HashService>();

builder.Services.AddScoped<ITokenWriter, TokenWriter>();
builder.Services.AddScoped<ITokenService, TokenService>();

builder.Services.AddScoped<IAuthService, AuthService>();

builder.Services.AddGrpc(options =>
{
    const short kilobyte = 1024;
    options.EnableDetailedErrors = builder.Environment.IsDevelopment();
    
    options.MaxReceiveMessageSize = 2 * kilobyte * kilobyte;
    options.MaxSendMessageSize = 2 * kilobyte * kilobyte; 

    options.ResponseCompressionLevel = CompressionLevel.Fastest;
});

builder.Services.AddSingleton<IAmazonS3, AmazonS3Client>();
builder.Services.AddApplicationSettings(builder.Configuration);

builder.Services.AddFluentValidationAutoValidation()
    .AddValidatorsFromAssemblyContaining<RegisterRequestValidator>(includeInternalTypes: true);

builder.Services.AddTransient<MetricsMiddleware>();
builder.Services.AddTransient<RequestIdMiddleware>();

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

builder.WebHost.ConfigureKestrel(options =>
{
    options.ListenAnyIP(50051, listenOptions => { listenOptions.Protocols = HttpProtocols.Http2; });
    options.ListenAnyIP(8080, listenOptions => { listenOptions.Protocols = HttpProtocols.Http1; });
});

var app = builder.Build();

app.MigrateDatabase();
app.UseSerilogRequestLogging();
app.UseMiddleware<RequestIdMiddleware>();
app.UseMiddleware<MetricsMiddleware>();

app.UseAuthentication();
app.UseAuthorization();
app.MapControllers();

app.MapHealthChecks("/healthz").ExcludeFromDescription();

app.MapGrpcService<UserManagement.Application.Services.UserProfileService>();
app.UseCors();
app.Run();