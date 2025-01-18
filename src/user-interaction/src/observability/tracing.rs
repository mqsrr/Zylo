use dotenv::dotenv;
use opentelemetry::{global, KeyValue};
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::trace::TracerProvider;

pub fn init_tracing() -> TracerProvider {
    dotenv().ok();

    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .build()
        .unwrap();

    let resource = Resource::new(vec![
        KeyValue::new("service.name", "user-interaction"),
        KeyValue::new("service.version", "1.0.0"),
    ]);

    let provider = TracerProvider::builder()
        .with_resource(resource)
        .with_batch_exporter(exporter, opentelemetry_sdk::runtime::Tokio)
        .build();


    global::set_tracer_provider(provider.clone());
    provider
}
