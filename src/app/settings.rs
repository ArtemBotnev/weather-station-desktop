use std::fs;
use std::path::PathBuf;

const DEFAULT_REFRESH_INTERVAL_SECONDS: u64 = 60;
const SETTINGS_FILE_NAME: &str = "weather-station-desktop.settings";
const HTTP_SCHEME: &str = "http";
const HOST_KEY: &str = "host";
const PORT_KEY: &str = "port";
const REFRESH_INTERVAL_SECONDS_KEY: &str = "refresh_interval_seconds";
const THEME_MODE_KEY: &str = "theme_mode";
const THEME_MODE_SYSTEM_VALUE: &str = "system";
const THEME_MODE_LIGHT_VALUE: &str = "light";
const THEME_MODE_DARK_VALUE: &str = "dark";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppSettings {
    pub host: String,
    pub port: String,
    pub refresh_interval_seconds: u64,
    pub theme_mode: ThemeMode,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            host: String::new(),
            port: String::new(),
            refresh_interval_seconds: DEFAULT_REFRESH_INTERVAL_SECONDS,
            theme_mode: ThemeMode::System,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeMode {
    System,
    Light,
    Dark,
}

impl ThemeMode {
    pub const ALL: [Self; 3] = [Self::System, Self::Light, Self::Dark];

    pub fn as_setting_value(self) -> &'static str {
        match self {
            Self::System => THEME_MODE_SYSTEM_VALUE,
            Self::Light => THEME_MODE_LIGHT_VALUE,
            Self::Dark => THEME_MODE_DARK_VALUE,
        }
    }

    fn from_setting_value(value: &str) -> Self {
        match value {
            THEME_MODE_LIGHT_VALUE => Self::Light,
            THEME_MODE_DARK_VALUE => Self::Dark,
            _ => Self::System,
        }
    }
}

impl std::fmt::Display for ThemeMode {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            Self::System => "System",
            Self::Light => "Light",
            Self::Dark => "Dark",
        };
        formatter.write_str(label)
    }
}

impl AppSettings {
    pub fn base_url(&self) -> Option<String> {
        let host = self.host.trim();
        if host.is_empty() {
            return None;
        }

        let port = self.port.trim();
        let address = if port.is_empty() {
            host.to_owned()
        } else {
            format!("{host}:{port}")
        };

        Some(format!("{HTTP_SCHEME}://{address}"))
    }
}

#[derive(Debug, Clone)]
pub struct PreferencesStore {
    path: PathBuf,
}

impl Default for PreferencesStore {
    fn default() -> Self {
        let path = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(SETTINGS_FILE_NAME);
        Self { path }
    }
}

impl PreferencesStore {
    pub fn load(&self) -> AppSettings {
        let Ok(content) = fs::read_to_string(&self.path) else {
            return AppSettings::default();
        };

        let mut settings = AppSettings::default();
        for line in content.lines() {
            let Some((key, value)) = line.split_once('=') else {
                continue;
            };

            match key.trim() {
                HOST_KEY => settings.host = value.trim().to_owned(),
                PORT_KEY => settings.port = value.trim().to_owned(),
                REFRESH_INTERVAL_SECONDS_KEY => {
                    if let Ok(value) = value.trim().parse::<u64>() {
                        settings.refresh_interval_seconds = value;
                    }
                }
                THEME_MODE_KEY => {
                    settings.theme_mode = ThemeMode::from_setting_value(value.trim());
                }
                _ => {}
            }
        }

        settings
    }

    pub fn save(&self, settings: &AppSettings) -> std::io::Result<()> {
        fs::write(
            &self.path,
            format!(
                "{HOST_KEY}={}\n{PORT_KEY}={}\n{REFRESH_INTERVAL_SECONDS_KEY}={}\n{THEME_MODE_KEY}={}\n",
                settings.host,
                settings.port,
                settings.refresh_interval_seconds,
                settings.theme_mode.as_setting_value()
            ),
        )
    }
}
