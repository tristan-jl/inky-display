use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use anyhow::Result;
use axum::Router;
use axum::extract::FromRef;
use axum::http::Request;
use axum::routing::get;
use axum::routing::post;
use headless_chrome::Browser;
use headless_chrome::LaunchOptions;
use headless_chrome::browser::default_executable;
use inky_display::AppError;
use inky_display::AppState;
use inky_display::controller;
use inky_display::controller::Inky;
use inky_display::page;
use inky_display::render::process_image;
use inky_display::render::process_page;
use inky_display::wrap;
use tower::ServiceBuilder;
use tower_http::LatencyUnit;
use tower_http::services::ServeDir;
use tower_http::trace::DefaultOnRequest;
use tower_http::trace::DefaultOnResponse;
use tower_http::trace::TraceLayer;
use tracing::Level;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!(
                    "{}=debug,tower_http=debug,axum::rejection=trace",
                    env!("CARGO_CRATE_NAME")
                )
                .into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let launch_options = LaunchOptions::default_builder()
        .idle_browser_timeout(Duration::MAX)
        .path(Some(default_executable().map_err(|e| anyhow::anyhow!(e))?))
        .build()?;

    let state = AppState {
        browser: Browser::new(launch_options)?,
        // inky: Arc::new(Mutex::new(Inky::new()?)),
    };

    let generate_router = Router::new()
        .route("/page/{id}", get(process_page))
        .route("/image/{id}", get(process_image))
        .with_state(state.clone());

    // let pi_controller = Router::new()
    //     .route("/blink", post(wrap::blink))
    //     .route("/page/{page_path}", post(wrap::set_to_page))
    //     .with_state(state);

    let page_router = Router::new()
        .route("/page.html", get(page::page_handler))
        .route("/large_text.html", get(page::large_text_handler))
        .route("/text.html", get(page::text_handler));

    let app = Router::new()
        .nest_service("/static", ServeDir::new("./static"))
        .nest_service("/image", ServeDir::new("./images"))
        .nest("/pages", page_router)
        .nest("/api/generate", generate_router)
        // .nest("/api/control", pi_controller)
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
