use cfg_if::cfg_if;
use tokio::signal;
use tracing::{info, instrument};

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
            // A future that will never complete, because non-Unix platforms don't have Unix
            // signals!
            let terminate = std::future::pending::<()>();
        }
    };

    // Wait for either of those futures to complete, which means that one of the termination
    // signals has been received.
    tokio::select! {
        _ = ctrl_c => {
            // Print a newline to move past the rightward drift created by terminals printing the
            // ctrl-C escape sequence.
            println!();
            info!("ctrl-c received")
        },
        _ = terminate => info!("termination signal received"),
    }

    info!("shutting down, see you soon!");
}
