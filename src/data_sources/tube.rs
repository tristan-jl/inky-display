use reqwest::Client;
use serde::Deserialize;

use crate::SERVER_CONFIG;

#[derive(Debug, Clone, Default)]
pub struct LineStatus {
    pub name: String,
    pub status: String,
    pub reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TflLineResponse {
    name: String,
    #[serde(rename = "lineStatuses")]
    line_statuses: Vec<TflLineStatus>,
}

#[derive(Debug, Deserialize)]
struct TflLineStatus {
    #[serde(rename = "statusSeverityDescription")]
    status_severity_description: String,
    reason: Option<String>,
}

pub async fn fetch_tube_status(client: &Client) -> Result<Vec<LineStatus>, anyhow::Error> {
    let url = format!(
        "https://api.tfl.gov.uk/Line/Mode/tube/Status?app_key={}",
        SERVER_CONFIG.tube_api_key
    );
    let response: Vec<TflLineResponse> = client.get(&url).send().await?.json().await?;

    let lines = response
        .into_iter()
        .map(|line| {
            let status = line.line_statuses.first().map_or_else(
                || "Unknown".to_string(),
                |s| s.status_severity_description.clone(),
            );
            let reason = line.line_statuses.first().and_then(|s| s.reason.clone());

            LineStatus {
                name: line.name,
                status,
                reason,
            }
        })
        .collect();

    Ok(lines)
}
