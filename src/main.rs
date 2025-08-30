pub mod page;
pub mod render;

use self::render::generate_screenshot;
use anyhow::Result;
use axum::extract::FromRef;
use axum::http::Request;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use headless_chrome::Browser;
use serde::Serialize;
use std::borrow::Cow;
use tower::ServiceBuilder;
use tower_http::LatencyUnit;
use tower_http::services::ServeDir;
use tower_http::trace::DefaultOnRequest;
use tower_http::trace::DefaultOnResponse;
use tower_http::trace::TraceLayer;
use tracing::Level;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .init();

    let state = AppState {
        browser: Browser::default()?,
    };

    let generate_router = Router::new()
        .route("/{id}", get(generate_screenshot))
        .with_state(state);

    let file_router = Router::new()
        .route("/page.html", get(page::page_handler))
        .route("/text.html", get(page::text_handler));

    let app = Router::new()
        .nest_service("/static", ServeDir::new("./static"))
        .nest("/pages", file_router)
        .nest("/api/generate", generate_router)
        .fallback(|| async { AppError::NotFound })
        .layer(
            ServiceBuilder::new().layer(
                TraceLayer::new_for_http()
                    .make_span_with(|request: &Request<_>| {
                        tracing::info_span!(
                            "http_request",
                            uri = %request.uri(),
                            method = %request.method(),
                        )
                    })
                    .on_request(DefaultOnRequest::new().level(Level::INFO))
                    .on_response(
                        DefaultOnResponse::new()
                            .level(Level::INFO)
                            .latency_unit(LatencyUnit::Micros),
                    ),
            ),
        );

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await?;
    tracing::info!("listening on {}", listener.local_addr()?);
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            _ = tokio::signal::ctrl_c().await;
            tracing::warn!("Initiating graceful shutdown");
        })
        .await?;

    Ok(())
}

#[derive(Clone)]
pub struct AppState {
    pub browser: Browser,
}

impl FromRef<AppState> for Browser {
    fn from_ref(app_state: &AppState) -> Browser {
        app_state.browser.clone()
    }
}

#[derive(Debug, thiserror::Error)]
enum AppError {
    #[error("Not found")]
    NotFound,

    #[error("Error in request")]
    InvalidInput(Cow<'static, str>),

    #[error("Axum http error")]
    AxumHttp(#[from] axum::http::Error),

    #[error("Error loading page")]
    Render(#[from] askama::Error),

    #[error("An internal error occured")]
    Anyhow(#[from] anyhow::Error),
}

impl AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::InvalidInput(_) => StatusCode::UNPROCESSABLE_ENTITY,
            AppError::AxumHttp(_) | AppError::Render(_) | AppError::Anyhow(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
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
            AppError::Anyhow(ref e) => {
                tracing::error!("Generic error: {:?}", e);
            }
            _ => (),
        }

        (self.status_code(), self.to_string()).into_response()
    }
}
