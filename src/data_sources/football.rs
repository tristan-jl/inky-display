use chrono::{DateTime, Local, NaiveDate, Utc};
use reqwest::Client;
use serde::Deserialize;

use crate::SERVER_CONFIG;

const FMT_STR: &str = "%d %b %H:%M";

#[derive(Debug, Clone, Default)]
pub struct Match {
    pub home_team: String,
    pub home_crest_url: String,
    pub away_team: String,
    pub away_crest_url: String,
    pub datetime_string: String,
    pub score: Option<String>,
    pub competition: String,
}

#[derive(Debug, Deserialize)]
struct FootballDataResponse {
    matches: Vec<FootballMatch>,
}

#[derive(Debug, Deserialize)]
struct FootballMatch {
    #[serde(rename = "homeTeam")]
    home_team: FootballTeam,
    #[serde(rename = "awayTeam")]
    away_team: FootballTeam,
    #[serde(rename = "utcDate")]
    utc_datetime: DateTime<Utc>,
    score: FootballScore,
    competition: FootballCompetition,
    status: String,
}

#[derive(Debug, Deserialize)]
struct FootballTeam {
    #[serde(rename = "shortName")]
    short_name: String,
    crest: String,
}

#[derive(Debug, Deserialize)]
struct FootballScore {
    #[serde(rename = "fullTime")]
    full_time: FootballScoreDetail,
}

#[derive(Debug, Deserialize)]
struct FootballScoreDetail {
    home: Option<u32>,
    away: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct FootballCompetition {
    code: String,
}

fn competition_code_to_name(code: &str) -> &'static str {
    match code {
        "PL" => "PL",
        "CL" => "CL",
        "ELC" => "EFL",
        "FAC" => "FA",
        "EFL" => "LC",
        "EC" => "Euro",
        "SA" => "SA",
        "BL1" => "BL",
        "FL1" => "L1",
        "PPL" => "PPL",
        "DED" => "ERE",
        "BSA" => "BRA",
        "WC" => "WC",
        _ => "Cup",
    }
}

fn format_match_date(utc_date: &str) -> String {
    // Parse ISO 8601 format like "2025-01-20T15:00:00Z" and extract "Jan 20"
    const MONTHS: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];

    let date_part = utc_date.split('T').next().unwrap_or(utc_date);
    let parts: Vec<&str> = date_part.split('-').collect();

    if parts.len() >= 3
        && let (Ok(month), Ok(day)) = (parts[1].parse::<usize>(), parts[2].parse::<u32>())
        && (1..=12).contains(&month)
    {
        return format!("{} {}", MONTHS[month - 1], day);
    }

    "???".to_string()
}

pub async fn fetch_arsenal_matches(client: &Client) -> Result<Vec<Match>, anyhow::Error> {
    // Fetch both scheduled and finished matches
    let now = Local::now().date_naive();
    let date_from = now - chrono::TimeDelta::days(7);
    let date_to = now + chrono::TimeDelta::days(7);
    let url = format!(
        "https://api.football-data.org/v4/teams/57/matches?dateFrom={date_from}&dateTo={date_to}&status=SCHEDULED,FINISHED&limit=10"
    );

    let response: FootballDataResponse = client
        .get(url)
        .header("X-Auth-Token", SERVER_CONFIG.football_api_key.clone())
        .send()
        .await?
        .json()
        .await?;

    // Separate finished and scheduled matches
    let mut finished: Vec<_> = response
        .matches
        .iter()
        .filter(|m| m.status == "FINISHED")
        .collect();
    let mut scheduled: Vec<_> = response
        .matches
        .iter()
        .filter(|m| m.status == "SCHEDULED" || m.status == "TIMED")
        .collect();

    // Sort finished by date descending (most recent first)
    finished.sort_by(|a, b| b.utc_datetime.cmp(&a.utc_datetime));
    // Sort scheduled by date ascending (nearest first)
    scheduled.sort_by(|a, b| a.utc_datetime.cmp(&b.utc_datetime));

    // Take last 2 finished + next 3 scheduled = 5 matches
    let mut matches: Vec<Match> = Vec::new();

    for m in finished.into_iter().take(2).rev() {
        matches.push(Match {
            home_team: m.home_team.short_name.clone(),
            home_crest_url: m.home_team.crest.clone(),
            away_team: m.away_team.short_name.clone(),
            away_crest_url: m.away_team.crest.clone(),
            datetime_string: m.utc_datetime.format(FMT_STR).to_string(),
            score: Some(format!(
                "{}-{}",
                m.score.full_time.home.unwrap_or(0),
                m.score.full_time.away.unwrap_or(0)
            )),
            competition: competition_code_to_name(&m.competition.code).to_string(),
        });
    }

    for m in scheduled.into_iter().take(3) {
        matches.push(Match {
            home_team: m.home_team.short_name.clone(),
            home_crest_url: m.home_team.crest.clone(),
            away_team: m.away_team.short_name.clone(),
            away_crest_url: m.away_team.crest.clone(),
            datetime_string: m.utc_datetime.format(FMT_STR).to_string(),
            score: None,
            competition: competition_code_to_name(&m.competition.code).to_string(),
        });
    }

    Ok(matches)
}
