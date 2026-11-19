use crate::weather_station_client::WeatherStationClientError;
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum AppError {
    #[error("{0}")]
    Client(String),
    #[error("{0}")]
    Settings(String),
}

impl From<WeatherStationClientError> for AppError {
    fn from(error: WeatherStationClientError) -> Self {
        Self::Client(error.to_string())
    }
}

impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> Self {
        Self::Settings(error.to_string())
    }
}
