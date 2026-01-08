use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use metrics_exporter_prometheus::{BuildError, PrometheusBuilder};
use tracing_subscriber::{
    fmt,
    layer::SubscriberExt as _,
    util::{SubscriberInitExt as _, TryInitError},
    EnvFilter,
};

use crate::config::Environment;

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

pub fn init_metrics(port: u16, environment: Environment) -> Result<(), BuildError> {
    PrometheusBuilder::new()
        .with_http_listener(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), port))
        .add_global_label("environment", environment.to_string())
        .install()
}
