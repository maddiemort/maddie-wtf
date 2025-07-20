use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, Request, Response},
};
use maud::Markup;
use tracing::{debug, warn};

use crate::{
    errors::HandlerError,
    state::{Content, Settings, Theme},
    templates::pages,
};

const STYLESHEET: &str = include_str!(concat!(env!("OUT_DIR"), "/style.css"));

pub async fn index(
    State(content): State<Content>,
    State(theme): State<Theme>,
    request: Request<Body>,
) -> Result<Markup, HandlerError> {
    debug!(route = %request.uri(), "handling request");

    if let Some(page) = content.page("_index").await {
        Ok(pages::page(page, theme).await)
    } else {
        Err(not_found(request).await)
    }
}

pub async fn posts(
    State(content): State<Content>,
    State(theme): State<Theme>,
    State(settings): State<Settings>,
    request: Request<Body>,
) -> Result<Markup, HandlerError> {
    debug!(route = %request.uri(), "handling request");

    let posts = content.nodes(settings.show_drafts()).await.into_posts();
    Ok(pages::posts(posts, theme).await)
}

pub async fn post(
    State(content): State<Content>,
    State(theme): State<Theme>,
    State(settings): State<Settings>,
    Path(post): Path<String>,
    request: Request<Body>,
) -> Result<Markup, HandlerError> {
    debug!(route = %request.uri(), "handling request");

    if let Some(post) = content.post(post, settings.show_drafts()).await {
        Ok(pages::post(post, theme).await)
    } else {
        Err(not_found(request).await)
    }
}

pub async fn chrono(
    State(content): State<Content>,
    State(theme): State<Theme>,
    State(settings): State<Settings>,
    request: Request<Body>,
) -> Result<Markup, HandlerError> {
    debug!(route = %request.uri(), "handling request");

    let posts = content.nodes(settings.show_drafts()).await.into_chrono();
    Ok(pages::chrono(posts, theme).await)
}

pub async fn stylesheet(request: Request<Body>) -> Result<Response<String>, HandlerError> {
    debug!(route = %request.uri(), "handling request");

    Response::builder()
        .header(header::CONTENT_TYPE, "text/css")
        .body(STYLESHEET.to_owned())
        .map_err(|_| HandlerError::InternalError)
}

pub async fn not_found(request: Request<Body>) -> HandlerError {
    warn!(route = %request.uri(), "request received for unknown URI");
    HandlerError::NotFound
}

#[cfg(debug_assertions)]
pub async fn internal_error(request: Request<Body>) -> HandlerError {
    warn!(route = %request.uri(), "internal error page explicitly requested");
    HandlerError::InternalError
}
