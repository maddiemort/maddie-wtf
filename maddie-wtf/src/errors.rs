use axum::{http::StatusCode, response::IntoResponse};
use thiserror::Error;
use www::errors::RenderError;

use crate::{state::Theme, templates};

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

impl RenderError for HandlerError {
    type State = Theme;

    async fn status_code(&self) -> StatusCode {
        match self {
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    async fn response(&self, theme: Theme) -> impl IntoResponse {
        match self {
            Self::NotFound => templates::pages::not_found(theme).await,
            Self::InternalError => templates::pages::internal_error(theme).await,
        }
    }
}
