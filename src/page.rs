use crate::AppError;
use crate::data_sources;
use askama::Template;
use axum::extract::State;
use axum::response::Html;
use reqwest::Client;

pub async fn large_text_handler() -> Result<Html<String>, AppError> {
    #[derive(Debug, Template)]
    #[template(path = "large_text.html")]
    struct Tmpl {
        words: String,
    }

    let template = Tmpl {
        words: "Hello there".to_string(),
    };
    Ok(Html(template.render()?))
}

pub async fn dashboard_handler(State(client): State<Client>) -> Result<Html<String>, AppError> {
    #[derive(Debug, Template)]
    #[template(path = "dashboard.html")]
    struct DashboardTmpl {
        tube_lines: Vec<data_sources::LineStatus>,
        has_disruptions: bool,
        weather: data_sources::Weather,
        matches: Vec<data_sources::Match>,
    }

    let (tube_result, weather_result, arsenal_result) = tokio::join!(
        data_sources::fetch_tube_status(&client),
        data_sources::fetch_weather(&client),
        data_sources::fetch_arsenal_matches(&client),
    );

    let tube_lines = tube_result.unwrap_or_else(|e| {
        tracing::error!("Failed to fetch TfL status: {}", e);
        Vec::new()
    });

    // Filter to only show lines with disruptions
    let disrupted_lines: Vec<data_sources::LineStatus> = tube_lines
        .iter()
        .filter(|l| l.status != "Good Service")
        .cloned()
        .collect();

    let has_disruptions = !disrupted_lines.is_empty();

    let weather = weather_result.unwrap_or_else(|e| {
        tracing::error!("Failed to fetch weather: {}", e);
        data_sources::Weather::default()
    });

    let matches = arsenal_result.unwrap_or_else(|e| {
        tracing::error!("Failed to fetch Arsenal matches: {}", e);
        Vec::new()
    });

    let template = DashboardTmpl {
        tube_lines: if has_disruptions {
            disrupted_lines
        } else {
            tube_lines
        },
        has_disruptions,
        weather,
        matches,
    };

    Ok(Html(template.render()?))
}
