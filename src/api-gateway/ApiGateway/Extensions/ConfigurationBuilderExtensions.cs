using System.Reflection;
using System.Text;
using Azure.Identity;
using Microsoft.AspNetCore.Authentication.JwtBearer;
using Microsoft.IdentityModel.Tokens;

namespace ApiGateway.Extensions;

internal static class ConfigurationBuilderExtensions
{
    public static IConfigurationBuilder AddAzureKeyVault(this IConfigurationBuilder configuration)
    {
        if (!string.IsNullOrEmpty(Environment.GetEnvironmentVariable(Environments.Staging)))
        {
            return configuration;
        }
        string? keyVaultUrl = Environment.GetEnvironmentVariable("AZURE_KEY_VAULT_URL");
        if (string.IsNullOrEmpty(keyVaultUrl))
        {
            throw new InvalidOperationException("Azure Key Vault URL is missing or empty.");
        }
        
        return configuration.AddEnvironmentVariables()
            .AddAzureKeyVault(new Uri(keyVaultUrl),
                new DefaultAzureCredential(),
                new PrefixKeyVaultSecretManager(["Zylo"]));
    }

    
    public static IConfigurationBuilder AddJwtBearer(this IConfigurationBuilder config, WebApplicationBuilder builder)
    {
        builder.Services.AddAuthorization(options =>
            {
                options.AddPolicy("Bearer", policyBuilder =>
                {
                    policyBuilder.AddAuthenticationSchemes(JwtBearerDefaults.AuthenticationScheme);
                    policyBuilder.RequireAuthenticatedUser().Build();
                });
            })
            .AddAuthentication(options =>
            {
                options.DefaultChallengeScheme = JwtBearerDefaults.AuthenticationScheme;
                options.DefaultAuthenticateScheme = JwtBearerDefaults.AuthenticationScheme;
                options.DefaultScheme = JwtBearerDefaults.AuthenticationScheme;
            })
            .AddJwtBearer("Bearer", options =>
            {
                options.TokenValidationParameters = new TokenValidationParameters
                {
                    ValidateIssuer = true,
                    ValidateAudience = true,
                    ValidateLifetime = true,
                    ValidateIssuerSigningKey = true,
                    
                    ValidAudience = builder.Configuration["Jwt:Audience"],
                    ValidIssuer = builder.Configuration["Jwt:Issuer"],
                    IssuerSigningKey = new SymmetricSecurityKey(Encoding.UTF8.GetBytes(builder.Configuration["Jwt:Secret"]!)),
                    ClockSkew = TimeSpan.FromSeconds(5)
                };
            });

        return builder.Configuration;
    }
}