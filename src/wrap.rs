use std::time::Duration;

use axum::debug_handler;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use headless_chrome::protocol::cdp::Page::{CaptureScreenshotFormatOption, Viewport};
use headless_chrome::protocol::cdp::Target::CreateTarget;
use image::load_from_memory_with_format;

use crate::controller::LedState;
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

#[allow(clippy::unused_async)]
#[debug_handler]
pub async fn set_to_page(
    State(app_state): State<AppState>,
    Path(page_path): Path<String>,
) -> Result<StatusCode, AppError> {
    let image = {
        let browser = app_state.browser;
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

    let (img_w, img_h) = screenshot_image.dimensions();
    screenshot_image.save("out.png").unwrap();

    let mut final_img = image::RgbImage::from_pixel(800, 480, [255, 255, 255].into());
    image::imageops::replace(
        &mut final_img,
        &screenshot_image,
        0,
        0,
        // ((800 - img_w) / 2).into(),
        // ((480 - img_h) / 2).into(),
    );

    let mut inky = app_state.inky.lock().expect("mutex poisoned");
    inky.set_display(&mut final_img, false)?;
    Ok(StatusCode::OK)
}
