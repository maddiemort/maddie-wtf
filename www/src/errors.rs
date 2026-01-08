use core::fmt::Display;
use std::future::Future;

use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use tracing::debug;

pub trait RenderError: Clone + Display + Send + Sync + 'static {
    type State;

    fn status_code(&self) -> impl Future<Output = StatusCode>;

    fn response(&self, state: Self::State) -> impl Future<Output = impl IntoResponse>;
}

#[derive(Clone)]
pub struct Render<E>(E);

impl<E> From<E> for Render<E>
where
    E: RenderError,
{
    fn from(error: E) -> Self {
        Self(error)
    }
}

/// `HandlerError` does implement [`IntoResponse`], so it can be returned from handlers as the
/// error type, but its implementation just injects the error enum into the extensions of the
/// response.
///
/// This approach relies on the [`render_error()`] middleware being added to the stack, which will
/// extract the `HandlerError` and actually render it into a response page. It's split like this
/// because there's state that needs to be accessible when rendering the error (like the dynamic
/// colours).
impl<E> IntoResponse for Render<E>
where
    E: RenderError,
{
    fn into_response(self) -> Response {
        let mut response = StatusCode::NOT_IMPLEMENTED.into_response();
        response.extensions_mut().insert(self);
        response
    }
}

/// Creates middleware that renders errors returned from handlers etc. by extracting the error
/// value from the extensions of the response.
///
/// This is done so that state can be accessed when rendering errors.
pub async fn render_error<E>(
    State(state): State<E::State>,
    request: Request<Body>,
    next: Next,
) -> Response
where
    E: RenderError,
{
    let mut response = next.run(request).await;

    if let Some(error) = response.extensions_mut().remove::<Render<E>>() {
        debug!(error = %error.0, "rendering error");
        let mut response = error.0.response(state).await.into_response();
        *response.status_mut() = error.0.status_code().await;
        response
    } else {
        response
    }
}
