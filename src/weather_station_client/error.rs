use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum WeatherStationClientError {
    #[error("http client error: {0}")]
    Http(String),
    #[error("weather station returned {code}: {message}")]
    Response { code: u16, message: String },
}

impl From<reqwest::Error> for WeatherStationClientError {
    fn from(error: reqwest::Error) -> Self {
        Self::Http(error.to_string())
    }
}
