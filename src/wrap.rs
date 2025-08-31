use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::debug_handler;
use axum::extract::State;
use axum::http::StatusCode;

use crate::controller::{Inky, LedState};
use crate::{AppError, AppState};

#[debug_handler]
pub async fn blink(State(app_state): State<AppState>) -> Result<StatusCode, AppError> {
    let mut i = app_state.inky.lock().expect("mutex poisoned");
    for _ in 0..10 {
        std::thread::sleep(Duration::from_secs(1));
        i.set_led(LedState::On)?;
        std::thread::sleep(Duration::from_secs(1));
        i.set_led(LedState::Off)?;
    }
    Ok(StatusCode::OK)
}
