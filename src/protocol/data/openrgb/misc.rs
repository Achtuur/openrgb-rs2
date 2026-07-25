use flagset::flags;

use crate::{DeserFromBuf, ReceivedMessage};

flags! {
    /// Client flags
    ///
    /// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation) for more information.
    pub enum ClientFlags: u32 {
        /// Supports RGBController API
        RGBControllerSupport = 1 << 0,
        /// Supports LogManager API
        LogManagerSupport = 1 << 1,
        /// Supports ProfileManager API
        ProfileManagerSupport = 1 << 2,
        /// Supports PluginManager API
        PluginManagerSupport = 1 << 3,
        /// Supports SettingsManager API
        SettingsManagerSupport = 1 << 4,
        /// Set to request local client status. Server will then send local client flag.
        RequestLocalClient = 1 << 16,
    }
}

flags! {
    /// Server flags
    ///
    /// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation) for more information.
    pub enum ServerFlags: u32 {
        /// Supports RGBController API
        RGBControllerSupport = 1 << 0,
        /// Supports LogManager API
        LogManagerSupport = 1 << 1,
        /// Supports ProfileManager API
        ProfileManagerSupport = 1 << 2,
        /// Supports PluginManager API
        PluginManagerSupport = 1 << 3,
        /// Supports SettingsManager API
        SettingsManagerSupport = 1 << 4,
        /// Supports detection functions
        DetectionSupport = 1 << 5,
        /// Supports device info functions
        DeviceInfoSupport = 1 << 6,
        /// Confirm that client is local client
        LocalClient = 1 << 16,
    }
}

#[derive(Debug)]
#[allow(unused, reason = "Might be used later")]
pub(crate) struct DetectionProgressChange {
    pub(crate) detection_percent: u32,
    pub(crate) detection_string: String,
}

impl DeserFromBuf for DetectionProgressChange {
    fn deserialize(buf: &mut ReceivedMessage<'_>) -> crate::OpenRgbResult<Self> {
        let _data_size = buf.read_value::<u32>()?;
        let detection_percent = buf.read_value()?;
        let detection_string = buf.read_value()?;
        Ok(Self {
            detection_percent,
            detection_string,
        })
    }
}
