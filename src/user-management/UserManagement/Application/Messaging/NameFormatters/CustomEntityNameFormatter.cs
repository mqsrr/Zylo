using MassTransit;

namespace UserManagement.Application.Messaging.NameFormatters;

public class CustomEntityNameFormatter : IEntityNameFormatter
{
    public string FormatEntityName<T>()
    {
        return typeof(T).Name.ToLower() switch
        {
            "usercreated" => "user-exchange",   
            "userupdated" => "user-exchange",   
            "userdeleted" => "user-exchange",    
            "verifyemailaddress" => "user-exchange",    
            _ => "default-exchange"
        };
    }
}