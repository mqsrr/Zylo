use prometheus::{Encoder, Registry, TextEncoder};


pub fn init_prometheus() -> Registry {
    let registry = Registry::new();
    registry
}


pub async fn metrics_handler(registry: Registry) -> impl axum::response::IntoResponse {
    let encoder = TextEncoder::new();
    let metric_families = registry.gather();
    let mut buffer = Vec::new();
    encoder.encode(&metric_families, &mut buffer).unwrap();
    String::from_utf8(buffer).unwrap()
}