namespace UserManagement.Application.Settings;

public abstract class BaseSettings(string sectionName)
{
    public string SectionName = sectionName;
}