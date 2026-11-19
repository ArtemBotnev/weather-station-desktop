use crate::app::settings::{AppSettings, ThemeMode};
use crate::data::weather_repository::WeatherStationRepository;
use crate::domain::error::AppError;
use crate::domain::use_case::group_devices_by_location;
use crate::presentation::mapper::to_device_measures_state;
use crate::presentation::message::Message;
use crate::presentation::state::{AppState, DeviceMeasuresState, SensorMeasuresState};
use crate::weather_station_client::Device;
use iced::widget::{
    Column, button, column, container, pick_list, row, scrollable, text, text_input,
};
use iced::{Alignment, Element, Fill, Subscription, Task, Theme, time};
use std::time::Duration;

const MIN_REFRESH_INTERVAL_SECONDS: u64 = 5;
const SIDEBAR_WIDTH: u16 = 280;
const SIDEBAR_CONTENT_WIDTH: u16 = 260;
const PAGE_PADDING: u16 = 16;
const PAGE_SPACING: u16 = 16;
const PANEL_SPACING: u16 = 12;
const CARD_PADDING: u16 = 12;
const CARD_SPACING: u16 = 8;
const MEASURE_SPACING: u16 = 4;
const TITLE_SIZE: u16 = 32;
const SECTION_TITLE_SIZE: u16 = 24;
const DEVICE_TITLE_SIZE: u16 = 22;
const BODY_SIZE: u16 = 18;

pub(crate) const APP_TITLE: &str = "Weather Station";
const SETTINGS_TITLE: &str = "Settings";
const HOST_PLACEHOLDER: &str = "Host";
const PORT_PLACEHOLDER: &str = "Port";
const REFRESH_INTERVAL_PLACEHOLDER: &str = "Refresh interval, sec";
const THEME_PICK_LIST_PLACEHOLDER: &str = "Theme";
const APPLY_BUTTON: &str = "Apply";
const REFRESH_BUTTON: &str = "Refresh";
const NO_LOCATIONS_TEXT: &str = "No locations loaded";
const NO_MEASUREMENTS_TEXT: &str = "No measurements";
const NO_DEVICE_DATA_TEXT: &str = "No data for this device";
const ERROR_STATUS_PREFIX: &str = "Error";
const LOADING_DEVICES_STATUS: &str = "Loading devices...";
const REFRESHING_STATUS: &str = "Refreshing...";
const EMPTY_HOST_STATUS: &str = "Enter host and apply settings";
const AUTO_REFRESH_STATUS_PREFIX: &str = "Auto refresh";

pub fn init() -> (AppState, Task<Message>) {
    let mut state = AppState::default();
    let task = configure_repository(&mut state);
    (state, task)
}

pub fn update(state: &mut AppState, message: Message) -> Task<Message> {
    match message {
        Message::HostInputChanged(value) => {
            state.host_input = value;
            Task::none()
        }
        Message::PortInputChanged(value) => {
            state.port_input = value;
            Task::none()
        }
        Message::RefreshIntervalChanged(value) => {
            state.refresh_interval_input = value
                .chars()
                .filter(char::is_ascii_digit)
                .collect::<String>();
            Task::none()
        }
        Message::ThemeModeSelected(theme_mode) => {
            state.selected_theme_mode = theme_mode;
            Task::none()
        }
        Message::ApplySettings => apply_settings(state),
        Message::ReloadClicked | Message::RefreshTick => refresh_current_location(state),
        Message::LocationSelected(location) => {
            state.selected_location = Some(location.clone());
            state.current_devices = state
                .device_location_map
                .get(&location)
                .cloned()
                .unwrap_or_default();
            refresh_current_location(state)
        }
        Message::DevicesLoaded(result) => {
            state.is_loading_devices = false;
            match result {
                Ok(devices) => {
                    state.error = None;
                    state.device_location_map = group_devices_by_location(devices);
                    state.locations = state.device_location_map.keys().cloned().collect();
                    state.selected_location = state
                        .selected_location
                        .clone()
                        .filter(|location| state.device_location_map.contains_key(location))
                        .or_else(|| state.locations.first().cloned());
                    state.current_devices = state
                        .selected_location
                        .as_ref()
                        .and_then(|location| state.device_location_map.get(location))
                        .cloned()
                        .unwrap_or_default();
                    refresh_current_location(state)
                }
                Err(error) => {
                    state.error = Some(error.to_string());
                    Task::none()
                }
            }
        }
        Message::MeasurementsLoaded(result) => {
            state.is_refreshing = false;
            match result {
                Ok(measures) => {
                    state.error = None;
                    state.device_measures = measures;
                }
                Err(error) => {
                    state.error = Some(error.to_string());
                }
            }
            Task::none()
        }
    }
}

