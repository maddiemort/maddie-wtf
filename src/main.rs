use std::net::SocketAddr;

use axum::{body::Body, http::Request, middleware, routing::get, Router};
use axum_tracing_opentelemetry::middleware::OtelAxumLayer;
use camino::Utf8PathBuf;
use clap::Parser;
use tokio::net::TcpListener;
use tower::ServiceExt;
use tower_http::services::ServeDir;
use tower_livereload::LiveReloadLayer;
use tracing::{debug, error, info};

use crate::state::Config;

mod errors;
mod handlers;
mod state;
mod templates;

#[derive(Parser, Clone, Debug)]
pub struct Args {
    #[arg(long, short, env = "ADDRESS", default_value = "0.0.0.0:6942")]
    address: SocketAddr,

    #[arg(long, short, env = "DRAFTS")]
    drafts: bool,

    #[arg(long, env = "CONTENT_PATH")]
    content_path: Utf8PathBuf,

    #[arg(long, env = "STATIC_PATH")]
    static_path: Utf8PathBuf,

    #[arg(long, env = "THEMES_PATH")]
    themes_path: Utf8PathBuf,
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    maddie_wtf::init_tracing();

    let args = Args::parse();

    info!(addr = %args.address, "starting TCP server");

    let listener = match TcpListener::bind(&args.address).await {
        Ok(listener) => {
            info!(addr = %args.address, "bound TCP listener");
            listener
        }
        Err(error) => {
            error!(addr = %args.address, %error, "failed to bind TCP listener, aborting");
            return;
        }
    };

    let config = Config::from(args);

    info!(
        %config.drafts,
        %config.content_path,
        %config.static_path,
        %config.themes_path,
        "loaded config",
    );

    let live_reload = LiveReloadLayer::new();
    let reloader = live_reload.reloader();

    let app = Router::new()
        .route("/", get(handlers::index))
        .route("/posts", get(handlers::posts))
        .route("/posts/:post", get(handlers::post))
        .route("/posts/:post/entry/:index", get(handlers::entry))
        .route("/chrono", get(handlers::chrono))
        .route("/tags", get(handlers::tags))
        .route("/tagged/:tag", get(handlers::tagged))
        .route("/style.css", get(handlers::stylesheet))
        .route("/rss.xml", get(handlers::rss_feed));

    let app = app.nest_service(
        "/static",
        ServeDir::new(&config.static_path).map_request(|req: Request<Body>| {
            debug!(route = %req.uri(), under = %"/static", "handling nested request");
            req
        }),
    );

    #[cfg(debug_assertions)]
    let app = app.route("/break", get(handlers::internal_error));

    let app = app.route("/:page", get(handlers::page));

    let state = match config.load_state(reloader).await {
        Ok(state) => state,
        Err(error) => {
            error!(%error, "failed to load state, aborting");
            return;
        }
    };

    let app = app.fallback(handlers::not_found);

    #[cfg(debug_assertions)]
    let app = app.layer(live_reload);

    let app = app
        .layer(OtelAxumLayer::default())
        .layer(middleware::from_fn_with_state(
            state.clone(),
            errors::render_error,
        ))
        .with_state(state);

    match axum::serve(listener, app.into_make_service())
        .with_graceful_shutdown(maddie_wtf::graceful_shutdown())
        .await
    {
        Ok(_) => {
            info!("app service exited normally");
        }
        Err(error) => {
            error!(%error, "app service exited with error");
        }
    }
}
