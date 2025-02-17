using System.Collections.Concurrent;
using System.Diagnostics;
using System.Diagnostics.Metrics;

namespace UserManagement.Application.Helpers;

public sealed class Instrumentation : IDisposable
{
    internal const string ActivitySourceName = "user-management";
    internal const string ActivitySourceVersion = "1.0.0";
    internal const string MeterName = "UserManagementAPI";

    public Instrumentation(IMeterFactory factory, ActivitySource activitySource)
    {
        ActivitySource = activitySource;
        Meter = factory.Create(MeterName);
        
        Counters = new ConcurrentDictionary<string, Counter<long>>();
        Histograms = new ConcurrentDictionary<string, Histogram<double>>();
    }

    internal ActivitySource ActivitySource { get; }
    internal Meter Meter { get; }

    private ConcurrentDictionary<string, Counter<long>> Counters { get; }
    private ConcurrentDictionary<string, Histogram<double>> Histograms { get; }

    public void Dispose()
    {
        ActivitySource.Dispose();
        Meter.Dispose();
    }

    public Counter<long> GetCounterOrCreate(string name, string? description = null)
    {
        return Counters.GetOrAdd(name, Meter.CreateCounter<long>(name, description: description));
    }

    public Histogram<double> GetHistogramOrCreate(string name, string? description = null)
    {
        return Histograms.GetOrAdd(name, Meter.CreateHistogram<double>(name, description: description));
    }
}