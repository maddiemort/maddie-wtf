use tracing_subscriber::{
    fmt,
    layer::SubscriberExt as _,
    util::{SubscriberInitExt as _, TryInitError},
    EnvFilter,
};

pub fn init_tracing(debug: bool) -> Result<(), TryInitError> {
    let registry = tracing_subscriber::registry().with(
        EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("otel::tracing=trace,info")),
    );

    if debug {
        registry
            .with(fmt::layer().with_timer(fmt::time::uptime()))
            .try_init()
    } else {
        registry.with(fmt::layer()).try_init()
    }
}
