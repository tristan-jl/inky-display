#![allow(clippy::diverging_sub_expression)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]
pub mod comm;
pub mod controller;
pub mod error;
pub mod frame;
pub mod page;
pub mod render;

use axum::extract::FromRef;
pub use error::AppError;
use headless_chrome::Browser;
use reqwest::Client;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct ServerAppState {
    // both already wrapped in Arcs
    pub browser: Browser,
    pub client: Client,
}

impl FromRef<ServerAppState> for Browser {
    fn from_ref(app_state: &ServerAppState) -> Browser {
        app_state.browser.clone()
    }
}

impl FromRef<ServerAppState> for Client {
    fn from_ref(app_state: &ServerAppState) -> Client {
        app_state.client.clone()
    }
}

#[derive(Clone)]
pub struct FrameAppState {
    pub inky: Arc<Mutex<controller::Inky>>,
}

impl FromRef<FrameAppState> for Arc<Mutex<controller::Inky>> {
    fn from_ref(app_state: &FrameAppState) -> Arc<Mutex<controller::Inky>> {
        app_state.inky.clone()
    }
}
