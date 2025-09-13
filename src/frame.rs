use crate::controller::LedState;
use crate::{AppError, FrameAppState};
use anyhow::Context;
use axum::debug_handler;
use axum::extract::{Multipart, State};
use axum::http::StatusCode;
use image::load_from_memory_with_format;
use std::time::Duration;

#[debug_handler]
pub async fn health_check(State(app_state): State<FrameAppState>) -> Result<StatusCode, AppError> {
    let mut i = app_state.inky.lock().expect("mutex poisoned");
    i.set_led(LedState::On)?;
    std::thread::sleep(Duration::from_secs(1));
    i.set_led(LedState::Off)?;
    Ok(StatusCode::OK)
}

#[debug_handler]
pub async fn blink(State(app_state): State<FrameAppState>) -> Result<StatusCode, AppError> {
    let mut i = app_state.inky.lock().expect("mutex poisoned");
    for _ in 0..10 {
        std::thread::sleep(Duration::from_secs(1));
        i.set_led(LedState::On)?;
        std::thread::sleep(Duration::from_secs(1));
        i.set_led(LedState::Off)?;
    }
    Ok(StatusCode::OK)
}

#[allow(clippy::unused_async)]
#[debug_handler]
pub async fn set_to_page(
    State(app_state): State<FrameAppState>,
    mut multipart: Multipart,
) -> Result<StatusCode, AppError> {
    let form_field = {
        let field = multipart
            .next_field()
            .await
            .map_err(|e| {
                tracing::warn!("Got multipart error during set to page: '{e}'");
                AppError::InvalidInput("Multipart form error".into())
            })?
            .context("No field provided")?;
        let name = field.name().unwrap_or("<NO FIELD NAME>").to_string();
        let data = field.bytes().await.unwrap();

        tracing::info!("Length of `{}` is {} bytes", name, data.len());
        data
    };

    let mut image = load_from_memory_with_format(&form_field, image::ImageFormat::Png)
        .expect("screenshot wasnt a png")
        .to_rgb8();

    let dims = image.dimensions();
    if dims != (800, 480) {
        return Err(AppError::InvalidInput(
            format!("Image was the incorrect dimensions: '{dims:?}'").into(),
        ));
    }

    let mut inky = app_state.inky.lock().expect("mutex poisoned");
    inky.set_display(&mut image, false)?;
    Ok(StatusCode::OK)
}
