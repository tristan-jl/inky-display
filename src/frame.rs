use crate::controller::{Inky, LedState};
use crate::{AppError, FrameAppState};
use axum::body::Bytes;
use axum::debug_handler;
use axum::extract::State;
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

#[debug_handler]
pub async fn set_to_page(
    State(app_state): State<FrameAppState>,
    body: Bytes,
) -> Result<StatusCode, AppError> {
    let mut image = load_from_memory_with_format(&body, image::ImageFormat::Png)
        .expect("screenshot wasnt a png")
        .to_rgb8();

    let dims = image.dimensions();
    if dims != (Inky::WIDTH as u32, Inky::HEIGHT as u32) {
        return Err(AppError::InvalidInput(
            format!("Image was the incorrect dimensions: '{dims:?}'").into(),
        ));
    }

    let mut inky = app_state.inky.lock().expect("mutex poisoned");
    inky.set_display(&mut image, true)?;
    Ok(StatusCode::OK)
}

#[allow(clippy::unused_async)]
#[debug_handler]
pub async fn stripe(State(app_state): State<FrameAppState>) -> Result<StatusCode, AppError> {
    let mut inky = app_state.inky.lock().expect("mutex poisoned");
    inky.set_stripes()?;
    Ok(StatusCode::OK)
}