pub fn theme(state: &AppState) -> Theme {
    match state.settings.theme_mode {
        ThemeMode::System => Theme::default(),
        ThemeMode::Light => Theme::Light,
        ThemeMode::Dark => Theme::Dark,
    }
}

pub fn subscription(state: &AppState) -> Subscription<Message> {
    if state.settings.refresh_interval_seconds == 0 {
        return Subscription::none();
    }

    time::every(Duration::from_secs(state.settings.refresh_interval_seconds))
        .map(|_| Message::RefreshTick)
}

pub fn view(state: &AppState) -> Element<'_, Message> {
    let settings = settings_panel(state);
    let content = dashboard(state);

    container(
        row![settings, content]
            .spacing(PAGE_SPACING)
            .padding(PAGE_PADDING),
    )
    .width(Fill)
    .height(Fill)
    .into()
}

fn settings_panel(state: &AppState) -> Element<'_, Message> {
    let refresh_button = if state.is_refreshing || state.is_loading_devices {
        button(REFRESH_BUTTON)
    } else {
        button(REFRESH_BUTTON).on_press(Message::ReloadClicked)
    };

    container(
        column![
            text(SETTINGS_TITLE).size(SECTION_TITLE_SIZE),
            text_input(HOST_PLACEHOLDER, &state.host_input).on_input(Message::HostInputChanged),
            text_input(PORT_PLACEHOLDER, &state.port_input).on_input(Message::PortInputChanged),
            text_input(REFRESH_INTERVAL_PLACEHOLDER, &state.refresh_interval_input)
                .on_input(Message::RefreshIntervalChanged),
            pick_list(
                ThemeMode::ALL,
                Some(state.selected_theme_mode),
                Message::ThemeModeSelected,
            )
            .placeholder(THEME_PICK_LIST_PLACEHOLDER),
            button(APPLY_BUTTON).on_press(Message::ApplySettings),
            refresh_button,
            status_text(state),
        ]
        .spacing(PANEL_SPACING)
        .width(SIDEBAR_CONTENT_WIDTH),
    )
    .width(SIDEBAR_WIDTH)
    .into()
}

fn dashboard(state: &AppState) -> Element<'_, Message> {
    let title = text(APP_TITLE).size(TITLE_SIZE);
    let location_selector: Element<'_, Message> = if state.locations.is_empty() {
        text(NO_LOCATIONS_TEXT).into()
    } else {
        pick_list(
            state.locations.clone(),
            state.selected_location.clone(),
            Message::LocationSelected,
        )
        .into()
    };

    let mut measures = Column::new().spacing(PANEL_SPACING);
    for device in &state.device_measures {
        measures = measures.push(device_measures(device));
    }

    if state.device_measures.is_empty() {
        measures = measures.push(text(NO_MEASUREMENTS_TEXT).size(BODY_SIZE));
    }

    container(
        column![title, location_selector, scrollable(measures).height(Fill)]
            .spacing(16)
            .width(Fill),
    )
    .width(Fill)
    .height(Fill)
    .into()
}

fn device_measures(device: &DeviceMeasuresState) -> Element<'_, Message> {
    let mut content = Column::new()
        .spacing(CARD_SPACING)
        .push(text(&device.device_name).size(DEVICE_TITLE_SIZE));

    if device.sensors.is_empty() {
        content = content.push(text(NO_DEVICE_DATA_TEXT));
    }

    for sensor in &device.sensors {
        content = content.push(sensor_measures(sensor));
    }

    container(content).padding(CARD_PADDING).width(Fill).into()
}

