use crate::error::WeatherStationClientError;
use crate::models::{Device, DeviceDailyErrors, Measurement};
use reqwest::Client;
use serde::de::DeserializeOwned;
use std::time::Duration;

const REQUEST_TIMEOUT: Duration = Duration::from_secs(3);

#[derive(Debug, Clone)]
pub struct WeatherStationClient {
    base_url: String,
    http: Client,
}

impl WeatherStationClient {
    pub fn new(base_url: impl Into<String>) -> Result<Self, WeatherStationClientError> {
        let http = Client::builder().timeout(REQUEST_TIMEOUT).build()?;

        Ok(Self {
            base_url: base_url.into().trim_end_matches('/').to_owned(),
            http,
        })
    }

    pub async fn get_device_list(&self) -> Result<Vec<Device>, WeatherStationClientError> {
        self.get("/devices").await
    }

    pub async fn get_measurement(
        &self,
        device_id: i32,
    ) -> Result<Option<Measurement>, WeatherStationClientError> {
        self.get(&format!("/last_measurement/{device_id}?additional=true"))
            .await
    }

    pub async fn get_device_list_daily_errors(
        &self,
    ) -> Result<Vec<DeviceDailyErrors>, WeatherStationClientError> {
        self.get("/device_analytics").await
    }

    pub async fn get_device_daily_errors(
        &self,
        device_id: i32,
    ) -> Result<Option<DeviceDailyErrors>, WeatherStationClientError> {
        self.get(&format!("/device_analytics/{device_id}")).await
    }

    async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, WeatherStationClientError> {
        let url = format!("{}{}", self.base_url, path);
        let response = self.http.get(url).send().await?;
        let status = response.status();

        if !status.is_success() {
            let message = response.text().await.unwrap_or_default();
            return Err(WeatherStationClientError::Response {
                code: status.as_u16(),
                message,
            });
        }

        Ok(response.json().await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DailyCalculation, Measure};
    use std::io::{Read, Write};
    use std::net::{TcpListener, TcpStream};
    use std::thread;

    #[tokio::test]
    async fn loads_device_list() {
        let server = TestServer::new(
            "GET /devices HTTP/1.1",
            r#"[{"id":1,"type":"esp","name":"Kitchen node","location":"Kitchen"}]"#,
        );
        let client = WeatherStationClient::new(server.base_url()).unwrap();

        let devices = client.get_device_list().await.unwrap();

        assert_eq!(
            devices,
            vec![Device {
                id: 1,
                device_type: Some("esp".to_owned()),
                name: Some("Kitchen node".to_owned()),
                location: Some("Kitchen".to_owned()),
            }]
        );
    }

    #[tokio::test]
    async fn loads_measurement_with_daily_calculations() {
        let server = TestServer::new(
            "GET /last_measurement/7?additional=true HTTP/1.1",
            r#"{
                "timestamp":"2026-11-16T10:15:30.000+0300",
                "timeZoneHours":3,
                "device":{"id":7,"type":null,"name":"Outdoor","location":null},
                "measures":[{
                    "sensorId":"dht-1",
                    "sensorName":"DHT22",
                    "sensorPlace":"balcony",
                    "measureName":"Temperature",
                    "measureValue":21.5,
                    "measureUnit":"C",
                    "sensorError":false,
                    "dailyCalculation":{
                        "maxValue":22.0,
                        "minValue":19.0,
                        "averageValue":20.5,
                        "maxValueTime":"2026-11-16T14:00:00.000+0300",
                        "minValueTime":null
                    }
                }]
            }"#,
        );
        let client = WeatherStationClient::new(server.base_url()).unwrap();

        let measurement = client.get_measurement(7).await.unwrap().unwrap();

        assert_eq!(
            measurement,
            Measurement {
                timestamp: Some("2026-11-16T10:15:30.000+0300".to_owned()),
                time_zone_hours: 3,
                device: Device {
                    id: 7,
                    device_type: None,
                    name: Some("Outdoor".to_owned()),
                    location: None,
                },
                measures: vec![Measure {
                    sensor_id: "dht-1".to_owned(),
                    sensor_name: Some("DHT22".to_owned()),
                    sensor_place: "balcony".to_owned(),
                    measure_name: "Temperature".to_owned(),
                    measure_value: 21.5,
                    measure_unit: "C".to_owned(),
                    sensor_error: false,
                    daily_calculation: Some(DailyCalculation {
                        max_value: 22.0,
                        min_value: 19.0,
                        average_value: 20.5,
                        max_value_time: Some("2026-11-16T14:00:00.000+0300".to_owned()),
                        min_value_time: None,
                    }),
                }],
            }
        );
    }

    #[tokio::test]
    async fn maps_non_success_response_to_response_error() {
        let server = TestServer::new_with_status("GET /devices HTTP/1.1", 500, "server error");
        let client = WeatherStationClient::new(server.base_url()).unwrap();

        let error = client.get_device_list().await.unwrap_err();

        assert_eq!(
            error,
            WeatherStationClientError::Response {
                code: 500,
                message: "server error".to_owned(),
            }
        );
    }

    struct TestServer {
        address: String,
        handle: Option<thread::JoinHandle<()>>,
    }

    impl TestServer {
        fn new(expected_request_line: &'static str, body: &'static str) -> Self {
            Self::new_with_status(expected_request_line, 200, body)
        }

        fn new_with_status(
            expected_request_line: &'static str,
            status: u16,
            body: &'static str,
        ) -> Self {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            let address = format!("http://{}", listener.local_addr().unwrap());
            let handle = thread::spawn(move || {
                let (mut stream, _) = listener.accept().unwrap();
                let request = read_request(&mut stream);
                let request_line = request.lines().next().unwrap_or_default();
                assert_eq!(request_line, expected_request_line);

                let reason = match status {
                    200 => "OK",
                    500 => "Internal Server Error",
                    _ => "Test Status",
                };
                let response = format!(
                    "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    body
                );
                stream.write_all(response.as_bytes()).unwrap();
            });

            Self {
                address,
                handle: Some(handle),
            }
        }

        fn base_url(&self) -> &str {
            &self.address
        }
    }

    impl Drop for TestServer {
        fn drop(&mut self) {
            if let Some(handle) = self.handle.take() {
                handle.join().unwrap();
            }
        }
    }

    fn read_request(stream: &mut TcpStream) -> String {
        let mut buffer = [0_u8; 4096];
        let mut request = Vec::new();

        loop {
            let read = stream.read(&mut buffer).unwrap();
            if read == 0 {
                break;
            }

            request.extend_from_slice(&buffer[..read]);

            if request.windows(4).any(|window| window == b"\r\n\r\n") {
                break;
            }
        }

        String::from_utf8(request).unwrap()
    }
}
