using ApiGateway.Aggregators;
using ApiGateway.DelegatingHandlers;
using ApiGateway.Extensions;
using Ocelot.DependencyInjection;
using Ocelot.Middleware;
using Ocelot.Provider.Polly;
using Serilog;

var builder = WebApplication.CreateBuilder(args);

builder.Host.UseSerilog((context, configuration) => 
    configuration.ReadFrom.Configuration(context.Configuration));

builder.Configuration
    .AddJsonFile("ocelot.json", false, true)
    .AddAzureKeyVault()
    .AddJwtBearer(builder);

builder.Services
    .AddHttpClient()
    .AddOcelot(builder.Configuration)
    .AddSingletonDefinedAggregator<UserAggregator>()
    .AddDelegatingHandler<FeedDelegatingHandler>()
    .AddDelegatingHandler<PostDelegatingHandler>()
    .AddPolly();

builder.Services.AddCors(options =>
    options.AddDefaultPolicy(policyBuilder =>
        policyBuilder
            .WithOrigins("http://localhost:5173")
            .AllowCredentials()
            .AllowAnyHeader()
            .AllowAnyMethod()));

var app = builder.Build();
app.UseSerilogRequestLogging();

app.UseAuthentication();
app.UseAuthorization();

app.UseCors();
await app.UseOcelot();

app.Run();