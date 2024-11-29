namespace NotificationService.Extensions;

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
}