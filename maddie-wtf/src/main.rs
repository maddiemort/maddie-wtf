use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    str::FromStr,
};

use axum::{
    extract::Request,
    middleware::{self, Next},
    response::Response,
    routing::get,
    Router,
};
use axum_tracing_opentelemetry::middleware::OtelAxumLayer;
use camino::Utf8PathBuf;
use clap::Parser;
use maddie_wtf::metric;
use metrics_exporter_prometheus::PrometheusBuilder;
use tokio::net::TcpListener;
use tower_http::services::ServeDir;
use tower_livereload::LiveReloadLayer;
use tracing::{error, error_span, field, info, Instrument, Span};
use url::Url;

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

    #[arg(long, env = "ENVIRONMENT")]
    environment: Environment,

    #[arg(long, env = "METRICS_PORT")]
    metrics_port: Option<u16>,
}

#[derive(Copy, Clone, Debug)]
enum Environment {
    Development,
    Production,
}

impl FromStr for Environment {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "dev" | "development" => Ok(Environment::Development),
            "prod" | "production" => Ok(Environment::Production),
            _ => Err(format!("`{}` is not a valid environment name", s)),
        }
    }
}

impl std::fmt::Display for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Environment::Development => write!(f, "development"),
            Environment::Production => write!(f, "production"),
        }
    }
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

    if let Some(port) = args.metrics_port {
        let environment = args.environment.to_string();

        PrometheusBuilder::new()
            .with_http_listener(SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), port))
            .add_global_label("environment", &environment)
            .install()
            .expect("should be able to install Prometheus metrics recorder and exporter");

        info!(%port, %environment, "installed Prometheus metrics recorder and exporter");
    }

    metrics::counter!(*metric::REQUESTS_RECEIVED).absolute(0);

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

    let app = app.nest_service("/static", ServeDir::new(&config.static_path));

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
        .layer(middleware::from_fn(
            async |request: Request, next: Next| -> Response {
                async {
                    let route = request.uri().to_string();
                    Span::current().record("route", route.clone());

                    if let Some(referer) = request
                        .headers()
                        .get("Referer")
                        .and_then(|val| val.to_str().ok())
                        .and_then(|str| str.parse::<Url>().ok())
                    {
                        if let Some(referer) = referer.host_str() {
                            if referer != "maddie.wtf" {
                                Span::current().record("referer", referer);
                            }
                        }
                    }

                    info!("handling request");

                    let response = next.run(request).await;
                    let status_code = response.status();

                    metrics::counter!(
                        *metric::REQUESTS_RECEIVED,
                        "route" => route,
                        "status_code" => status_code.as_str().to_owned(),
                    )
                    .increment(1);

                    response
                }
                .instrument(error_span!(
                    "request",
                    route = field::Empty,
                    referer = field::Empty
                ))
                .await
            },
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
