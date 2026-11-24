use crate::app::settings::{AppSettings, ThemeMode};
use crate::data::weather_repository::WeatherStationRepository;
use crate::domain::error::AppError;
use crate::domain::use_case::group_devices_by_location;
use crate::presentation::mapper::to_device_measures_state;
use crate::presentation::message::Message;
use crate::presentation::state::{AppState, DeviceMeasuresState, SensorMeasuresState};
use iced::widget::{
    Column, button, column, container, horizontal_rule, horizontal_space, mouse_area, opaque,
    pick_list, row, scrollable, stack, svg, text, text_input, tooltip,
};
use iced::widget::{
    button::{Status as ButtonStatus, Style as ButtonStyle},
    container::Style as ContainerStyle,
    svg::{Handle as SvgHandle, Style as SvgStyle},
    tooltip::Position as TooltipPosition,
};
use iced::{
    Alignment, Background, Border, Color, Element, Fill, Shadow, Subscription, Task, Theme, Vector,
    time,
};
use std::time::Duration;
use weather_station_client::Device;

const MIN_REFRESH_INTERVAL_SECONDS: u64 = 5;
const DRAWER_WIDTH: u16 = 320;
const TOP_BAR_HEIGHT: u16 = 64;
const PAGE_PADDING: u16 = 16;
const PAGE_SPACING: u16 = 16;
const PANEL_SPACING: u16 = 12;
const FIELD_SPACING: u16 = 8;
const CARD_PADDING: u16 = 12;
const CARD_SPACING: u16 = 8;
const MEASURE_SPACING: u16 = 4;
const SECTION_TITLE_SIZE: u16 = 24;
const DEVICE_TITLE_SIZE: u16 = 22;
const BODY_SIZE: u16 = 18;
const CAPTION_SIZE: u16 = 14;
const ICON_BUTTON_SIZE: u16 = 44;

pub(crate) const APP_TITLE: &str = "Weather Station";
const SETTINGS_TITLE: &str = "Settings";
const HOST_PLACEHOLDER: &str = "Host";
const PORT_PLACEHOLDER: &str = "Port";
const REFRESH_INTERVAL_PLACEHOLDER: &str = "Refresh interval, sec";
const THEME_PICK_LIST_PLACEHOLDER: &str = "Theme";
const APPLY_BUTTON: &str = "Apply";
const REFRESH_TOOLTIP: &str = "Refresh";
const SETTINGS_TOOLTIP: &str = "Settings";
const CLOSE_TOOLTIP: &str = "Close";
const SETTINGS_ICON: &[u8] = include_bytes!("../../assets/icons/settings.svg");
const CLOSE_ICON: &[u8] = include_bytes!("../../assets/icons/close.svg");
const REFRESH_ICON: &[u8] = include_bytes!("../../assets/icons/refresh.svg");
const NO_LOCATIONS_TEXT: &str = "No locations loaded";
const NO_MEASUREMENTS_TEXT: &str = "No measurements";
const NO_DEVICE_DATA_TEXT: &str = "No data for this device";
const ERROR_STATUS_PREFIX: &str = "Error";
const LOADING_DEVICES_STATUS: &str = "Loading devices...";
const REFRESHING_STATUS: &str = "Refreshing...";
const EMPTY_HOST_STATUS: &str = "Enter host and apply settings";
const AUTO_REFRESH_STATUS_PREFIX: &str = "Auto refresh";
const SELECT_LOCATION_TEXT: &str = "Location";

pub fn init() -> (AppState, Task<Message>) {
    let mut state = AppState::default();
    let task = configure_repository(&mut state);
    (state, task)
}

pub fn update(state: &mut AppState, message: Message) -> Task<Message> {
    match message {
        Message::SettingsDrawerToggled => {
            state.is_settings_drawer_open = !state.is_settings_drawer_open;
            Task::none()
        }
        Message::CloseSettingsDrawer => {
            state.is_settings_drawer_open = false;
            Task::none()
        }
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
    let content = main_screen(state);

    if !state.is_settings_drawer_open {
        return content;
    }

    stack([content, drawer_layer(state)])
        .width(Fill)
        .height(Fill)
        .into()
}

fn main_screen(state: &AppState) -> Element<'_, Message> {
    container(
        column![top_bar(state), dashboard(state)]
            .width(Fill)
            .height(Fill),
    )
    .width(Fill)
    .height(Fill)
    .style(app_background)
    .into()
}

