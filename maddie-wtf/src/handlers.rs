use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, Request, Response},
};
use maud::Markup;
use tap::TryConv;
use tracing::warn;

use crate::{
    errors::HandlerError,
    state::{names::TagName, Content, Settings, Theme},
    templates::pages,
};

const STYLESHEET: &str = include_str!(concat!(env!("OUT_DIR"), "/style.css"));

pub async fn index(
    State(content): State<Content>,
    State(theme): State<Theme>,
    State(settings): State<Settings>,
    request: Request<Body>,
) -> Result<Markup, HandlerError> {
    let recent_posts = content
        .nodes(settings.show_drafts())
        .await
        .into_recent_pubs();
    if let Some(index) = content.page("_index").await {
        Ok(pages::index(index, recent_posts, theme).await)
    } else {
        Err(not_found(request).await)
    }
}

pub async fn page(
    State(content): State<Content>,
    State(theme): State<Theme>,
    Path(page): Path<String>,
    request: Request<Body>,
) -> Result<Markup, HandlerError> {
    if let Some(page) = content.page(page).await {
        Ok(pages::page(page, theme).await)
    } else {
        Err(not_found(request).await)
    }
}

pub async fn posts(
    State(content): State<Content>,
    State(theme): State<Theme>,
    State(settings): State<Settings>,
    _request: Request<Body>,
) -> Result<Markup, HandlerError> {
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
    if let Some(post) = content.post(post, settings.show_drafts()).await {
        Ok(pages::post(post, theme).await)
    } else {
        Err(not_found(request).await)
    }
}

pub async fn entry(
    State(content): State<Content>,
    State(theme): State<Theme>,
    State(settings): State<Settings>,
    Path((post, index)): Path<(String, usize)>,
    request: Request<Body>,
) -> Result<Markup, HandlerError> {
    if let Some(entry) = content
        .post(post, settings.show_drafts())
        .await
        .and_then(|p| p.into_entry(index, settings.show_drafts()))
    {
        Ok(pages::entry(entry, theme).await)
    } else {
        Err(not_found(request).await)
    }
}

pub async fn chrono(
    State(content): State<Content>,
    State(theme): State<Theme>,
    State(settings): State<Settings>,
    _request: Request<Body>,
) -> Result<Markup, HandlerError> {
    let posts = content.nodes(settings.show_drafts()).await.into_chrono();
    Ok(pages::chrono(posts, theme).await)
}

pub async fn tags(
    State(content): State<Content>,
    State(theme): State<Theme>,
    State(settings): State<Settings>,
    _request: Request<Body>,
) -> Result<Markup, HandlerError> {
    let posts = content.nodes(settings.show_drafts()).await.into_tags();
    Ok(pages::tags(posts, theme).await)
}

pub async fn tagged(
    State(content): State<Content>,
    State(theme): State<Theme>,
    State(settings): State<Settings>,
    Path(tag): Path<String>,
    _request: Request<Body>,
) -> Result<Markup, HandlerError> {
    match tag.try_conv::<TagName>() {
        Ok(tag) => {
            if content.tag_exists(&tag).await {
                let posts = content.nodes(settings.show_drafts()).await.into_tagged(tag);
                Ok(pages::tagged(posts, theme).await)
            } else {
                warn!(%tag, "requested tag doesn't exist");
                Err(HandlerError::NotFound)
            }
        }
        Err(error) => {
            warn!(%error, "requested tag is invalid");
            Err(HandlerError::NotFound)
        }
    }
}

pub async fn stylesheet(_request: Request<Body>) -> Result<Response<String>, HandlerError> {
    Response::builder()
        .header(header::CONTENT_TYPE, "text/css")
        .body(STYLESHEET.to_owned())
        .map_err(|_| HandlerError::InternalError)
}

pub async fn rss_feed(
    State(content): State<Content>,
    State(settings): State<Settings>,
    _request: Request<Body>,
) -> Result<Response<String>, HandlerError> {
    let feed = content.nodes(settings.show_drafts()).await.into_rss_feed();
    let feed_output = pages::rss_feed(feed).await;

    Response::builder()
        .header(header::CONTENT_TYPE, "application/rss+xml")
        .body(feed_output.into_string())
        .map_err(|_| HandlerError::InternalError)
}

pub async fn not_found(_request: Request<Body>) -> HandlerError {
    HandlerError::NotFound
}

#[cfg(debug_assertions)]
pub async fn internal_error(request: Request<Body>) -> HandlerError {
    warn!(route = %request.uri(), "internal error page explicitly requested");
    HandlerError::InternalError
}