fn sensor_measures(sensor: &SensorMeasuresState) -> Element<'_, Message> {
    let mut content = Column::new()
        .spacing(MEASURE_SPACING)
        .push(text(sensor.sensor_text()).size(BODY_SIZE));

    for measure in &sensor.measures {
        content = content.push(
            container(
                column![
                    row![
                        text(&measure.title).size(BODY_SIZE),
                        text(&measure.value).size(BODY_SIZE),
                    ]
                    .spacing(PANEL_SPACING)
                    .align_y(Alignment::Center),
                    text(measure.updated_text()),
                    text(measure.daily_text()),
                ]
                .spacing(MEASURE_SPACING),
            )
            .padding(CARD_PADDING)
            .width(Fill),
        );
    }

    container(content).padding(CARD_PADDING).width(Fill).into()
}

fn status_text(state: &AppState) -> Element<'_, Message> {
    let status = if let Some(error) = &state.error {
        format!("{ERROR_STATUS_PREFIX}: {error}")
    } else if state.is_loading_devices {
        LOADING_DEVICES_STATUS.to_owned()
    } else if state.is_refreshing {
        REFRESHING_STATUS.to_owned()
    } else if state.settings.base_url().is_none() {
        EMPTY_HOST_STATUS.to_owned()
    } else {
        format!(
            "{AUTO_REFRESH_STATUS_PREFIX}: {} sec",
            state.settings.refresh_interval_seconds
        )
    };

    text(status).into()
}

fn apply_settings(state: &mut AppState) -> Task<Message> {
    let refresh_interval_seconds = state
        .refresh_interval_input
        .parse::<u64>()
        .unwrap_or(state.settings.refresh_interval_seconds);
    let refresh_interval_seconds = if refresh_interval_seconds == 0 {
        0
    } else {
        refresh_interval_seconds.max(MIN_REFRESH_INTERVAL_SECONDS)
    };

    state.refresh_interval_input = refresh_interval_seconds.to_string();
    state.settings = AppSettings {
        host: state.host_input.trim().to_owned(),
        port: state.port_input.trim().to_owned(),
        refresh_interval_seconds,
        theme_mode: state.selected_theme_mode,
    };

    if let Err(error) = state.settings_store.save(&state.settings) {
        state.error = Some(AppError::from(error).to_string());
    }

    configure_repository(state)
}

fn configure_repository(state: &mut AppState) -> Task<Message> {
    let Some(base_url) = state.settings.base_url() else {
        state.repository = None;
        state.locations.clear();
        state.selected_location = None;
        state.device_location_map.clear();
        state.current_devices.clear();
        state.device_measures.clear();
        return Task::none();
    };

    match WeatherStationRepository::new(base_url) {
        Ok(repository) => {
            state.repository = Some(repository.clone());
            state.is_loading_devices = true;
            Task::perform(load_devices(repository), Message::DevicesLoaded)
        }
        Err(error) => {
            state.error = Some(error.to_string());
            Task::none()
        }
    }
}

fn refresh_current_location(state: &mut AppState) -> Task<Message> {
    let Some(repository) = state.repository.clone() else {
        return Task::none();
    };

    if state.current_devices.is_empty() {
        return Task::none();
    }

    state.is_refreshing = true;
    Task::perform(
        load_measurements(repository, state.current_devices.clone()),
        Message::MeasurementsLoaded,
    )
}

async fn load_devices(repository: WeatherStationRepository) -> Result<Vec<Device>, AppError> {
    repository.devices().await
}

async fn load_measurements(
    repository: WeatherStationRepository,
    devices: Vec<Device>,
) -> Result<Vec<DeviceMeasuresState>, AppError> {
    let mut result = Vec::new();

    for device in devices {
        let measurement = repository.measurement(device.id).await?;
        result.push(to_device_measures_state(&device, measurement));
    }

    Ok(result)
}
