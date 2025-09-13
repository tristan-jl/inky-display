use crate::{AppError, ServerAppState, pad_and_convert};
use anyhow::{Context, Result};
use axum::debug_handler;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption;
use headless_chrome::protocol::cdp::Target::CreateTarget;
use image::{ImageReader, load_from_memory_with_format};

#[debug_handler]
pub async fn health_check(State(state): State<ServerAppState>) -> Result<StatusCode, AppError> {
    let res = state
        .client
        .get(format!("{}/api/control/check", state.frame_url))
        .send()
        .await?;

    Ok(res.error_for_status()?.status())
}

#[debug_handler]
pub async fn set_to_image(
    State(state): State<ServerAppState>,
    Path(image_path): Path<String>,
) -> Result<StatusCode, AppError> {
    let image_path = format!("./images/{image_path}");
    let image = ImageReader::open(&image_path)
        .map_err(|e| {
            tracing::info!("Error reading image file at '{}': '{}'", image_path, e);
            AppError::InvalidInput("Unable to find image".into())
        })?
        .decode()
        .context("Couldn't decode image")?
        .resize(800, 480, image::imageops::FilterType::Lanczos3)
        .to_rgb8();
    let b = pad_and_convert(&image)?;

    let res = state
        .client
        .post(format!("{}/api/control/set", state.frame_url))
        .body(b)
        .send()
        .await?;

    Ok(res.error_for_status()?.status())
}

#[allow(clippy::unused_async)]
pub async fn set_to_page(
    State(state): State<ServerAppState>,
    Path(page_path): Path<String>,
) -> Result<StatusCode, AppError> {
    let image = {
        let tab = state.browser.new_tab_with_options(CreateTarget {
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
    let image = load_from_memory_with_format(&image, image::ImageFormat::Png)
        .expect("screenshot wasnt a png")
        .to_rgb8();

    let b = pad_and_convert(&image)?;

    let res = state
        .client
        .post(format!("{}/api/control/set", state.frame_url))
        .body(b)
        .send()
        .await?;

    Ok(res.error_for_status()?.status())
}

#[debug_handler]
pub async fn set_to_stripes(State(state): State<ServerAppState>) -> Result<StatusCode, AppError> {
    let res = state
        .client
        .get(format!("{}/api/control/stripe", state.frame_url))
        .send()
        .await?;

    Ok(res.error_for_status()?.status())
}
