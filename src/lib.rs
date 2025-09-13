#![allow(clippy::unused_async)]
#![allow(clippy::diverging_sub_expression)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::missing_errors_doc)]
pub mod comm;
pub mod controller;
pub mod error;
pub mod frame;
pub mod page;

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
    pub frame_url: String,
}

impl FromRef<ServerAppState> for Browser {
    fn from_ref(app_state: &ServerAppState) -> Browser {
        app_state.browser.clone()
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

pub fn pad_and_convert(input_image: &RgbImage) -> anyhow::Result<Vec<u8>> {
    let (img_w, img_h) = input_image.dimensions();
    let mut final_img = image::RgbImage::from_pixel(800, 480, [255, 255, 255].into());
    image::imageops::replace(
        &mut final_img,
        input_image,
        ((800_u32.saturating_sub(img_w)) / 2).into(),
        ((480_u32.saturating_sub(img_h)) / 2).into(),
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
