mod football;
mod tube;
mod weather;

pub use football::{Match, fetch_arsenal_matches};
pub use tube::{LineStatus, fetch_tube_status};
pub use weather::{DayForecast, Weather, fetch_weather};
