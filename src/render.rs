use std::io::Cursor;

use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Response;

use anyhow::{Context, Result};
use headless_chrome::Browser;
use headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption;
use headless_chrome::protocol::cdp::Target::CreateTarget;
use image::codecs::png::PngEncoder;
use image::{ImageEncoder, ImageReader, RgbImage, load_from_memory_with_format};

use crate::AppError;

fn resize_and_convert(input_image: &RgbImage) -> anyhow::Result<Vec<u8>> {
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

#[allow(clippy::unused_async)]
pub async fn process_page(
    State(browser): State<Browser>,
    Path(page_path): Path<String>,
) -> Result<Response<Body>, AppError> {
    let image = {
        let tab = browser.new_tab_with_options(CreateTarget {
            url: "about:blank".to_string(),
            width: Some(800),
            height: Some(480),
            browser_context_id: None,
            enable_begin_frame_control: None,
            new_window: Some(true),
            background: None,
            for_tab: None,
        })?;

        tab.navigate_to(&format!("http://localhost:8080/pages/{page_path}"))?;
        tab.wait_until_navigated()?;
        tab.wait_for_element("body")?;

        let element = tab.find_element("body")?;
        element.scroll_into_view()?;

        let screenshot =
            tab.capture_screenshot(CaptureScreenshotFormatOption::Png, Some(100), None, false)?;

        tab.close_with_unload()?;

        screenshot
    };
    let screenshot_image = load_from_memory_with_format(&image, image::ImageFormat::Png)
        .expect("screenshot wasnt a png")
        .to_rgb8();

    let img_bytes = resize_and_convert(&screenshot_image)?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "image/png")
        .body(Body::from(img_bytes))?)
}

#[allow(clippy::unused_async)]
pub async fn process_image(Path(image_path): Path<String>) -> Result<Response<Body>, AppError> {
    let img_path = format!("./images/{image_path}");
    let img = ImageReader::open(&img_path)
        .map_err(|e| {
            tracing::info!("Error reading image file at '{}': '{}'", img_path, e);
            AppError::InvalidInput("Unable to find image".into())
        })?
        .decode()
        .context("Couldn't decode image")?
        .resize(800, 480, image::imageops::FilterType::Lanczos3)
        .to_rgb8();

    let img_bytes = resize_and_convert(&img)?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "image/png")
        .body(Body::from(img_bytes))?)
}
