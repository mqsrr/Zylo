using Azure.Extensions.AspNetCore.Configuration.Secrets;
using Azure.Security.KeyVault.Secrets;

namespace UserManagement.Application.Services;

internal sealed class PrefixKeyVaultSecretManager : KeyVaultSecretManager
{
    private readonly IEnumerable<string> _prefixes;

    public PrefixKeyVaultSecretManager(IEnumerable<string> prefixes)
    {
        _prefixes = prefixes.Select(prefix => $"{prefix}-");
    }

    public override bool Load(SecretProperties secret)
    {
        return _prefixes.Any(prefix => secret.Name.StartsWith(prefix, StringComparison.OrdinalIgnoreCase));
    }

    public override string GetKey(KeyVaultSecret secret)
    {
        string matchingPrefix = _prefixes.First(prefix => secret.Name.StartsWith(prefix, StringComparison.OrdinalIgnoreCase));
        return secret.Name[matchingPrefix.Length..].Replace("--", ConfigurationPath.KeyDelimiter);
    }
}