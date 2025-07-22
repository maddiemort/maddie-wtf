use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use thiserror::Error;
use tracing::debug;

use crate::{state::Theme, templates::pages};

/// Errors that can be returned by request handlers.
#[derive(Error, Clone, Debug)]
pub enum HandlerError {
    /// The requested page was not found.
    #[error("page not found")]
    NotFound,

    /// An internal server error occurred while trying to handle the request.
    #[error("internal server error")]
    InternalError,
}

/// `HandlerError` does implement [`IntoResponse`], so it can be returned from handlers as the error
/// type, but its implementation just injects the error enum into the extensions of the response.
///
/// This approach relies on the [`render_error()`] middleware being added to the stack, which will
/// extract the `HandlerError` and actually render it into a response page. It's split like this
/// because there's state that needs to be accessible when rendering the error (like the dynamic
/// colours).
impl IntoResponse for HandlerError {
    fn into_response(self) -> Response {
        let mut response = StatusCode::NOT_IMPLEMENTED.into_response();
        response.extensions_mut().insert(self);
        response
    }
}

/// Renders errors returned from handlers etc. by extracting the error value from the extensions of
/// the response.
///
/// This is done so that state can be accessed when rendering errors.
pub async fn render_error(
    State(theme): State<Theme>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let mut response = next.run(request).await;

    if let Some(handler_error) = response.extensions_mut().remove::<HandlerError>() {
        debug!(error = %handler_error, "rendering error");
        match handler_error {
            HandlerError::NotFound => {
                let mut response = pages::not_found(theme).await.into_response();
                *response.status_mut() = StatusCode::NOT_FOUND;
                response
            }
            HandlerError::InternalError => {
                let mut response = pages::internal_error(theme).await.into_response();
                *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
                response
            }
        }
    } else {
        response
    }
}
