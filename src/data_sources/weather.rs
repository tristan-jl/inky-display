use crate::SERVER_CONFIG;
use chrono::NaiveDate;
use reqwest::Client;
use serde::Deserialize;

#[derive(Debug, Clone, Default)]
pub struct Weather {
    pub current_temp: i32,
    pub current_description: String,
    pub icon: String,
    pub forecast: Vec<DayForecast>,
}

#[derive(Debug, Clone)]
pub struct DayForecast {
    pub day: String,
    pub high: i32,
    pub low: i32,
    pub description: String,
    pub icon: String,
}

#[derive(Debug, Deserialize)]
struct OpenMeteoResponse {
    current: OpenMeteoCurrent,
    daily: OpenMeteoDaily,
}

#[derive(Debug, Deserialize)]
struct OpenMeteoCurrent {
    temperature_2m: f32,
    weather_code: u8,
}

#[derive(Debug, Deserialize)]
struct OpenMeteoDaily {
    time: Vec<NaiveDate>,
    weather_code: Vec<u8>,
    temperature_2m_max: Vec<f32>,
    temperature_2m_min: Vec<f32>,
}

pub fn get_weather_info(code: u8) -> (&'static str, &'static str) {
    match code {
        // Clear & Partly Cloudy
        0 => ("Clear sky", "clear-day.svg"),
        1 => ("Mainly clear", "mostly-clear-day.svg"),
        2 => ("Partly cloudy", "partly-cloudy-day.svg"),
        3 => ("Overcast", "cloudy.svg"),
        4..=9 | 30..=35 => ("Dust or haze", "sandstorm.svg"),
        10..=12 | 40..=49 => ("Fog", "fog.svg"),
        50..=55 => ("Drizzle", "drizzle.svg"),
        56..=57 => ("Freezing drizzle", "freezingdrizzle.svg"),
        60..=61 | 80 => ("Light rain", "rain.svg"),
        62..=63 | 81 => ("Moderate rain", "rain.svg"),
        64..=65 | 82 => ("Heavy rain", "rain.svg"),
        66..=67 => ("Freezing rain", "freezingrain.svg"),
        70..=71 | 85 => ("Light snow", "snow.svg"),
        72..=73 | 86 => ("Moderate snow", "snow.svg"),
        74..=75 => ("Heavy snow", "snow.svg"),
        77 | 36..=39 => ("Blowing snow", "blowingsnow.svg"),
        68..=69 | 83..=84 | 79 | 87..=88 => ("Sleet", "sleet.svg"),
        95..=96 => ("Thunderstorm", "thunderstorm.svg"),
        99 => ("Heavy thunderstorm", "thunderstorm-hail.svg"),
        19 => ("Tornado", "tornado.svg"),

        _ => ("Unknown", "unknown.svg"),
    }
}

pub async fn fetch_weather(client: &Client) -> Result<Weather, anyhow::Error> {
    let url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={}&longitude={}&current=temperature_2m,weather_code&daily=weather_code,temperature_2m_max,temperature_2m_min&timezone=Europe/London",
        SERVER_CONFIG.lat, SERVER_CONFIG.long
    );

    let response: OpenMeteoResponse = client.get(&url).send().await?.json().await?;
    dbg!(&response);
    let forecast: Vec<DayForecast> = response
        .daily
        .time
        .iter()
        .zip(response.daily.weather_code.iter())
        .zip(response.daily.temperature_2m_max.iter())
        .zip(response.daily.temperature_2m_min.iter())
        .skip(1) // Skip today
        .take(5) // Next 5 days
        .map(|(((date, &code), &high), &low)| {
            let (desc, icon) = get_weather_info(code);
            DayForecast {
                day: date.format("%a").to_string(),
                high: high.round() as i32,
                low: low.round() as i32,
                description: desc.to_string(),
                icon: icon.to_string(),
            }
        })
        .collect();

    let (desc, icon) = get_weather_info(response.current.weather_code);

    Ok(Weather {
        current_temp: response.current.temperature_2m.round() as i32,
        current_description: desc.to_string(),
        icon: icon.to_string(),
        forecast,
    })
}
