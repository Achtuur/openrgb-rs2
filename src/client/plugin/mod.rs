use crate::PluginData;

pub enum OpenRgbPlugin {
    // https://openrgb.org/plugin_effects.html
    Effects,
    /// <https://openrgb.org/plugin_visual_map.html>
    VisualMap,
    /// <https://openrgb.org/plugin_hardware_sync.html>
    HardwareSync,
    /// <https://openrgb.org/plugin_fan_sync.html>
    FanSync,
    /// <https://openrgb.org/plugin_e131_receiver.html>
    E131Receiver,
    /// <https://openrgb.org/plugin_scheduler.html>
    Scheduler,
    /// Third party plugin that is not on the `OpenRGB` page.
    ///
    /// Argument is the plugin name.
    Unknown(String),
}

impl From<PluginData> for OpenRgbPlugin {
    fn from(value: PluginData) -> Self {
        match value.name() {
            "OpenRGB Effects Plugin" => Self::Effects,
            "Visual Map" => Self::VisualMap,
            "Hardware Sync" => Self::HardwareSync,
            "Fan Sync" => Self::FanSync,
            "E131 Receiver" => Self::E131Receiver,
            "Scheduler" => Self::Scheduler,
            name => Self::Unknown(name.to_owned()),
        }
    }
}

impl OpenRgbPlugin {
    pub fn name(&self) -> &str {
        match self {
            Self::Effects => "OpenRGB Effects Plugin",
            Self::VisualMap => "Visual Map",
            Self::HardwareSync => "Hardware Sync",
            Self::FanSync => "Fan Sync",
            Self::E131Receiver => "E131 Receiver",
            Self::Scheduler => "Scheduler",
            Self::Unknown(name) => name,
        }
    }
}
