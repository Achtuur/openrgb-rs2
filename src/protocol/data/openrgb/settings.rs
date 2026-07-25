use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

use crate::{impl_bufserde_json, impl_enum_discriminant};

/// Settings schema returned by `GetSettingsSchema` packet
#[derive(Serialize, Deserialize, Debug)]
pub struct SettingsSchema {
    /// All entries, the key is the name of the setting
    #[serde(flatten)]
    pub entries: HashMap<String, SettingsSchemaEntry>,
}

impl_bufserde_json!(SettingsSchema);

/// Entry for a setting in a `SettingsSchema`
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SettingsSchemaEntry {
    /// Title for the setting
    pub title: String,
    /// Description containing some extra info
    pub description: Option<String>,
    /// UI order for the setting
    pub order: usize,
    /// Data type of the value
    #[serde(rename = "type")]
    pub value_type: String,
}

/// Keys for settings in `OpenRGB`.
pub enum SettingsKey {
    /// Automatic startup, see [`AutoStartSettings`]
    AutoStart,
    /// Detection and enabled detectors, see [`DetectorSettings`]
    Detectors,
    /// Logs, see [`LogManagerSettings`]
    LogManager,
    /// Profiles, see [`ProfileManagerSettings`]
    ProfileManager,
    /// SDK server related settings, see [`ServerSettings`]
    Server,
    /// UI Settings for the `OpenRGB` app, see [`UiSettings`]
    UserInterface,
    /// Controller specific settings
    ///
    /// You'll have to find out yourself what the keys are here
    Controller(String),
}

impl SettingsKey {
    /// Returns the string for this key. This is used in `OpenRGB` and the `OpenRGB.json` config file.
    pub fn as_str(&self) -> &str {
        match self {
            Self::AutoStart => "AutoStart",
            Self::Detectors => "Detectors",
            Self::LogManager => "LogManager",
            Self::ProfileManager => "ProfileManager",
            Self::Server => "Server",
            Self::UserInterface => "UserInterface",
            Self::Controller(s) => s.as_str(),
        }
    }
}

/// Settings related to when and how `OpenRGB` automatically starts
#[derive(Debug, Serialize, Deserialize)]
pub struct AutoStartSettings {
    enabled: bool,
    start_minimized: bool,
    custom_arguments: String,
}

impl_bufserde_json!(AutoStartSettings);

/// Detection related settings
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DetectorSettings {
    /// Map of Detector name -> detection on/off
    pub detectors: HashMap<String, bool>,
    /// HID safe mode
    #[serde(default)]
    pub hid_safe_mode: bool,
    /// Number of milliseconds to wait before detecting devices when started
    #[serde(default)]
    pub initial_detection_delay_ms: usize,
}

impl_bufserde_json!(DetectorSettings);

/// Log levels that `OpenRGB` uses
#[derive(Serialize_repr, Deserialize_repr, Debug, PartialEq, Eq)]
#[repr(u32)]
#[allow(missing_docs, reason = "self-explanatory fields")]
pub enum OpenRgbLogLevel {
    Fatal = 0,
    Error = 1,
    Warning = 2,
    Info = 3,
    Verbose = 4,
    Debug = 5,
    Trace = 6,
}

impl_enum_discriminant!(OpenRgbLogLevel,
    Fatal: 0,
    Error: 1,
    Warning: 2,
    Info: 3,
    Verbose: 4,
    Debug: 5,
    Trace: 6
);

/// Settings for the `LogManager` in `OpenRGB`
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogManagerSettings {
    /// Max number of log files
    pub file_count_limit: usize,
    /// Whether to log the console inside `OpenRGB`
    pub log_console: bool,
    /// Whether to log to log file(s)
    pub log_file: bool,
    /// Minimum log level
    pub loglevel: OpenRgbLogLevel,
}

impl_bufserde_json!(LogManagerSettings);

/// A profile that should be automatically loaded
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutoloadProfile {
    /// Whether autoloading is enabled
    pub enabled: bool,
    /// Name of the profile
    pub name: String,
}

/// Settings for the `ProfileManager` in `OpenRGB`
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProfileManagerSettings {
    /// Profile to load on exit
    pub exit_profile: AutoloadProfile,
    /// Profile to load on open
    pub open_profile: AutoloadProfile,
    /// Profile to load on resume
    pub resume_profile: AutoloadProfile,
    /// Profile to load on suspend
    pub suspend_profile: AutoloadProfile,
}

impl_bufserde_json!(ProfileManagerSettings);

/// SDK server related settings
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServerSettings {
    /// Whether to serve all controllers
    pub all_controllers: bool,
    /// Default host to use
    pub default_host: String,
    /// Default port to use
    pub default_port: u16,
    /// Workaround for older sdk clients that send the wrong size for packets
    pub legacy_workaround: bool,
}

impl_bufserde_json!(ServerSettings);

/// Window Geometry of `OpenRGB` window
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct WindowGeometry {
    /// X position
    x: i32,
    /// Y position
    y: i32,
    /// Window width
    width: u32,
    /// Window height
    height: u32,
    /// Whether to load window geometry on startup
    #[serde(rename = "load_window_geometry")]
    enabled: bool,
    /// Whether to save geometry on exit
    save_on_exit: bool,
}

/// User Interface settings for `OpenRGB`
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UiSettings {
    /// Whether to use compact tabs
    pub compact_tabs: bool,
    /// Whether to disable key expansion in LED view
    pub disable_key_expansion: bool,
    /// Window geometry
    pub geometry: WindowGeometry,
    /// UI language
    pub language: String,
    /// Whether to minimize window on close instead of exiting
    pub minimize_on_close: bool,
    /// Whether to use a monochrome icon in tray.
    pub monochrome_tray_icon: bool,
    /// Whether to use numerical labels in the LED view
    pub numerical_labels: bool,
    /// Whether to run zone checks on rescan
    pub run_zone_checks: bool,
    /// Whether to show the led view by default
    pub show_led_view: bool,
    /// Whether to put the tabs on top instead of on the left.
    pub tabs_on_top: bool,
}

impl_bufserde_json!(UiSettings);
