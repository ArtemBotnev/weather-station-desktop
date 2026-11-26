use crate::app::settings::ThemeMode;
use crate::domain::error::AppError;
use crate::presentation::state::DeviceMeasuresState;
use weather_station_client::Device;

#[derive(Debug, Clone)]
pub enum Message {
    SettingsDrawerToggled,
    CloseSettingsDrawer,
    HostInputChanged(String),
    PortInputChanged(String),
    RefreshIntervalChanged(String),
    ThemeModeSelected(ThemeMode),
    ApplySettings,
    ReloadClicked,
    RefreshTick,
    LocationSelected(String),
    DevicesLoaded(Result<Vec<Device>, AppError>),
    MeasurementsLoaded(Result<Vec<DeviceMeasuresState>, AppError>),
}
