use opentelemetry::global;
use opentelemetry::propagation::Injector;
use tonic::Request;
use tracing_opentelemetry::OpenTelemetrySpanExt;

pub mod feed_service;
pub mod key_vault;
pub mod post_service;
pub mod user_service;

pub mod aggregator {
    tonic::include_proto!("user_profile_service");
    tonic::include_proto!("post_server");
    tonic::include_proto!("relationship_service");
    tonic::include_proto!("reply_server");
    tonic::include_proto!("feed_service");
}

struct MetadataMap<'a>(&'a mut tonic::metadata::MetadataMap);

impl Injector for MetadataMap<'_> {
    fn set(&mut self, key: &str, value: String) {
        if let Ok(key) = tonic::metadata::MetadataKey::from_bytes(key.as_bytes()) {
            if let Ok(val) = tonic::metadata::MetadataValue::try_from(&value) {
                self.0.insert(key, val);
            }
        }
    }
}

pub trait InjectTraceContext {
    fn inject_trace_context(self) -> Self;
}

impl<T> InjectTraceContext for Request<T> {
    fn inject_trace_context(mut self) -> Self {
        let cx = tracing::Span::current().context();
        global::get_text_map_propagator(|propagator| {
            propagator.inject_context(&cx, &mut MetadataMap(self.metadata_mut()))
        });
        self
    }
}
