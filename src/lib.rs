use cfg_if::cfg_if;
use tokio::signal;
use tracing::{info, instrument};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

mod result_option_exts;

pub mod build_info;
pub mod metric;

pub use result_option_exts::{OptionExt, ResultExt};

pub fn init_tracing() {
    #[cfg(debug_assertions)]
    let fmt_layer = fmt::layer().with_timer(fmt::time::uptime());
    #[cfg(not(debug_assertions))]
    let fmt_layer = fmt::layer();

    tracing_subscriber::registry()
        .with(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("otel::tracing=trace,info")),
        )
        .with(fmt_layer)
        .init();
}

#[instrument(level = "error")]
pub async fn graceful_shutdown() {
    // A future that will listen for the ctrl-c input from a terminal.
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("should be able to listen for ctrl-c event");
    };

    cfg_if! {
        if #[cfg(unix)] {
            // A future that will listen for a SIGTERM signal.
            let terminate = async {
                signal::unix::signal(signal::unix::SignalKind::terminate())
                    .expect("should be able to install signal handler")
                    .recv()
                    .await;
            };
        } else {
            // A future that will never complete, because non-Unix platforms
            // don't have Unix signals!
            let terminate = std::future::pending::<()>();
        }
    };

    // Wait for either of those futures to complete, which means that one of the
    // termination signals has been received.
    tokio::select! {
        _ = ctrl_c => {
            // Print a newline to move past the rightward drift created by
            // terminals printing the ctrl-C escape sequence.
            println!();
            info!("ctrl-c received")
        },
        _ = terminate => info!("termination signal received"),
    }

    info!("shutting down, see you soon!");
}
