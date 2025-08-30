use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Response;

use anyhow::Result;
use headless_chrome::Browser;
use headless_chrome::protocol::cdp::Page::CaptureScreenshotFormatOption;
use headless_chrome::protocol::cdp::Target::CreateTarget;

use crate::AppError;

pub async fn generate_screenshot(
    State(browser): State<Browser>,
    Path(id): Path<String>,
) -> Result<Response<Body>, AppError> {
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

    tab.navigate_to(&format!("http://localhost:8080/pages/{id}"))?;
    tab.wait_until_navigated()?;
    tab.wait_for_element("body")?;

    let element = tab.find_element("body")?;
    element.scroll_into_view()?;

    let screenshot =
        tab.capture_screenshot(CaptureScreenshotFormatOption::Png, Some(100), None, false)?;

    tab.close_with_unload()?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "image/png")
        .body(Body::from(screenshot))?)
}
