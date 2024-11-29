using System.Text;
using Azure.Identity;
using Microsoft.AspNetCore.Authentication.JwtBearer;
using Microsoft.IdentityModel.Tokens;
using UserManagement.Application.Services;

namespace UserManagement.Application.Extensions;

internal static class ConfigurationBuilderExtensions
{
    public static IConfigurationBuilder AddEnvFile(this IConfigurationBuilder config, string filePath = ".env")
    {
        if (Environment.GetEnvironmentVariable("ASPNETCORE_ENVIRONMENT") == Environments.Production)
        {
            return config;
        }
        
        if (!File.Exists(filePath))
        {
            throw new FileNotFoundException($"The .env file at path '{filePath}' was not found.");
        }

        ReadOnlySpan<string> lines = File.ReadAllLines(filePath);
        ReadOnlySpan<char> separators = [':', '='];
        ReadOnlySpan<char> trimCharacters = ['"', ' ', ','];
        
        foreach (string line in lines)
        {
            var trimmedLine = line.AsSpan().Trim(trimCharacters);
            if (trimmedLine.IsEmpty || trimmedLine.StartsWith("#"))
            {
                continue;
            }
            
            int delimiterIndex = trimmedLine.IndexOfAny(separators);
            if (delimiterIndex == -1)
            {
                continue;
            }

            var key = trimmedLine[..delimiterIndex].Trim(trimCharacters);
            var value = trimmedLine[(delimiterIndex + 1)..].Trim(trimCharacters);
            
          
            Environment.SetEnvironmentVariable(key.ToString(), value.ToString());
        }

        return config;
    }

    public static IConfigurationBuilder AddAzureKeyVault(this IConfigurationBuilder configuration)
    {
        configuration.AddEnvironmentVariables();
        if (!string.IsNullOrEmpty(Environment.GetEnvironmentVariable("Test")))
        {
            return configuration;
        }

        string? keyVaultUrl = Environment.GetEnvironmentVariable("AZURE_KEY_VAULT_URL");
        if (string.IsNullOrEmpty(keyVaultUrl))
        {
            throw new InvalidOperationException("Azure Key Vault URL is missing or empty.");
        }

        return configuration.AddAzureKeyVault(new Uri(keyVaultUrl),
            new DefaultAzureCredential(),
            new PrefixKeyVaultSecretManager(["UserManagement", "Zylo"]));
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
                    ClockSkew = TimeSpan.FromSeconds(5),
                };

                options.Events = new JwtBearerEvents
                {
                    OnTokenValidated = async context =>
                    {
                        var emailVerifiedClaim = context.Principal?.FindFirst("email_verified");

                        if (emailVerifiedClaim == null || !bool.TryParse(emailVerifiedClaim.Value, out bool emailVerified) || !emailVerified)
                        {
                            context.Fail("Email not verified.");
                        }

                        await Task.CompletedTask;
                    },
                    OnChallenge = async context =>
                    {
                        if (!context.Response.HasStarted)
                        {
                            context.Response.ContentType = "application/json";
                            context.Response.StatusCode = StatusCodes.Status401Unauthorized;

                            string errorMessage = context.AuthenticateFailure?.Message switch
                            {
                                "Email not verified." => "Email has not been verified. Please verify your email to continue.",
                                _ => "Invalid token."
                            };

                            await context.Response.WriteAsync($"{{\"error\": \"{errorMessage}\"}}");
                        }

                        context.HandleResponse();
                    },
                    OnAuthenticationFailed = async context =>
                    {
                        // Check if the response has already started
                        if (!context.Response.HasStarted)
                        {
                            context.Response.ContentType = "application/json";
                            context.Response.StatusCode = StatusCodes.Status401Unauthorized;

                            string errorMessage = context.Exception switch
                            {
                                SecurityTokenExpiredException => "Token has expired.",
                                SecurityTokenInvalidSignatureException => "Invalid token signature.",
                                _ => "Authentication failed."
                            };

                            await context.Response.WriteAsync($"{{\"error\": \"{errorMessage}\"}}");
                        }
                    }
                };

            });

        return builder.Configuration;
    }

}