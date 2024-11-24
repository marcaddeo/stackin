use std::sync::Arc;

use crate::view::LowboyView;
use axum::{http::StatusCode, response::IntoResponse};

#[derive(Debug, thiserror::Error)]
pub enum LowboyError {
    #[error("Bad Request")]
    BadRequest,

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Forbidden")]
    Forbidden,

    #[error("Not Found")]
    NotFound,

    #[error("Internal Server Error: {0}")]
    Internal(#[from] anyhow::Error),
}

#[derive(Clone)]
pub(crate) struct ErrorWrapper(pub Arc<LowboyError>);

impl IntoResponse for LowboyError {
    fn into_response(self) -> axum::response::Response {
        use LowboyError::*;

        let code = match self {
            BadRequest => StatusCode::BAD_REQUEST,
            Unauthorized => StatusCode::UNAUTHORIZED,
            Forbidden => StatusCode::FORBIDDEN,
            NotFound => StatusCode::NOT_FOUND,
            Internal(ref inner) => {
                tracing::error!("{inner}");
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };

        let mut response = (code, "").into_response();
        response
            .extensions_mut()
            .insert(ErrorWrapper(Arc::new(self)));

        response
    }
}

pub trait LowboyErrorView: LowboyView + Clone + Default {
    fn message(&self) -> &String;
    fn set_message(&mut self, message: &str) -> &mut Self;
    fn code(&self) -> u16;
    fn set_code(&mut self, code: u16) -> &mut Self;
}