namespace NotificationService.Application.Settings;

public abstract class BaseSettings(string sectionName)
{
    public readonly string SectionName = sectionName;
}