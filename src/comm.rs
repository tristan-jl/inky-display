use crate::controller::Inky;
use crate::{AppError, SERVER_CONFIG, ServerAppState, pad_and_convert};
use anyhow::{Context, Result};
use axum::debug_handler;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption;
use headless_chrome::protocol::cdp::Page::SetDeviceMetricsOverride;
use headless_chrome::protocol::cdp::Target::CreateTarget;
use image::{ImageReader, load_from_memory_with_format};
use reqwest::Client;

#[debug_handler]
pub async fn health_check(State(client): State<Client>) -> Result<StatusCode, AppError> {
    let res = client
        .get(format!("{}/api/control/check", &SERVER_CONFIG.frame_url))
        .send()
        .await?;

    Ok(res.error_for_status()?.status())
}

#[debug_handler]
pub async fn set_to_image(
    State(client): State<Client>,
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
        .resize(
            Inky::WIDTH as u32,
            Inky::HEIGHT as u32,
            image::imageops::FilterType::Lanczos3,
        )
        .to_rgb8();

    let b = pad_and_convert(&image)?;
    let res = client
        .post(format!("{}/api/control/set", &SERVER_CONFIG.frame_url))
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
            width: Some(Inky::WIDTH as u32),
            height: Some(Inky::HEIGHT as u32),
            browser_context_id: None,
            enable_begin_frame_control: None,
            new_window: Some(true),
            background: None,
            for_tab: None,
        })?;

        tab.navigate_to(&format!(
            "http://localhost:{}/pages/{page_path}",
            &SERVER_CONFIG.port
        ))?;
        tab.wait_until_navigated()?;
        tab.wait_for_element("body")?;
        tab.call_method(SetDeviceMetricsOverride {
            width: Inky::WIDTH as u32,
            height: Inky::HEIGHT as u32,
            device_scale_factor: 1.0,
            mobile: false,
            scale: Some(1.0),
            screen_width: Some(Inky::WIDTH as u32),
            screen_height: Some(Inky::HEIGHT as u32),
            position_x: Some(0),
            position_y: Some(0),
            dont_set_visible_size: Some(false),
            screen_orientation: None,
            viewport: None,
        })?;

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
        .post(format!("{}/api/control/set", &SERVER_CONFIG.frame_url))
        .body(b)
        .send()
        .await?;

    Ok(res.error_for_status()?.status())
}

#[debug_handler]
pub async fn set_to_stripes(State(state): State<ServerAppState>) -> Result<StatusCode, AppError> {
    let res = state
        .client
        .get(format!("{}/api/control/stripe", &SERVER_CONFIG.frame_url))
        .send()
        .await?;

    Ok(res.error_for_status()?.status())
}