fn top_bar(state: &AppState) -> Element<'_, Message> {
    let refresh_button = if state.is_refreshing || state.is_loading_devices {
        button(icon(REFRESH_ICON)).style(icon_button_style)
    } else {
        button(icon(REFRESH_ICON))
            .on_press(Message::ReloadClicked)
            .style(icon_button_style)
    };

    container(
        row![
            status_badge(state),
            horizontal_space(),
            icon_tooltip(
                refresh_button
                    .width(ICON_BUTTON_SIZE)
                    .height(ICON_BUTTON_SIZE)
                    .padding(0),
                REFRESH_TOOLTIP,
                TooltipPosition::Bottom,
            ),
            icon_tooltip(
                button(icon(SETTINGS_ICON))
                    .on_press(Message::SettingsDrawerToggled)
                    .width(ICON_BUTTON_SIZE)
                    .height(ICON_BUTTON_SIZE)
                    .padding(0)
                    .style(icon_button_style),
                SETTINGS_TOOLTIP,
                TooltipPosition::Bottom,
            ),
        ]
        .spacing(PANEL_SPACING)
        .align_y(Alignment::Center),
    )
    .width(Fill)
    .height(TOP_BAR_HEIGHT)
    .padding(PAGE_PADDING)
    .style(top_bar_style)
    .into()
}

fn dashboard(state: &AppState) -> Element<'_, Message> {
    let mut measures = Column::new().spacing(PANEL_SPACING);
    for device in &state.device_measures {
        measures = measures.push(device_measures(device));
    }

    if state.device_measures.is_empty() {
        measures = measures.push(text(NO_MEASUREMENTS_TEXT).size(BODY_SIZE));
    }

    container(
        column![location_selector(state), scrollable(measures).height(Fill)]
            .spacing(PAGE_SPACING)
            .width(Fill),
    )
    .padding(PAGE_PADDING)
    .width(Fill)
    .height(Fill)
    .into()
}

fn location_selector(state: &AppState) -> Element<'_, Message> {
    let selector: Element<'_, Message> = if state.locations.is_empty() {
        text(NO_LOCATIONS_TEXT).size(BODY_SIZE).into()
    } else if state.locations.len() == 1 {
        text(
            state
                .selected_location
                .as_deref()
                .unwrap_or(NO_LOCATIONS_TEXT),
        )
        .size(BODY_SIZE)
        .into()
    } else {
        pick_list(
            state.locations.clone(),
            state.selected_location.clone(),
            Message::LocationSelected,
        )
        .placeholder(SELECT_LOCATION_TEXT)
        .into()
    };

    container(selector)
        .width(Fill)
        .padding(CARD_PADDING)
        .style(surface_panel)
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

    container(content)
        .padding(CARD_PADDING)
        .width(Fill)
        .style(surface_panel)
        .into()
}

fn sensor_measures(sensor: &SensorMeasuresState) -> Element<'_, Message> {
    let mut content = Column::new()
        .spacing(MEASURE_SPACING)
        .push(text(sensor.sensor_text()).size(BODY_SIZE));

    for measure in &sensor.measures {
        content = content.push(horizontal_rule(1)).push(
            column![
                row![
                    text(&measure.title).size(BODY_SIZE),
                    horizontal_space(),
                    text(&measure.value).size(BODY_SIZE),
                ]
                .spacing(PANEL_SPACING)
                .align_y(Alignment::Center),
                text(measure.updated_text()).size(CAPTION_SIZE),
                text(measure.daily_text()).size(CAPTION_SIZE),
            ]
            .spacing(MEASURE_SPACING),
        );
    }

    container(content).padding(CARD_PADDING).width(Fill).into()
}

fn status_badge(state: &AppState) -> Element<'_, Message> {
    let has_error = state.error.is_some();
    container(text(status_message(state)).size(CAPTION_SIZE))
        .padding(8)
        .style(move |theme| status_badge_style(theme, has_error))
        .into()
}

fn drawer_layer(state: &AppState) -> Element<'_, Message> {
    row![
        opaque(settings_drawer(state)),
        mouse_area(
            container(text(""))
                .width(Fill)
                .height(Fill)
                .style(drawer_scrim)
        )
        .on_press(Message::CloseSettingsDrawer),
    ]
    .width(Fill)
    .height(Fill)
    .into()
}

fn settings_drawer(state: &AppState) -> Element<'_, Message> {
    container(
        column![
            row![
                text(SETTINGS_TITLE).size(SECTION_TITLE_SIZE),
                horizontal_space(),
                icon_tooltip(
                    button(icon(CLOSE_ICON))
                        .on_press(Message::CloseSettingsDrawer)
                        .width(ICON_BUTTON_SIZE)
                        .height(ICON_BUTTON_SIZE)
                        .padding(0)
                        .style(icon_button_style),
                    CLOSE_TOOLTIP,
                    TooltipPosition::Right,
                ),
            ]
            .align_y(Alignment::Center),
            horizontal_rule(1),
            column![
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
            ]
            .spacing(FIELD_SPACING),
            button(APPLY_BUTTON).on_press(Message::ApplySettings),
        ]
        .spacing(PANEL_SPACING),
    )
    .width(DRAWER_WIDTH)
    .height(Fill)
    .padding(PAGE_PADDING)
    .style(drawer_panel)
    .into()
}

