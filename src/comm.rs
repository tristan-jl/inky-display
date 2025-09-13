use crate::AppError;
use axum::debug_handler;
use axum::extract::State;
use axum::http::StatusCode;
use reqwest::Client;

#[debug_handler]
pub async fn health_check(State(client): State<Client>) -> Result<StatusCode, AppError> {
    let res = client
        .get("http://192.168.2.17:8080/api/control/check")
        .send()
        .await?;

    Ok(res.error_for_status()?.status())
}
