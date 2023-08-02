pub use opentelemetry::sdk::propagation::TraceContextPropagator;
pub use opentelemetry::trace::*;
pub use opentelemetry::{global, KeyValue};

pub fn tracing_init(name: &String) -> Result<impl Tracer, TraceError> {
    global::set_text_map_propagator(TraceContextPropagator::new());
    opentelemetry_jaeger::new_agent_pipeline()
        .with_service_name(name)
        .install_simple()
}
