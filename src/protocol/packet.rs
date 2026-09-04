use crate::impl_enum_discriminant;

/// `OpenRGB` protocol packet ID.
///
/// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation#packet-ids) for more information.
#[derive(PartialEq, Debug, Copy, Clone)]
pub(crate) enum PacketId {
    /// Request `RGBController` device count from server.
    RequestControllerCount = 0,

    /// Request `RGBController` data block.
    RequestControllerData = 1,

    /// Acknowledgement
    Acknowledge = 10,

    /// Request `OpenRGB` SDK protocol version from server.
    RequestProtocolVersion = 40,

    /// Send client name string to server.
    SetClientName = 50,

    /// Send server name string to client
    SetServerName = 51,

    /// Send client flags to server
    SetClientFlags = 52,

    /// Send server flags to server
    SetServerFlags = 53,

    /// Indicate to clients that device list has updated.
    DeviceListUpdated = 100,

    /// Inidicate to clients that detection started
    DetectionStarted = 101,

    /// Inidicate to clients that detection progress changed
    DetectionProgressChanged = 102,

    /// Inidicate to clients that detection has completed
    DetectionComplete = 103,

    /// Request list of I2C bus info
    RequestI2cBusInfo = 120,
    /// Request list of HID device info
    RequestHidDeviceInfo = 121,
    /// Request list of USB device info
    RequestUsbDeviceInfo = 122,
    /// Request list of serial ports
    RequestListSerialPorts = 123,
    /// Request list of USB serial port info
    RequestListUsbPorts = 124,

    /// Request a device rescan. (Protocol 5)
    RequestDeviceRescan = 140,

    /// Request profile list. (Protocol 2)
    RequestProfileList = 150,

    /// Save current configuration in a new profile. (Protocol 2)
    RequestSaveProfile = 151,

    /// Load a given profile. (Protocol 2)
    RequestLoadProfile = 152,

    /// Delete a given profile. (Protocol 2)
    RequestDeleteProfile = 153,

    /// Upload a profile to the server in JSON format (Protocol 6)
    ProfileManagerUploadProfile = 154,

    /// Download a profile from the server in JSON format (Protocol 6)
    ProfileManagerDownloadProfile = 155,

    /// Get active profile name (Protocol 6)
    ProfileManagerGetActiveProfile = 156,

    /// Indicate to clients active profile has changed (Protocol 6)
    ProfileManagerActiveProfileChanged = 157,

    /// Notify active client that profile has loaded (Protocol 6)
    ///
    /// Server only
    ProfileManagerProfileLoaded = 158,

    /// Indicate to clients profile about to load (Protocol 6)
    ///
    /// Server only
    ProfileManagerProfileAboutToLoad = 159,

    /// Indicate to clients profile list updated (Protocol 6)
    ///
    /// Server only
    ProfileManagerProfileListUpdated = 160,

    /// Clears the active profile (Protocol 6)
    ProfileManagerClearActiveProfile = 161,

    /// Request list of plugins. (Protocol 4)
    RequestPluginList = 200,

    /// Plugin specific request. (Protocol 4)
    PluginSpecific = 201,

    /// Get settings for a given key in JSON format (Protocol 6)
    SettingsManagerGetSettings = 250,

    /// Get settings schema for a given key in JSON format (Protocol 6)
    SettingsManagerGetSettingsSchema = 251,

    /// Modify settings for a given key in JSON format (Protocol 6)
    SettingsManagerModifySettings = 252,

    /// Set settings for a given key in JSON format (Protocol 6)
    SettingsManagerSetSettings = 253,

    /// Save settings (Protocol 6)
    SettingsManagerSaveSettings = 254,

    /// `LogManager::ClearLogBuffer()` (Protocol 6)
    LogManagerClearLogBuffer = 300,

    /// `LogManager::GetLogBuffer()` (Protocol 6)
    LogManagerGetLogBuffer = 301,

    /// `LogManager::GetLogLevel()` (Protocol 6)
    LogManagerGetLogLevel = 302,

    /// `LogManager::SetLogLevel()` (Protocol 6)
    LogManagerSetLogLevel = 303,

    /// `LogManager::LogEntry` Callback (Protocol 6)
    LogManagerLoggedEntry = 304,
    /// `RGBController::ResizeZone()`.
    RGBControllerResizeZone = 1000,

    /// `RGBController::ClearSegments()`. (Protocol 5)
    RgbControllerClearSegments = 1001,

    /// `RGBController::AddSegment()`. (Protocol 5)
    RGBControllerAddSegment = 1002,

    /// `RGBController::ConfigureZone()` (Protocol 6)
    RGBControllerConfigureZone = 1003,
    /// `RGBController::ConfigureDevice()` (Protocol 6)
    RGBControllerConfigureDevice = 1004,
    /// `RGBController::SetHidden()` (Protocol 6)
    RGBControllerSetHidden = 1005,

    /// `RGBController::UpdateLEDs()`.
    RGBControllerUpdateLeds = 1050,

    /// `RGBController::UpdateZoneLEDs()`.
    RGBControllerUpdateZoneLeds = 1051,

    /// `RGBController::UpdateSingleLED()`.
    RGBControllerUpdateSingleLed = 1052,

    /// `RGBController::SetCustomMode()`.
    RGBControllerSetCustomMode = 1100,

