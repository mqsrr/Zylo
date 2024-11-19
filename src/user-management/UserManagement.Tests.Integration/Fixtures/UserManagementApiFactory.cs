using System.IdentityModel.Tokens.Jwt;
using System.Net.Http.Headers;
using System.Security.Claims;
using System.Text;
using Amazon.S3;
using DotNet.Testcontainers.Builders;
using MassTransit;
using Microsoft.AspNetCore.Hosting;
using Microsoft.AspNetCore.Mvc.Testing;
using Microsoft.AspNetCore.TestHost;
using Microsoft.Extensions.DependencyInjection;
using Microsoft.Extensions.DependencyInjection.Extensions;
using Microsoft.Extensions.Options;
using Microsoft.IdentityModel.Tokens;
using NSubstitute;
using Testcontainers.PostgreSql;
using Testcontainers.RabbitMq;
using Testcontainers.Redis;
using UserManagement.Application.Settings;
using UserManagement.Controllers;

namespace UserManagement.Tests.Integration.Fixtures;

public sealed class UserManagementApiFactory : WebApplicationFactory<UsersController>, IAsyncLifetime
{
    private readonly RedisContainer _redisContainer;
    private readonly RabbitMqContainer _rabbitMqContainer;
    private readonly PostgreSqlContainer _postgreContainer;

    public IAmazonS3 S3 { get; init; }


    public UserManagementApiFactory()
    {
        _redisContainer = new RedisBuilder()
            .WithName("user-management-test-redis")
            .WithImage("redis:alpine")
            .WithExposedPort(6379)
            .WithWaitStrategy(Wait.ForUnixContainer().UntilPortIsAvailable(6379))
            .Build();

        _rabbitMqContainer = new RabbitMqBuilder()
            .WithUsername("user-management-test-username")
            .WithPassword("user-management-test-password")
            .WithName("user-management-test-rabbitmq")
            .WithImage("rabbitmq:management-alpine")
            .WithExposedPort(15672)
            .WithExposedPort(5672)
            .WithWaitStrategy(Wait.ForUnixContainer().UntilPortIsAvailable(5672))
            .Build();

        _postgreContainer = new PostgreSqlBuilder()
            .WithUsername("user-management-postgres-username")
            .WithPassword("user-management-postgres-password")
            .WithDatabase("test")
            .WithName("user-management-postgres")
            .WithExposedPort(5432)
            .WithWaitStrategy(Wait.ForUnixContainer().UntilPortIsAvailable(5432))
            .Build();
        
        S3 = Substitute.For<IAmazonS3>();
    }

    protected override void ConfigureWebHost(IWebHostBuilder builder)
    {
        Environment.SetEnvironmentVariable("Test", "true");

        builder.UseSetting("Jwt:Audience", "user-management-test");
        builder.UseSetting("Jwt:Issuer", "user-management-test");
        builder.UseSetting("Jwt:Secret", "user-management-test-api-key!!!!!!!!!!!!!!");
        
        builder.UseSetting("S3:BucketName", "user-management-test-bucket");
        builder.UseSetting("S3:PresignedUrlExpire", "600");

        builder.UseSetting("Redis:ConnectionString", _redisContainer.GetConnectionString());
        builder.UseSetting("RabbitMq:ConnectionString", _rabbitMqContainer.GetConnectionString());
        builder.UseSetting("Postgres:ConnectionString", _postgreContainer.GetConnectionString());

        builder.ConfigureTestServices(services =>
        {
            services.RemoveAll(typeof(IAmazonS3));
            services.AddScoped<IAmazonS3>(_ => S3);

            services.AddMassTransitTestHarness();
        });
    }

    protected override void ConfigureClient(HttpClient client)
    {
        var jwtSettings = Services.GetRequiredService<IOptions<JwtSettings>>().Value;
        string token = WriteToken(jwtSettings);
        
        client.DefaultRequestHeaders.Authorization = new AuthenticationHeaderValue("Bearer", token);
    }

    public async Task InitializeAsync()
    {
        await _redisContainer.StartAsync();
        await _rabbitMqContainer.StartAsync();
        await _postgreContainer.StartAsync();
    }

    public async new Task DisposeAsync()
    {
        await _redisContainer.StopAsync();
        await _rabbitMqContainer.StopAsync();
        await _postgreContainer.StopAsync();
    }
    
    private static string WriteToken(JwtSettings jwtSettings)
    {
        var symmetricKey = new SymmetricSecurityKey(Encoding.UTF8.GetBytes(jwtSettings.Secret));
        var signingCred = new SigningCredentials(symmetricKey, SecurityAlgorithms.HmacSha256Signature);
        var jwtSecurityToken = new JwtSecurityToken(
            jwtSettings.Issuer,
            jwtSettings.Audience,
            [new Claim(ClaimTypes.System, "testing")],
            DateTime.UtcNow,
            DateTime.UtcNow.AddMinutes(jwtSettings.Expire),
            signingCred);

        return new JwtSecurityTokenHandler().WriteToken(jwtSecurityToken);
    }
}