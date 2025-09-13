use axum::Json;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde::Serialize;
use std::borrow::Cow;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Not found")]
    NotFound,

    #[error("Error in request")]
    InvalidInput(Cow<'static, str>),

    #[error("Axum http error")]
    AxumHttp(#[from] axum::http::Error),

    #[error("Error loading page")]
    Render(#[from] askama::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("GPIO error: {0}")]
    Gpio(#[from] gpio_cdev::errors::Error),

    #[error("Request to frame failed")]
    RequestError(#[from] reqwest::Error),

    #[error("An internal error occured")]
    Anyhow(#[from] anyhow::Error),
}

impl AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::InvalidInput(_) => StatusCode::UNPROCESSABLE_ENTITY,
            AppError::AxumHttp(_)
            | AppError::Render(_)
            | AppError::Io(_)
            | AppError::Gpio(_)
            | AppError::RequestError(_)
            | AppError::Anyhow(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        match self {
            AppError::InvalidInput(ref e) => {
                #[derive(Debug, Serialize)]
                struct ErrorDetail<'a> {
                    detail: &'a Cow<'static, str>,
                }

                return (self.status_code(), Json(ErrorDetail { detail: e })).into_response();
            }
            AppError::RequestError(ref e) => {
                tracing::error!("Request error: {:?}", e);
            }
            AppError::Anyhow(ref e) => {
                tracing::error!("Generic error: {:?}", e);
            }
            _ => (),
        }

        (self.status_code(), self.to_string()).into_response()
    }
}