    /// `RGBController::UpdateMode()`.
    RGBControllerUpdateMode = 1101,

    /// `RGBController::SaveMode()`. (Protocol 3)
    RGBControllerSaveMode = 1102,

    /// `RGBController::UpdateZoneMode()` (Protocol 6)
    RgbControllerUpdateZoneMode = 1103,

    /// `RGBController::SetDeviceSpecificConfiguration` (Protocol 6)
    RGBControllerSetDeviceSpecificConfiguration = 1130,

    /// `RGBController::SetDeviceSpecificZoneConfiguration` (Protocol 6)
    RGBControllerSetDeviceSpecificZoneConfiguration = 1131,

    /// `RGBController::SignalUpdate` (Protocol 6)
    RGBControllerSignalUpdate = 1150,
}

impl std::fmt::Display for PacketId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} ({})", self, u32::from(self))
    }
}

impl PacketId {
    pub fn expected_response(&self) -> Self {
        match self {
            Self::SetClientFlags => Self::SetServerFlags,
            p => *p,
        }
    }

    pub fn is_server_only(&self) -> bool {
        matches!(
            self,
            PacketId::SetServerName
                | PacketId::SetServerFlags
                | PacketId::DeviceListUpdated
                | PacketId::DetectionStarted
                | PacketId::DetectionProgressChanged
                | PacketId::DetectionComplete
                | PacketId::ProfileManagerActiveProfileChanged
                | PacketId::ProfileManagerProfileLoaded
                | PacketId::ProfileManagerProfileAboutToLoad
                | PacketId::ProfileManagerProfileListUpdated
                | PacketId::LogManagerLoggedEntry
                | PacketId::RGBControllerSignalUpdate
        )
    }
}

impl_enum_discriminant!(PacketId,
    RequestControllerCount: 0,
    RequestControllerData: 1,
    Acknowledge: 10,
    RequestProtocolVersion: 40,
    SetClientName: 50,
    SetServerName: 51,
    SetClientFlags: 52,
    SetServerFlags: 53,
    DeviceListUpdated: 100,
    DetectionStarted: 101,
    DetectionProgressChanged: 102,
    DetectionComplete: 103,
    RequestI2cBusInfo: 120,
    RequestHidDeviceInfo: 121,
    RequestUsbDeviceInfo: 122,
    RequestListSerialPorts: 123,
    RequestListUsbPorts: 124,
    RequestDeviceRescan: 140,
    RequestProfileList: 150,
    RequestSaveProfile: 151,
    RequestLoadProfile: 152,
    RequestDeleteProfile: 153,
    ProfileManagerUploadProfile: 154,
    ProfileManagerDownloadProfile: 155,
    ProfileManagerGetActiveProfile: 156,
    ProfileManagerActiveProfileChanged: 157,
    ProfileManagerProfileLoaded: 158,
    ProfileManagerProfileAboutToLoad: 159,
    ProfileManagerProfileListUpdated: 160,
    ProfileManagerClearActiveProfile: 161,
    RequestPluginList: 200,
    PluginSpecific: 201,
    SettingsManagerGetSettings: 250,
    SettingsManagerGetSettingsSchema: 251,
    SettingsManagerModifySettings: 252,
    SettingsManagerSetSettings: 253,
    SettingsManagerSaveSettings: 254,
    LogManagerClearLogBuffer: 300,
    LogManagerGetLogBuffer: 301,
    LogManagerGetLogLevel: 302,
    LogManagerSetLogLevel: 303,
    LogManagerLoggedEntry: 304,
    RGBControllerResizeZone: 1000,
    RgbControllerClearSegments: 1001,
    RGBControllerAddSegment: 1002,
    RGBControllerConfigureZone: 1003,
    RGBControllerConfigureDevice: 1004,
    RGBControllerSetHidden: 1005,
    RGBControllerUpdateLeds: 1050,
    RGBControllerUpdateZoneLeds: 1051,
    RGBControllerUpdateSingleLed: 1052,
    RGBControllerSetCustomMode: 1100,
    RGBControllerUpdateMode: 1101,
    RGBControllerSaveMode: 1102,
    RgbControllerUpdateZoneMode: 1103,
    RGBControllerSetDeviceSpecificConfiguration: 1130,
    RGBControllerSetDeviceSpecificZoneConfiguration: 1131,
    RGBControllerSignalUpdate: 1150
);

#[cfg(test)]
mod tests {
    use std::error::Error;

    use crate::{PacketId, WriteMessage};

    #[tokio::test]
    async fn test_read_001() -> Result<(), Box<dyn Error>> {
        let mut buf = WriteMessage::new(crate::DEFAULT_PROTOCOL);
        let mut msg = buf.push_value(152_u32)?.to_received_msg();

        assert_eq!(msg.read_value::<PacketId>()?, PacketId::RequestLoadProfile);
        Ok(())
    }

    #[tokio::test]
    async fn test_write_001() -> Result<(), Box<dyn Error>> {
        let mut buf = WriteMessage::new(crate::DEFAULT_PROTOCOL);
        let mut msg = buf
            .push_value(PacketId::RequestLoadProfile)?
            .to_received_msg();
        assert_eq!(msg.read_value::<u32>()?, 152);
        Ok(())
    }
}
