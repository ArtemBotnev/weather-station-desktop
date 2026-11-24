use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Device {
    pub id: i32,
    #[serde(rename = "type")]
    pub device_type: Option<String>,
    pub name: Option<String>,
    pub location: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Measurement {
    pub timestamp: Option<String>,
    pub time_zone_hours: i32,
    pub device: Device,
    pub measures: Vec<Measure>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Measure {
    pub sensor_id: String,
    pub sensor_name: Option<String>,
    pub sensor_place: String,
    pub measure_name: String,
    pub measure_value: f64,
    pub measure_unit: String,
    #[serde(default)]
    pub sensor_error: bool,
    pub daily_calculation: Option<DailyCalculation>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyCalculation {
    pub max_value: f64,
    pub min_value: f64,
    pub average_value: f64,
    pub max_value_time: Option<String>,
    pub min_value_time: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceDailyErrors {
    pub device_id: i32,
    pub device_name: Option<String>,
    #[serde(default)]
    pub start_analysis_time: Option<String>,
    #[serde(default)]
    pub last_update_analysis_time: Option<String>,
    #[serde(default)]
    pub analysis_duration: Option<String>,
    #[serde(default)]
    pub time_zone_hours: i32,
    pub sensor_error_list: Vec<SensorDailyErrors>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SensorDailyErrors {
    pub sensor_id: String,
    pub sensor_name: Option<String>,
    pub total_measures: i32,
    pub error_count: i32,
    pub error_percent: f64,
}
