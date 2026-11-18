use crate::app::settings::{AppSettings, PreferencesStore, ThemeMode};
use crate::data::weather_repository::WeatherStationRepository;
use crate::weather_station_client::Device;
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct AppState {
    pub settings: AppSettings,
    pub settings_store: PreferencesStore,
    pub repository: Option<WeatherStationRepository>,
    pub host_input: String,
    pub port_input: String,
    pub refresh_interval_input: String,
    pub selected_theme_mode: ThemeMode,
    pub locations: Vec<String>,
    pub selected_location: Option<String>,
    pub device_location_map: BTreeMap<String, Vec<Device>>,
    pub current_devices: Vec<Device>,
    pub device_measures: Vec<DeviceMeasuresState>,
    pub is_loading_devices: bool,
    pub is_refreshing: bool,
    pub error: Option<String>,
}

impl Default for AppState {
    fn default() -> Self {
        let settings_store = PreferencesStore::default();
        let settings = settings_store.load();
        Self {
            host_input: settings.host.clone(),
            port_input: settings.port.clone(),
            refresh_interval_input: settings.refresh_interval_seconds.to_string(),
            selected_theme_mode: settings.theme_mode,
            settings,
            settings_store,
            repository: None,
            locations: Vec::new(),
            selected_location: None,
            device_location_map: BTreeMap::new(),
            current_devices: Vec::new(),
            device_measures: Vec::new(),
            is_loading_devices: false,
            is_refreshing: false,
            error: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct DeviceMeasuresState {
    pub device_name: String,
    pub sensors: Vec<SensorMeasuresState>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SensorMeasuresState {
    pub sensor_name: String,
    pub sensor_place: String,
    pub measures: Vec<MeasureState>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeasureState {
    pub title: String,
    pub value: String,
    pub time: String,
    pub value_min: String,
    pub value_average: String,
    pub value_max: String,
    pub time_min: String,
    pub time_max: String,
}

impl MeasureState {
    const UPDATED_LABEL: &'static str = "Updated";
    const DAILY_LABEL: &'static str = "Daily";
    const MIN_LABEL: &'static str = "min";
    const AT_LABEL: &'static str = "at";
    const AVG_LABEL: &'static str = "avg";
    const MAX_LABEL: &'static str = "max";

    pub fn updated_text(&self) -> String {
        format!("{}: {}", Self::UPDATED_LABEL, self.time)
    }

    pub fn daily_text(&self) -> String {
        format!(
            "{}: {} {} {} {}, {} {}, {} {} {} {}",
            Self::DAILY_LABEL,
            Self::MIN_LABEL,
            self.value_min,
            Self::AT_LABEL,
            self.time_min,
            Self::AVG_LABEL,
            self.value_average,
            Self::MAX_LABEL,
            self.value_max,
            Self::AT_LABEL,
            self.time_max
        )
    }
}

impl SensorMeasuresState {
    const SENSOR_LABEL: &'static str = "Sensor";
    const SENSOR_SEPARATOR: &'static str = " / ";

    pub fn sensor_text(&self) -> String {
        format!(
            "{}: {}{}{}",
            Self::SENSOR_LABEL,
            self.sensor_name,
            Self::SENSOR_SEPARATOR,
            self.sensor_place
        )
    }
}
