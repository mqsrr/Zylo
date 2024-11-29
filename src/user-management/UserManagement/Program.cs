using System.IO.Compression;
using Amazon.S3;
using Asp.Versioning;
using FluentValidation;
using FluentValidation.AspNetCore;
using MassTransit;
using Microsoft.AspNetCore.Server.Kestrel.Core;
using Microsoft.Extensions.Options;
using Serilog;
using UserManagement.Application.Extensions;
using UserManagement.Application.Factories.Abstractions;
using UserManagement.Application.Interceptors;
using UserManagement.Application.Messaging.NameFormatters;
using UserManagement.Application.Messaging.Users;
using UserManagement.Application.Repositories;
using UserManagement.Application.Repositories.Abstractions;
using UserManagement.Application.Services;
using UserManagement.Application.Services.Abstractions;
using UserManagement.Application.Settings;
using UserManagement.Application.Validators;

var builder = WebApplication.CreateBuilder(args);

builder.Host.UseSerilog((context, configuration) =>
    configuration.ReadFrom.Configuration(context.Configuration));

builder.Configuration.AddEnvFile();
builder.Configuration.AddAzureKeyVault();
builder.Configuration.AddJwtBearer(builder);

builder.Services.AddControllers();
builder.Services.AddMediator();

builder.Services.AddApiVersioning(new HeaderApiVersionReader());
builder.Services.AddConnectionMultiplexer(builder.Configuration["Redis:ConnectionString"]!);

builder.Services.AddApplicationService<ITokenWriter>();
builder.Services.AddApplicationService<IAuthService>();

builder.Services.AddApplicationService<IDbConnectionFactory>();
builder.Services.AddApplicationService<IUserRepository>();

builder.Services.AddApplicationService<ICacheService>();
builder.Services.AddApplicationService<IImageService>();

builder.Services.AddApplicationService<IOtpService>();
builder.Services.AddApplicationService<IEncryptionService>();
builder.Services.AddApplicationService<IHashService>();

builder.Services.AddGrpc(options =>
{
    const short kilobyte = 1024;
    options.EnableDetailedErrors = builder.Environment.IsDevelopment();
    
    options.MaxReceiveMessageSize = 2 * kilobyte * kilobyte;
    options.MaxSendMessageSize = 2 * kilobyte * kilobyte; 

    options.Interceptors.Add<ExceptionInterceptor>();
    options.ResponseCompressionLevel = CompressionLevel.Fastest;
});

builder.Services.AddSingleton<IAmazonS3, AmazonS3Client>();

builder.Services.Decorate<IUserRepository, CachedUserRepository>();
builder.Services.Decorate<IAuthService, ExceptionlessAuthService>();

builder.Services.AddOptionsWithValidateOnStart<JwtSettings>()
    .Bind(builder.Configuration.GetRequiredSection(JwtSettings.SectionName));

builder.Services.AddOptionsWithValidateOnStart<PostgresDbSettings>()
    .Bind(builder.Configuration.GetRequiredSection(PostgresDbSettings.SectionName));

builder.Services.AddOptionsWithValidateOnStart<RabbitMqSettings>()
    .Bind(builder.Configuration.GetRequiredSection(RabbitMqSettings.SectionName));

builder.Services.AddOptionsWithValidateOnStart<S3Settings>()
    .Bind(builder.Configuration.GetRequiredSection(S3Settings.SectionName));

builder.Services.AddOptionsWithValidateOnStart<EncryptionSettings>()
    .Bind(builder.Configuration.GetRequiredSection(EncryptionSettings.SectionName));

builder.Services.AddOptionsWithValidateOnStart<OtpSettings>()
    .Bind(builder.Configuration.GetRequiredSection(OtpSettings.SectionName));

builder.Services.AddFluentValidationAutoValidation()
    .AddValidatorsFromAssemblyContaining<RegisterRequestValidator>(includeInternalTypes: true);

builder.Services.AddMassTransit(configurator =>
{
    configurator.SetKebabCaseEndpointNameFormatter();
    configurator.UsingRabbitMq((context, factoryConfigurator) =>
    {
        factoryConfigurator.MessageTopology.SetEntityNameFormatter(new CustomEntityNameFormatter());
        var settings = context.GetRequiredService<IOptions<RabbitMqSettings>>().Value;
        
        factoryConfigurator.Publish<VerifyEmailAddress>(topologyConfigurator => topologyConfigurator.ExchangeType = "direct");
        factoryConfigurator.Publish<UserCreated>(topologyConfigurator => topologyConfigurator.ExchangeType = "direct");
        factoryConfigurator.Publish<UserUpdated>(topologyConfigurator => topologyConfigurator.ExchangeType = "direct");
        factoryConfigurator.Publish<UserDeleted>(topologyConfigurator => topologyConfigurator.ExchangeType = "direct");

        factoryConfigurator.Host(new Uri(settings.ConnectionString));

        factoryConfigurator.UseRawJsonSerializer(RawSerializerOptions.All, true);
        factoryConfigurator.UseRawJsonDeserializer(RawSerializerOptions.All, true);

        factoryConfigurator.ConfigureEndpoints(context);
    });
});

builder.WebHost.ConfigureKestrel(options =>
{
    options.ListenAnyIP(8070, listenOptions =>
    {
        listenOptions.Protocols = HttpProtocols.Http2;
    });

    options.ListenAnyIP(8080, listenOptions =>
    {
        listenOptions.Protocols = HttpProtocols.Http1;
    });
});

builder.Services.AddServiceHealthChecks(builder);
builder.Services.AddCors(options =>
    options.AddDefaultPolicy(policyBuilder =>
        policyBuilder.AllowAnyHeader()
            .AllowAnyMethod()
            .WithOrigins("http://localhost:5173")
            .AllowCredentials()));

var app = builder.Build();

app.MigrateDatabase();

app.UseSerilogRequestLogging();

app.UseAuthentication();
app.UseAuthorization();
app.MapControllers();

app.MapHealthChecks("/healthz").ExcludeFromDescription();

app.MapGrpcService<ProfileService>();
app.UseCors();
app.Run();


