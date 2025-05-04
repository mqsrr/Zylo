namespace NotificationService.Helpers;

internal static class ApiEndpoints
{
    private const string ApiBase = "/api";

    public class Notifications
    {
        private const string Base = $"{ApiBase}/users/{{id}}";

        public const string GetAll = $"{Base}/notifications";
        public const string UpdateManySeen = $"{Base}/notifications";
        public const string DeleteManyById = $"{Base}/notifications";
    }
}