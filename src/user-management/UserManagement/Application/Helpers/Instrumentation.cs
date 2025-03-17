using System.Collections.Concurrent;
using System.Collections.Frozen;
using System.Diagnostics;
using System.Diagnostics.Metrics;

namespace UserManagement.Application.Helpers;

public sealed class Instrumentation : IDisposable
{
    internal const string ActivitySourceName = "user-management";
    internal const string MeterName = "UserManagementAPI";
    internal ActivitySource ActivitySource { get; }
    internal Meter Meter { get; }

    private ConcurrentDictionary<string, Counter<long>> Counters { get; }
    private ConcurrentDictionary<string, Histogram<double>> Histograms { get; }
    private ConcurrentDictionary<string, long> GaugeValues { get; }

    public Instrumentation(IMeterFactory factory, ActivitySource activitySource)
    {
        ActivitySource = activitySource;
        Meter = factory.Create(MeterName);

        Counters = new ConcurrentDictionary<string, Counter<long>>();
        Histograms = new ConcurrentDictionary<string, Histogram<double>>();
        GaugeValues = new ConcurrentDictionary<string, long>();
    }

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


    public void RegisterGauge(string name, string? description = null)
    {
        Meter.CreateObservableGauge(name, () => GaugeValues.GetValueOrDefault(name, 0),  null, description);
    }

    public void IncrementGauge(string name, long delta)
    {
        GaugeValues.AddOrUpdate(name, delta, (_, current) => current + delta);
    }
}