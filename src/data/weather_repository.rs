use crate::domain::error::AppError;
use weather_station_client::{Device, Measurement, WeatherStationClient};

#[derive(Debug, Clone)]
pub struct WeatherStationRepository {
    client: WeatherStationClient,
}

impl WeatherStationRepository {
    pub fn new(base_url: impl Into<String>) -> Result<Self, AppError> {
        Ok(Self {
            client: WeatherStationClient::new(base_url)?,
        })
    }

    pub async fn devices(&self) -> Result<Vec<Device>, AppError> {
        Ok(self.client.get_device_list().await?)
    }

    pub async fn measurement(&self, device_id: i32) -> Result<Option<Measurement>, AppError> {
        Ok(self.client.get_measurement(device_id).await?)
    }
}
