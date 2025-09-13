#![allow(clippy::unused_async)]
#![allow(clippy::diverging_sub_expression)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]
pub mod comm;
pub mod controller;
pub mod error;
pub mod frame;
pub mod page;

use crate::controller::Inky;
use anyhow::Context;
use axum::extract::FromRef;
pub use error::AppError;
use headless_chrome::Browser;
use image::codecs::png::PngEncoder;
use image::{ImageEncoder, RgbImage};
use reqwest::Client;
use std::io::Cursor;
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

#[derive(Debug)]
pub struct ServerConfig {
    pub port: u16,
    pub frame_url: String,
    pub static_root: String,
}

impl ServerConfig {
    fn new() -> Self {
        let port = std::env::var("PORT")
            .map(|i| i.parse().unwrap_or(8080))
            .unwrap_or(8080);

        let frame_url = std::env::var("FRAME_URL").expect("FRAME_URL was not set");
        let static_root = std::env::var("STATIC_ROOT").unwrap_or("./static".to_string());

        tracing::debug!("Connecting to frame at: '{frame_url}'");
        tracing::debug!("Using static folder: '{static_root}'");

        Self {
            port,
            frame_url,
            static_root,
        }
    }
}

pub static SERVER_CONFIG: std::sync::LazyLock<&'static ServerConfig> =
    std::sync::LazyLock::new(|| Box::leak(Box::new(ServerConfig::new())));

pub fn pad_and_convert(input_image: &RgbImage) -> anyhow::Result<Vec<u8>> {
    let (img_w, img_h) = input_image.dimensions();
    let mut final_img = image::RgbImage::from_pixel(
        Inky::WIDTH as u32,
        Inky::HEIGHT as u32,
        [255, 255, 255].into(),
    );
    image::imageops::replace(
        &mut final_img,
        input_image,
        (((Inky::WIDTH as u32).saturating_sub(img_w)) / 2).into(),
        (((Inky::HEIGHT as u32).saturating_sub(img_h)) / 2).into(),
    );

    let mut buffer = Cursor::new(Vec::new());
    let encoder = PngEncoder::new(&mut buffer);

    encoder
        .write_image(
            &final_img,
            final_img.width(),
            final_img.height(),
            image::ExtendedColorType::Rgb8,
        )
        .context("Failed to convert image to png bytes")?;

    Ok(buffer.into_inner())
}
