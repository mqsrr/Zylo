namespace UserManagement.Application.Helpers;

internal static class ApiEndpoints
{
    private const string ApiBase = "/api";
    
    public class Authentication
    {
        private const string Base = $"{ApiBase}/auth";

        public const string Register = $"{Base}/register";
        public const string Login = $"{Base}/login";
        public const string RefreshAccessToken = $"{Base}/token/refresh";
        public const string RevokeRefreshToken = $"{Base}/token/revoke";
        
        public const string VerifyUserEmail = $"{Base}/users/{{id}}/verify/email";
    }
    
    public class Users
    {
        private const string Base = $"{ApiBase}/users";
        
        public const string Update = $"{Base}/{{id}}";
        public const string GetById = $"{Base}/{{id}}";
        public const string DeleteById = $"{Base}/{{id}}";

    }
}