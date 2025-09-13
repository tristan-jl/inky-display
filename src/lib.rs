#![allow(clippy::diverging_sub_expression)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]
pub mod controller;
pub mod error;
pub mod page;
pub mod render;
pub mod wrap;

use std::sync::{Arc, Mutex};

use axum::extract::FromRef;
pub use error::AppError;
use headless_chrome::Browser;

#[derive(Clone)]
pub struct AppState {
    pub browser: Browser,
    // pub inky: Arc<Mutex<controller::Inky>>,
}

impl FromRef<AppState> for Browser {
    fn from_ref(app_state: &AppState) -> Browser {
        app_state.browser.clone()
    }
}

// impl FromRef<AppState> for Arc<Mutex<controller::Inky>> {
//     fn from_ref(app_state: &AppState) -> Arc<Mutex<controller::Inky>> {
//         app_state.inky.clone()
//     }
// }
