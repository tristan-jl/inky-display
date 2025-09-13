use anyhow::Result;
use axum::Router;
use axum::http::Request;
use axum::routing::get;
use headless_chrome::Browser;
use headless_chrome::LaunchOptions;
use headless_chrome::browser::default_executable;
use inky_display::AppError;
use inky_display::ServerAppState;
use inky_display::comm;
use inky_display::page;
use inky_display::render::process_image;
use inky_display::render::process_page;
use reqwest::Client;
use std::time::Duration;
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
                    "{}=debug,inky_display=debug,tower_http=debug,axum::rejection=trace",
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

    let state = ServerAppState {
        browser: Browser::new(launch_options)?,
        client: Client::new(),
    };

    let page_router = Router::new()
        .route("/page.html", get(page::page_handler))
        .route("/large_text.html", get(page::large_text_handler))
        .route("/text.html", get(page::text_handler));

    let generate_router = Router::new()
        .route("/page/{id}", get(process_page))
        .route("/image/{id}", get(process_image))
        .with_state(state.clone());

    let controller_router = Router::new()
        .route("/check", get(comm::health_check))
        .with_state(state.clone());

    let app = Router::new()
        .nest_service("/static", ServeDir::new("./static"))
        .nest_service("/image", ServeDir::new("./images"))
        .nest("/pages", page_router)
        .nest("/api/generate", generate_router)
        .nest("/api/control", controller_router)
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

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
    tracing::info!("listening on {}", listener.local_addr()?);
    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            _ = tokio::signal::ctrl_c().await;
            tracing::warn!("Initiating graceful shutdown");
        })
        .await?;

    Ok(())
}
