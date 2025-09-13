use anyhow::Result;
use axum::Router;
use axum::http::Request;
use axum::routing::get;
use axum::routing::post;
use inky_display::AppError;
use inky_display::FrameAppState;
use inky_display::controller::Inky;
use inky_display::frame;
use std::sync::Arc;
use std::sync::Mutex;
use tower::ServiceBuilder;
use tower_http::LatencyUnit;
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

    let state = FrameAppState {
        inky: Arc::new(Mutex::new(Inky::new()?)),
    };

    let pi_controller = Router::new()
        .route("/check", get(frame::health_check))
        .route("/blink", post(frame::blink))
        .route("/set", post(frame::set_to_page))
        .with_state(state);

    let app = Router::new()
        .nest("/api/control", pi_controller)
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