fn status_message(state: &AppState) -> String {
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

    status
}

fn icon(bytes: &'static [u8]) -> Element<'static, Message> {
    container(
        svg(SvgHandle::from_memory(bytes))
            .width(24)
            .height(24)
            .style(|theme: &Theme, _| SvgStyle {
                color: Some(icon_color(theme)),
            }),
    )
    .center(Fill)
    .into()
}

fn icon_tooltip<'a>(
    content: impl Into<Element<'a, Message>>,
    label: &'static str,
    position: TooltipPosition,
) -> Element<'a, Message> {
    tooltip(content, text(label).size(CAPTION_SIZE), position)
        .gap(6)
        .padding(8)
        .style(tooltip_style)
        .into()
}

fn icon_color(theme: &Theme) -> Color {
    if theme.extended_palette().is_dark {
        Color::WHITE
    } else {
        Color::from_rgb8(0x00, 0x4D, 0x40)
    }
}

fn icon_button_style(theme: &Theme, status: ButtonStatus) -> ButtonStyle {
    let color = icon_color(theme);
    let hover_background = if theme.extended_palette().is_dark {
        Color::from_rgba8(0xFF, 0xFF, 0xFF, 0.12)
    } else {
        Color::from_rgba8(0x00, 0x4D, 0x40, 0.12)
    };

    let background = match status {
        ButtonStatus::Hovered | ButtonStatus::Pressed => Some(Background::Color(hover_background)),
        ButtonStatus::Active => None,
        ButtonStatus::Disabled => None,
    };

    ButtonStyle {
        background,
        text_color: if matches!(status, ButtonStatus::Disabled) {
            color.scale_alpha(0.45)
        } else {
            color
        },
        border: Border {
            radius: 4.0.into(),
            ..Border::default()
        },
        ..Default::default()
    }
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

fn app_background(theme: &Theme) -> ContainerStyle {
    let palette = theme.extended_palette();

    ContainerStyle::default()
        .background(palette.background.base.color)
        .color(palette.background.base.text)
}

fn top_bar_style(theme: &Theme) -> ContainerStyle {
    let is_dark = theme.extended_palette().is_dark;
    let background = if is_dark {
        Color::from_rgb8(0x34, 0x34, 0x34)
    } else {
        Color::from_rgb8(0xB2, 0xDF, 0xDB)
    };
    let text_color = if is_dark {
        Color::from_rgb8(0xFF, 0xFF, 0xFF)
    } else {
        Color::from_rgb8(0x00, 0x79, 0x6B)
    };

    ContainerStyle::default()
        .background(background)
        .color(text_color)
}

fn surface_panel(theme: &Theme) -> ContainerStyle {
    let palette = theme.extended_palette();

    ContainerStyle {
        text_color: Some(palette.background.base.text),
        background: Some(palette.background.weak.color.into()),
        border: Border {
            color: palette.background.strong.color,
            width: 1.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    }
}

fn drawer_panel(theme: &Theme) -> ContainerStyle {
    let palette = theme.extended_palette();

    ContainerStyle {
        text_color: Some(palette.background.base.text),
        background: Some(palette.background.base.color.into()),
        border: Border {
            color: palette.background.strong.color,
            width: 1.0,
            radius: 4.0.into(),
        },
        shadow: Shadow {
            color: Color::from_rgba8(0, 0, 0, 0.18),
            offset: Vector::new(3.0, 0.0),
            blur_radius: 14.0,
        },
    }
}

fn drawer_scrim(_theme: &Theme) -> ContainerStyle {
    ContainerStyle::default().background(Color::from_rgba8(0, 0, 0, 0.05))
}

fn tooltip_style(theme: &Theme) -> ContainerStyle {
    let palette = theme.extended_palette();

    ContainerStyle {
        text_color: Some(palette.background.base.color),
        background: Some(palette.background.base.text.into()),
        border: Border {
            radius: 4.0.into(),
            ..Border::default()
        },
        shadow: Shadow {
            color: Color::from_rgba8(0, 0, 0, 0.18),
            offset: Vector::new(0.0, 2.0),
            blur_radius: 8.0,
        },
    }
}

fn status_badge_style(theme: &Theme, has_error: bool) -> ContainerStyle {
    let palette = theme.extended_palette();
    let pair = if has_error {
        palette.danger.weak
    } else {
        palette.primary.weak
    };

    ContainerStyle {
        text_color: Some(pair.text),
        background: Some(pair.color.into()),
        border: Border {
            color: if has_error {
                palette.danger.strong.color
            } else {
                palette.primary.strong.color
            },
            width: 1.0,
            radius: 4.0.into(),
        },
        ..Default::default()
    }
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
