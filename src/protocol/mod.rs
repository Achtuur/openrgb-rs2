use std::net::Ipv4Addr;
use std::sync::Arc;
use std::{fmt::Debug, time::Duration};

use flagset::FlagSet;
use tokio::net::ToSocketAddrs;
use tokio::sync::Mutex;

use super::data::{Color, ControllerData, ModeData, RawString, SegmentData};
use crate::{
    AutoStartSettings, ClientFlags, ControllerIds, DetectionProgressChange, DetectorSettings,
    EffectsPluginPacket, HidDeviceList, I2cBusList, Json, LogEntry, LogManagerSettings,
    OpenRgbError, OpenRgbLogLevel, OpenRgbResult, PluginData, PluginEffect, ProfileData,
    ProfileManagerSettings, SerialPortList, ServerFlags, ServerSettings, SettingsKey,
    SettingsSchema, UiSettings, UsbDeviceList, ZoneData,
};

/// Default protocol version used by the [`crate::OpenRgbClient::connect`].
pub const DEFAULT_PROTOCOL: u32 = 6;

/// Default address used by [`crate::OpenRgbClient::connect`].
pub const DEFAULT_ADDR: (Ipv4Addr, u16) = (Ipv4Addr::LOCALHOST, 6742);

/// Device ID to use when no specific device is targeted.
const NO_DEVICE_ID: u32 = 0;

const NO_DATA: &() = &();

pub mod data;
mod deserialize;
mod packet;
mod serialize;
mod stream;

pub(crate) use {deserialize::*, packet::*, serialize::*, stream::*};

/// `OpenRGB` client.
///
/// This struct makes sure the `protocol_id` and the stream are in sync.
///
/// Todo: reintroduce a generic `stream` type to support sync/async streams.
#[derive(Clone)]
pub(crate) struct OpenRgbProtocol {
    protocol_id: u32,
    stream: Arc<Mutex<ProtocolStream>>,
}

impl OpenRgbProtocol {
    /// Connect to `OpenRGB` server at given address with given protocol version.
    pub async fn connect_to(
        addr: impl ToSocketAddrs + Debug + Copy,
        protocol_version: u32,
    ) -> OpenRgbResult<Self> {
        tracing::debug!("Connecting to OpenRGB server at {:?}...", addr);
        let stream = ProtocolStream::connect(addr, protocol_version)
            .await
            .map_err(|source| OpenRgbError::ConnectionError {
                addr: format!("{addr:?}"),
                source,
            })?;
        Self::new(stream).await
    }
}

impl OpenRgbProtocol {
    /// Build a new client from given stream.
    ///
    /// This constructor expects a connected, ready to use stream.
    pub async fn new(mut stream: ProtocolStream) -> OpenRgbResult<Self> {
        let req_protocol = stream
            .request(
                NO_DEVICE_ID,
                PacketId::RequestProtocolVersion,
                &DEFAULT_PROTOCOL,
            )
            .await?;
        let protocol = DEFAULT_PROTOCOL.min(req_protocol);

        tracing::debug!(
            "Connected to OpenRGB server using protocol version {:?}",
            protocol
        );
        stream.set_protocol_version(protocol);

        let proto = Self {
            protocol_id: protocol,
            stream: Arc::new(Mutex::new(stream)),
        };

        if proto.protocol_id >= 6 {
            let _server_flags = proto
                .set_client_flags(
                    ClientFlags::RGBControllerSupport
                        | ClientFlags::LogManagerSupport
                        | ClientFlags::ProfileManagerSupport
                        | ClientFlags::PluginManagerSupport
                        | ClientFlags::SettingsManagerSupport
                        | ClientFlags::RequestLocalClient,
                )
                .await?;
        }
        Ok(proto)
    }

    /// Get protocol version negotiated with server.
    ///
    /// This is the lowest between this client maximum supported version ([`DEFAULT_PROTOCOL`]) and server version.
    ///
    /// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation#protocol-versions) for more information.
    pub fn get_protocol_version(&self) -> u32 {
        self.protocol_id
    }

    /// Write a packet to the server, without expecting a response
    async fn write<T: SerToBuf>(
        &self,
        device_id: u32,
        packet_id: PacketId,
        data: T,
    ) -> OpenRgbResult<()> {
        self.stream
            .lock()
            .await
            .write(device_id, packet_id, data)
            .await
    }

    #[must_use = "Unused packet is useless"]
    async fn recv_packet(&self) -> OpenRgbResult<RecvPacket> {
        self.stream.lock().await.recv_packet().await
    }

    /// Write a packet to the server and wait for the response.
    #[must_use = "Result should be used"]
    async fn request<I: SerToBuf, O: DeserFromBuf>(
        &self,
        device_id: u32,
        packet_id: PacketId,
        data: I,
    ) -> OpenRgbResult<O> {
        self.stream
            .lock()
            .await
            .request(device_id, packet_id, data)
            .await
    }

    /// Set client name.
    ///
    /// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation#net_packet_id_set_client_name) for more information.
    pub async fn set_name(&self, name: impl Into<String>) -> OpenRgbResult<()> {
        self.write(
            NO_DEVICE_ID,
            PacketId::SetClientName,
            &RawString(&name.into()),
        )
        .await
    }

    /// Get number of controllers. Use only for protocol v5 and under. For v6 and up, use [`get_controller_ids()`] instead.
    /// This is because this packet has chagned in function between v5 and v6.
    ///
    /// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation#net_packet_id_request_controller_count) for more information.
    pub async fn get_controller_count(&self) -> OpenRgbResult<u32> {
        assert!(self.protocol_id < 6, "Us");
        self.request(NO_DEVICE_ID, PacketId::RequestControllerCount, NO_DATA)
            .await
    }

    /// Gets the id's of all controllers
    pub async fn get_controller_ids(&self) -> OpenRgbResult<ControllerIds> {
        self.check_protocol_version(6, "get_controller_ids")?;
        self.request(NO_DEVICE_ID, PacketId::RequestControllerCount, NO_DATA)
            .await
    }

    /// Get controller data. This also caches the obtained controller.
    ///
    /// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation#net_packet_id_request_controller_data) for more information.
    pub async fn get_controller(&self, controller_id: u32) -> OpenRgbResult<ControllerData> {
        let mut c: ControllerData = self
            .request(
                controller_id,
                PacketId::RequestControllerData,
                &self.protocol_id,
            )
            .await?;
        c.set_id(controller_id);
        Ok(c)
    }

    /// Resize a controller zone.
    ///
    /// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation#net_packet_id_rgbcontroller_resizezone) for more information.
    pub async fn resize_zone(
        &self,
        controller_id: u32,
        zone_id: u32,
        new_size: u32,
    ) -> OpenRgbResult<()> {
        self.write(
            controller_id,
            PacketId::RGBControllerResizeZone,
            &(zone_id, new_size),
        )
        .await
    }

    /// Update a single LED.
    ///
    /// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation#net_packet_id_rgbcontroller_updatesingleled) for more information.
    pub async fn update_led(
        &self,
        controller_id: u32,
        led_id: i32,
        color: &Color,
    ) -> OpenRgbResult<()> {
        self.write(
            controller_id,
            PacketId::RGBControllerUpdateSingleLed,
            (led_id, color),
        )
        .await
    }

    /// Update LEDs.
    ///
    /// Structure:
    /// - `u32` - data size
    /// - `u16` - color counts
    /// - `[u32]` - colors
    ///
    /// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation#net_packet_id_rgbcontroller_updateleds) for more information.
    pub async fn update_leds(&self, controller_id: u32, colors: &[Color]) -> OpenRgbResult<()> {
        let packet = OpenRgbWritePacket::new(colors);
        self.write(controller_id, PacketId::RGBControllerUpdateLeds, &packet)
            .await
    }

    /// Update a zone LEDs.
    ///
    /// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation#net_packet_id_rgbcontroller_updatezoneleds) for more information.
    pub async fn update_zone_leds(
        &self,
        controller_id: u32,
        zone_id: u32,
        colors: &[Color],
    ) -> OpenRgbResult<()> {
        let packet = OpenRgbWritePacket::new((zone_id, colors));
        self.write(
            controller_id,
            PacketId::RGBControllerUpdateZoneLeds,
            &packet,
        )
        .await
    }

    /// Update a mode. This sets it to the current mode.
    ///
    /// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation#net_packet_id_rgbcontroller_updatemode) for more information.
    pub async fn update_mode(&self, controller_id: u32, mode: &ModeData) -> OpenRgbResult<()> {
        let packet = OpenRgbWritePacket::new((mode.id() as u32, mode));
        self.write(controller_id, PacketId::RGBControllerUpdateMode, &packet)
            .await
    }

    /// Set custom mode.
    ///
    /// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation#net_packet_id_rgbcontroller_setcustommode) for more information.
    #[expect(unused, reason = "Recommendation from OpenRGB dev is to not use this")] // unused on purpose
    pub async fn set_custom_mode(&self, controller_id: u32) -> OpenRgbResult<()> {
        unimplemented!(
            "Not implemented as per recommendation from OpenRGB devs (https://discord.com/channels/699861463375937578/709998213310054490/1372954035581096158)"
        );
        // self
        //     .write_packet(controller_id, PacketId::RGBControllerSetCustomMode, NO_DATA)
        //     .await
    }

    #[allow(unused, reason = "Might be used later")]
    async fn configure_zone(&self, controller_id: u32, zone_data: &ZoneData) -> OpenRgbResult<()> {
        let packet = OpenRgbWritePacket::new((zone_data.id() as u32, zone_data));
        self.write(controller_id, PacketId::RGBControllerConfigureZone, packet)
            .await
    }

    #[allow(unused, reason = "Might be used later")]
    async fn update_zone_mode(
        &self,
        controller_id: u32,
        zone_id: u32,
        mode_data: &ModeData,
    ) -> OpenRgbResult<()> {
        let packet = OpenRgbWritePacket::new((zone_id, mode_data.id() as u32, mode_data));
        self.write(controller_id, PacketId::RgbControllerUpdateZoneMode, packet)
            .await
    }

    #[allow(unused, reason = "Might be used later")]
    async fn set_controller_hidden(&self, controller_id: u32, hidden: bool) -> OpenRgbResult<()> {
        self.write(
            controller_id,
            PacketId::RGBControllerSetHidden,
            u8::from(hidden),
        )
        .await
    }

    /// Get profiles.
    ///
    /// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation#net_packet_id_request_profile_list) for more information.
    pub async fn get_profiles(&self) -> OpenRgbResult<Vec<String>> {
        self.check_protocol_version(2, "Get profiles")?;
        self.request::<_, (u32, Vec<String>)>(0, PacketId::RequestProfileList, NO_DATA)
            .await
            .map(|(_size, profiles)| profiles)
    }

    /// Load a profile.
    ///
    /// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation#net_packet_id_request_load_profile) for more information.
    pub async fn load_profile(&self, name: impl Into<String>) -> OpenRgbResult<()> {
        self.check_protocol_version(2, "Load profiles")?;
        self.write(0, PacketId::RequestLoadProfile, &RawString(&name.into()))
            .await
    }

    /// Save a profile.
    ///
    /// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation#net_packet_id_request_save_profile) for more information.
    pub async fn save_profile(&self, name: impl Into<String>) -> OpenRgbResult<()> {
        self.check_protocol_version(2, "Save profiles")?;
        self.write(0, PacketId::RequestSaveProfile, &name.into())
            .await
    }

    /// Delete a profile.
    ///
    /// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation#net_packet_id_request_delete_profile) for more information.
    pub async fn delete_profile(&self, name: impl Into<String>) -> OpenRgbResult<()> {
        self.check_protocol_version(2, "Delete profiles")?;
        self.write(0, PacketId::RequestDeleteProfile, &name.into())
            .await
    }

    /// Save a mode.
    ///
    /// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation#net_packet_id_rgbcontroller_savemode) for more information.
    pub async fn save_mode(&self, controller_id: u32, mode: &ModeData) -> OpenRgbResult<()> {
        self.check_protocol_version(3, "Save mode")?;
        let packet = OpenRgbWritePacket::new((mode.id() as u32, mode));
        self.write(controller_id, PacketId::RGBControllerSaveMode, &packet)
            .await
    }

    /// Returns a a list of names of installed plugins.
    pub async fn get_plugins(&self) -> OpenRgbResult<Vec<PluginData>> {
        self.check_protocol_version(4, "Request Plugin List")?;
        // response contains length of data in the packet
        let resp: (u32, Vec<_>) = self
            .request(NO_DEVICE_ID, PacketId::RequestPluginList, NO_DATA)
            .await?;
        Ok(resp.1)
    }

    /// Performs a plugin specific command. Depends on the plugin what this does.
    ///
    /// In this case, the `pkt_dev_idx` (`controller_id`) is used as the Plugin ID.
    pub async fn plugin_specific_receive<I, O>(
        &self,
        plugin_id: u32,
        header: u32,
        data: &I,
    ) -> OpenRgbResult<O>
    where
        I: SerToBuf,
        O: DeserFromBuf,
    {
        self.check_protocol_version(4, "Plugin Specific Command")?;
        let (recv_header, resp): (u32, O) = self
            .request(plugin_id, PacketId::PluginSpecific, &(header, data))
            .await?;
        if header != recv_header {
            return Err(OpenRgbError::ProtocolError(format!(
                "Plugin Specific Command header mismatch: expected {header}, got {recv_header}"
            )));
        }
        Ok(resp)
    }

    pub async fn plugin_specific_write_packet<I>(
        &self,
        plugin_id: u32,
        header: u32,
        data: &I,
    ) -> OpenRgbResult<()>
    where
        I: SerToBuf,
    {
        self.check_protocol_version(4, "Plugin Specific Command")?;
        self.write(plugin_id, PacketId::PluginSpecific, &(header, data))
            .await
    }

    pub async fn add_segment(
        &self,
        controller_id: u32,
        zone_id: u32,
        segment: &SegmentData,
    ) -> OpenRgbResult<()> {
        // segments are version 4, segments commands are version 5
        self.check_protocol_version(5, "Add Segment")?;
        let packet = OpenRgbWritePacket::new((zone_id, segment));
        self.write(controller_id, PacketId::RGBControllerAddSegment, &packet)
            .await
    }

    pub async fn clear_segments(&self, controller_id: u32) -> OpenRgbResult<()> {
        self.check_protocol_version(5, "Clear segment")?;
        self.write(controller_id, PacketId::RgbControllerClearSegments, NO_DATA)
            .await
    }

    /// Request a device rescan.
    pub async fn rescan_devices(&self, timeout: Duration) -> OpenRgbResult<()> {
        self.check_protocol_version(6, "Rescan devices")?;
        self.stream
            .lock()
            .await
            .write_packet(NO_DEVICE_ID, PacketId::RequestDeviceRescan, NO_DATA)
            .await?;
        // self.write(NO_DEVICE_ID, PacketId::RequestDeviceRescan, NO_DATA)
        //     .await?;

        loop {
            let packet = tokio::time::timeout(timeout, self.recv_packet())
                .await
                .map_err(|_| OpenRgbError::ProtocolError("Timeout elapsed".to_owned()))??;
            match packet.header.packet_id {
                PacketId::DetectionStarted => {
                    tracing::info!("Detection started");
                    println!("Detection started")
                }
                PacketId::DetectionProgressChanged => {
                    let change = packet.deser::<DetectionProgressChange>(6)?;
                    tracing::info!("Detection progress changed: {change:?}");
                    println!("Detection progress changed: {change:?}");
                }
                PacketId::DetectionComplete => {
                    tracing::info!("Detection complete");
                    println!("Detection complete");
                    break;
                }
                PacketId::LogManagerLoggedEntry => {
                    let log = packet.deser::<LogEntry>(self.protocol_id)?;
                    println!("log: {0:?}", log);
                }
                _ => {
                    return Err(OpenRgbError::ProtocolError(
                        "Invalid packet received".to_owned(),
                    ));
                }
            }
        }
        Ok(())
    }

    fn check_protocol_version(&self, min: u32, msg: &str) -> OpenRgbResult<()> {
        if self.protocol_id < min {
            return Err(OpenRgbError::UnsupportedOperation {
                operation: msg.to_owned(),
                current_protocol_version: self.protocol_id,
                min_protocol_version: min,
            });
        }
        Ok(())
    }

    /* EFFECTS PLUGIN */

    #[expect(unused, reason = "Plugin effect api todo")]
    pub async fn effect_plugin_get_effects(
        &self,
        effects_plugin_id: u32,
    ) -> OpenRgbResult<Vec<PluginEffect>> {
        let (_data_size, list): (u32, Vec<_>) = self
            .plugin_specific_receive(
                effects_plugin_id,
                EffectsPluginPacket::RequestEffectList.into(),
                NO_DATA,
            )
            .await?;
        Ok(list)
    }

    #[expect(unused, reason = "Plugin effect api todo")]
    pub async fn effect_plugin_start_effect(
        &self,
        effect_plugin_id: u32,
        effect_name: &str,
    ) -> OpenRgbResult<()> {
        self.plugin_specific_write_packet(
            effect_plugin_id,
            EffectsPluginPacket::StartEffect.into(),
            &effect_name,
        )
        .await
    }

    #[expect(unused, reason = "Plugin effect api todo")]
    pub async fn effect_plugin_stop_effect(
        &self,
        effect_plugin_id: u32,
        effect_name: &str,
    ) -> OpenRgbResult<()> {
        self.plugin_specific_write_packet(
            effect_plugin_id,
            EffectsPluginPacket::StopEffect.into(),
            &effect_name,
        )
        .await
    }
}

// Protocol v6
impl OpenRgbProtocol {
    pub async fn set_client_flags(
        &self,
        flags: FlagSet<ClientFlags>,
    ) -> OpenRgbResult<FlagSet<ServerFlags>> {
        self.check_protocol_version(6, "set_client_flags")?;
        self.request(NO_DEVICE_ID, PacketId::SetClientFlags, &flags)
            .await
    }

    #[allow(unused, reason = "Might be used later")]
    async fn get_i2c_bus_info(&self) -> OpenRgbResult<I2cBusList> {
        self.check_protocol_version(6, "get_i2c_bus_info")?;
        self.request(NO_DEVICE_ID, PacketId::RequestI2cBusInfo, NO_DATA)
            .await
    }

    #[allow(unused, reason = "Might be used later")]
    async fn get_hid_device_info(&self) -> OpenRgbResult<HidDeviceList> {
        self.check_protocol_version(6, "get_hid_device_info")?;
        self.request(NO_DEVICE_ID, PacketId::RequestHidDeviceInfo, NO_DATA)
            .await
    }

    #[allow(unused, reason = "Might be used later")]
    async fn get_usb_device_info(&self) -> OpenRgbResult<UsbDeviceList> {
        self.check_protocol_version(6, "get_usb_device_info")?;
        self.request(NO_DEVICE_ID, PacketId::RequestUsbDeviceInfo, NO_DATA)
            .await
    }

    #[allow(unused, reason = "Might be used later")]
    async fn get_serial_ports(&self) -> OpenRgbResult<SerialPortList> {
        self.check_protocol_version(6, "get_serial_ports")?;
        self.request(NO_DEVICE_ID, PacketId::RequestListSerialPorts, NO_DATA)
            .await
    }
}

// Profilemanager
impl OpenRgbProtocol {
    #[allow(unused, reason = "Might be used later")]
    async fn upload_profile(&self, profile: &ProfileData) -> OpenRgbResult<()> {
        self.check_protocol_version(6, "upload_profile")?;
        self.write(NO_DEVICE_ID, PacketId::ProfileManagerUploadProfile, profile)
            .await?;
        Ok(())
    }

    #[allow(unused, reason = "Might be used later")]
    async fn download_profile(&self, profile_name: &str) -> OpenRgbResult<ProfileData> {
        self.check_protocol_version(6, "download_profile")?;
        self.request(
            NO_DEVICE_ID,
            PacketId::ProfileManagerDownloadProfile,
            &profile_name,
        )
        .await
    }

    #[allow(unused, reason = "Might be used later")]
    async fn get_active_profile(&self) -> OpenRgbResult<String> {
        self.check_protocol_version(6, "get_active_profile")?;
        self.request(
            NO_DEVICE_ID,
            PacketId::ProfileManagerGetActiveProfile,
            NO_DATA,
        )
        .await
    }

    #[allow(unused, reason = "Might be used later")]
    async fn clear_active_profile(&self) -> OpenRgbResult<String> {
        self.check_protocol_version(6, "clear_active_profile")?;
        self.request(
            NO_DEVICE_ID,
            PacketId::ProfileManagerClearActiveProfile,
            NO_DATA,
        )
        .await
    }
}

// Settingsmanager
impl OpenRgbProtocol {
    #[allow(unused, reason = "Might be used later")]
    async fn get_schema(&self, key: &SettingsKey) -> OpenRgbResult<SettingsSchema> {
        self.check_protocol_version(6, "get_schema")?;
        self.request(
            NO_DEVICE_ID,
            PacketId::SettingsManagerGetSettingsSchema,
            RawString(key.as_str()),
        )
        .await
    }

    /// Reads a settings key and deserializes it to T
    async fn get_settings<T: DeserFromBuf>(&self, key: &SettingsKey) -> OpenRgbResult<T> {
        self.check_protocol_version(6, "get_settings")?;
        self.request(
            NO_DEVICE_ID,
            PacketId::SettingsManagerGetSettings,
            RawString(key.as_str()),
        )
        .await
    }

    async fn set_settings<T: SerToBuf>(&self, key: &SettingsKey, data: T) -> OpenRgbResult<()> {
        self.check_protocol_version(6, "set_settings")?;
        self.write(
            NO_DEVICE_ID,
            PacketId::SettingsManagerSetSettings,
            (RawString(key.as_str()), data),
        )
        .await
    }

    async fn modify_settings<T: SerToBuf>(&self, key: &SettingsKey, data: T) -> OpenRgbResult<()> {
        self.check_protocol_version(6, "set_settings")?;
        self.write(
            NO_DEVICE_ID,
            PacketId::SettingsManagerSetSettings,
            (RawString(key.as_str()), data),
        )
        .await
    }

    #[allow(unused, reason = "Might be used later")]
    async fn save_settings(&self) -> OpenRgbResult<()> {
        self.check_protocol_version(6, "save_settings")?;
        self.write(NO_DEVICE_ID, PacketId::SettingsManagerSaveSettings, ())
            .await
    }

    #[allow(unused, reason = "Might be used later")]
    async fn get_autostart_settings(&self) -> OpenRgbResult<AutoStartSettings> {
        self.get_settings::<AutoStartSettings>(&SettingsKey::AutoStart)
            .await
    }

    #[allow(unused, reason = "Might be used later")]
    async fn set_autostart_settings(&self, data: &AutoStartSettings) -> OpenRgbResult<()> {
        self.set_settings(&SettingsKey::AutoStart, data).await
    }

    #[allow(unused, reason = "Might be used later")]
    async fn modify_autostart_settings(&self, data: &AutoStartSettings) -> OpenRgbResult<()> {
        self.modify_settings(&SettingsKey::AutoStart, data).await
    }

    #[allow(unused, reason = "Might be used later")]
    async fn get_detector_settings(&self) -> OpenRgbResult<DetectorSettings> {
        self.get_settings(&SettingsKey::Detectors).await
    }

    #[allow(unused, reason = "Might be used later")]
    async fn set_detector_settings(&self, data: &DetectorSettings) -> OpenRgbResult<()> {
        self.set_settings(&SettingsKey::Detectors, data).await
    }

    #[allow(unused, reason = "Might be used later")]
    async fn modify_detector_settings(&self, data: &DetectorSettings) -> OpenRgbResult<()> {
        self.modify_settings(&SettingsKey::Detectors, data).await
    }

    #[allow(unused, reason = "Might be used later")]
    async fn get_log_manager_settings(&self) -> OpenRgbResult<LogManagerSettings> {
        self.get_settings(&SettingsKey::LogManager).await
    }

    #[allow(unused, reason = "Might be used later")]
    async fn set_log_manager_settings(&self, data: &LogManagerSettings) -> OpenRgbResult<()> {
        self.set_settings(&SettingsKey::LogManager, data).await
    }

    #[allow(unused, reason = "Might be used later")]
    async fn modify_log_manager_settings(&self, data: &LogManagerSettings) -> OpenRgbResult<()> {
        self.modify_settings(&SettingsKey::LogManager, data).await
    }

    #[allow(unused, reason = "Might be used later")]
    async fn get_profile_manager_settings(&self) -> OpenRgbResult<ProfileManagerSettings> {
        self.get_settings(&SettingsKey::ProfileManager).await
    }

    #[allow(unused, reason = "Might be used later")]
    async fn set_profile_manager_settings(
        &self,
        data: &ProfileManagerSettings,
    ) -> OpenRgbResult<()> {
        self.set_settings(&SettingsKey::ProfileManager, data).await
    }

    #[allow(unused, reason = "Might be used later")]
    async fn modify_profile_manager_settings(
        &self,
        data: &ProfileManagerSettings,
    ) -> OpenRgbResult<()> {
        self.modify_settings(&SettingsKey::ProfileManager, data)
            .await
    }

    #[allow(unused, reason = "Might be used later")]
    async fn get_server_settings(&self) -> OpenRgbResult<ServerSettings> {
        self.get_settings(&SettingsKey::Server).await
    }

    #[allow(unused, reason = "Might be used later")]
    async fn set_server_settings(&self, data: &ServerSettings) -> OpenRgbResult<()> {
        self.set_settings(&SettingsKey::Server, data).await
    }

    #[allow(unused, reason = "Might be used later")]
    async fn modify_server_settings(&self, data: &ServerSettings) -> OpenRgbResult<()> {
        self.modify_settings(&SettingsKey::Server, data).await
    }

    #[allow(unused, reason = "Might be used later")]
    async fn get_user_interface_settings(&self) -> OpenRgbResult<UiSettings> {
        self.get_settings(&SettingsKey::UserInterface).await
    }

    #[allow(unused, reason = "Might be used later")]
    async fn set_user_interface_settings(&self, data: &UiSettings) -> OpenRgbResult<()> {
        self.set_settings(&SettingsKey::UserInterface, data).await
    }

    #[allow(unused, reason = "Might be used later")]
    async fn modify_user_interface_settings(&self, data: &UiSettings) -> OpenRgbResult<()> {
        self.modify_settings(&SettingsKey::UserInterface, data)
            .await
    }

    #[allow(
        unused,
        reason = "Not supporting this for now, it's very controller specific and sdk devs can't really use this"
    )]
    async fn set_device_specific_config(
        &self,
        device_id: u32,
        data: &serde_json::Value,
    ) -> OpenRgbResult<()> {
        self.write(
            device_id,
            PacketId::RGBControllerSetDeviceSpecificConfiguration,
            Json(data),
        )
        .await
    }

    #[allow(
        unused,
        reason = "Not supporting this for now, it's very controller specific and sdk devs can't really use this"
    )]
    async fn set_device_specific_zone_config(
        &self,
        device_id: u32,
        zone_id: u32,
        data: &serde_json::Value,
    ) -> OpenRgbResult<()> {
        let json_string = serde_json::to_string(data)?;
        let packet = OpenRgbWritePacket::new((zone_id, json_string));
        self.write(
            device_id,
            PacketId::RGBControllerSetDeviceSpecificConfiguration,
            packet,
        )
        .await
    }
}

// Logmanager
impl OpenRgbProtocol {
    #[allow(unused, reason = "Might be used later")]
    async fn clear_log_buffer(&self) -> OpenRgbResult<()> {
        self.write(NO_DEVICE_ID, PacketId::LogManagerClearLogBuffer, ())
            .await
    }

    #[allow(unused, reason = "Might be used later")]
    async fn get_log_buffer(&self) -> OpenRgbResult<Vec<u8>> {
        self.request(NO_DEVICE_ID, PacketId::LogManagerGetLogBuffer, ())
            .await
    }

    #[allow(unused, reason = "Might be used later")]
    async fn get_log_level(&self) -> OpenRgbResult<OpenRgbLogLevel> {
        self.request(NO_DEVICE_ID, PacketId::LogManagerGetLogLevel, ())
            .await
    }
}

#[cfg(test)]
mod tests {

    use std::time::Duration;

    use crate::{SegmentData, ZoneFlags};
    use flagset::FlagSet;
    use tracing_test::traced_test;

    use crate::{
        // protocol::tests::{setup, OpenRGBMockBuilder},
        Color,
        DEFAULT_ADDR,
        DEFAULT_PROTOCOL,
        OpenRgbProtocol,
        OpenRgbResult,
    };

    // create test methods for each of the OpenRGBProtocol methods

    #[tokio::test]
    #[traced_test]
    #[ignore = "can only test with openrgb running"]
    async fn test_set_name() -> OpenRgbResult<()> {
        let client = OpenRgbProtocol::connect_to(DEFAULT_ADDR, DEFAULT_PROTOCOL).await?;
        client.set_name("TestClient").await?;
        Ok(())
    }

    #[tokio::test]
    #[traced_test]
    #[ignore = "can only test with openrgb running"]
    async fn test_set_client_flags() -> OpenRgbResult<()> {
        let client = OpenRgbProtocol::connect_to(DEFAULT_ADDR, DEFAULT_PROTOCOL).await?;
        let flags = client.set_client_flags(FlagSet::full()).await?;
        println!("flags: {0:?}", flags);
        Ok(())
    }

    #[tokio::test]
    #[traced_test]
    #[ignore = "can only test with openrgb running"]
    async fn test_get_controller_count() -> OpenRgbResult<()> {
        let client = OpenRgbProtocol::connect_to(DEFAULT_ADDR, DEFAULT_PROTOCOL).await?;
        let count = client.get_controller_count().await?;
        assert!(count > 0);
        Ok(())
    }

    #[tokio::test]
    #[traced_test]
    #[ignore = "can only test with openrgb running"]
    async fn test_get_controller_ids() -> OpenRgbResult<()> {
        let client = OpenRgbProtocol::connect_to(DEFAULT_ADDR, DEFAULT_PROTOCOL).await?;
        let count = client.get_controller_ids().await?;
        assert!(!count.ids.is_empty());
        Ok(())
    }

    #[tokio::test]
    #[traced_test]
    #[ignore = "can only test with openrgb running"]
    async fn test_get_controller() -> OpenRgbResult<()> {
        let client = OpenRgbProtocol::connect_to(DEFAULT_ADDR, DEFAULT_PROTOCOL).await?;
        let count = client.get_controller_count().await?;
        if count > 0 {
            let controller = client.get_controller(0).await?;
            println!("controller: {0:#?}", controller);
            assert_eq!(controller.id(), 0);
        }
        Ok(())
    }

    #[tokio::test]
    #[traced_test]
    #[ignore = "can only test with openrgb running"]
    async fn test_get_controller_v6() -> OpenRgbResult<()> {
        let client = OpenRgbProtocol::connect_to(DEFAULT_ADDR, DEFAULT_PROTOCOL).await?;
        let ids = client.get_controller_ids().await?;
        for id in ids.ids {
            let controller = client.get_controller(id).await?;
            println!("controller: {0:#?}", controller.name());
            assert_eq!(controller.id(), id);
        }
        Ok(())
    }

    #[tokio::test]
    #[traced_test]
    #[ignore = "can only test with openrgb running"]
    async fn test_resize_zone() -> OpenRgbResult<()> {
        let client = OpenRgbProtocol::connect_to(DEFAULT_ADDR, DEFAULT_PROTOCOL).await?;
        // These IDs may need to be adjusted for your setup
        let _ = client.resize_zone(0, 0, 10).await;
        Ok(())
    }

    #[tokio::test]
    #[traced_test]
    #[ignore = "can only test with openrgb running"]
    async fn test_update_zone_leds() -> OpenRgbResult<()> {
        let client = OpenRgbProtocol::connect_to(DEFAULT_ADDR, DEFAULT_PROTOCOL).await?;
        let colors = vec![Color::new(0, 255, 0); 5];
        let _ = client.update_zone_leds(0, 0, &colors).await;
        Ok(())
    }

    #[tokio::test]
    #[traced_test]
    #[ignore = "can only test with openrgb running"]
    async fn test_update_mode() -> OpenRgbResult<()> {
        let client = OpenRgbProtocol::connect_to(DEFAULT_ADDR, DEFAULT_PROTOCOL).await?;
        let controller = client.get_controller(0).await?;
        if let Some(mode) = controller.modes().first() {
            let _ = client.update_mode(0, mode).await;
        }
        Ok(())
    }

    #[tokio::test]
    #[traced_test]
    #[ignore = "can only test with openrgb running"]
    async fn test_get_profiles() -> OpenRgbResult<()> {
        let client = OpenRgbProtocol::connect_to(DEFAULT_ADDR, DEFAULT_PROTOCOL).await?;
        let profiles = client.get_profiles().await?;
        println!("profiles: {0:?}", profiles);
        Ok(())
    }

    #[tokio::test]
    #[traced_test]
    #[ignore = "can only test with openrgb running"]
    async fn test_save_profile() -> OpenRgbResult<()> {
        let client = OpenRgbProtocol::connect_to(DEFAULT_ADDR, DEFAULT_PROTOCOL).await?;
        let _ = client.save_profile("test_profile").await;
        Ok(())
    }

    #[tokio::test]
    #[traced_test]
    #[ignore = "can only test with openrgb running"]
    async fn test_load_profile() -> OpenRgbResult<()> {
        let client = OpenRgbProtocol::connect_to(DEFAULT_ADDR, DEFAULT_PROTOCOL).await?;
        let _ = client.load_profile("test_profile").await;
        Ok(())
    }

    #[tokio::test]
    #[traced_test]
    #[ignore = "can only test with openrgb running"]
    async fn test_delete_profile() -> OpenRgbResult<()> {
        let client = OpenRgbProtocol::connect_to(DEFAULT_ADDR, DEFAULT_PROTOCOL).await?;
        let _ = client.delete_profile("test_profile").await;
        Ok(())
    }

    #[tokio::test]
    #[traced_test]
    #[ignore = "can only test with openrgb running"]
    async fn test_save_mode() -> OpenRgbResult<()> {
        let client = OpenRgbProtocol::connect_to(DEFAULT_ADDR, DEFAULT_PROTOCOL).await?;
        let controller = client.get_controller(0).await?;
        if let Some(mode) = controller.modes().first() {
            let _ = client.save_mode(0, mode).await;
        }
        Ok(())
    }

    #[tokio::test]
    #[traced_test]
    #[ignore = "can only test with openrgb running"]
    async fn test_get_plugins() -> OpenRgbResult<()> {
        let client = OpenRgbProtocol::connect_to(DEFAULT_ADDR, DEFAULT_PROTOCOL).await?;
        let _ = client.get_plugins().await?;
        Ok(())
    }

    #[tokio::test]
    #[traced_test]
    #[ignore = "can only test with openrgb running"]
    async fn test_add_segment() -> OpenRgbResult<()> {
        let client = OpenRgbProtocol::connect_to(DEFAULT_ADDR, DEFAULT_PROTOCOL).await?;
        let segment = SegmentData::new("TestSegment", 0, 1);
        let _ = client.add_segment(0, 0, &segment).await;
        Ok(())
    }

    #[tokio::test]
    #[traced_test]
    #[ignore = "can only test with openrgb running"]
    async fn test_clear_segments() -> OpenRgbResult<()> {
        let client = OpenRgbProtocol::connect_to(DEFAULT_ADDR, DEFAULT_PROTOCOL).await?;
        let _ = client.clear_segments(0).await;
        Ok(())
    }

    #[tokio::test]
    // #[traced_test]
    #[ignore = "can only test with openrgb running"]
    async fn test_rescan_devices() -> OpenRgbResult<()> {
        let client = OpenRgbProtocol::connect_to(DEFAULT_ADDR, DEFAULT_PROTOCOL).await?;
        let timer = std::time::Instant::now();
        client.rescan_devices(Duration::from_secs(10)).await?;
        println!("{:?}", timer.elapsed());
        Ok(())
    }

    #[tokio::test]
    #[traced_test]
    #[ignore = "can only test with openrgb running"]
    async fn test_connect() -> OpenRgbResult<()> {
        let _client = OpenRgbProtocol::connect_to(DEFAULT_ADDR, DEFAULT_PROTOCOL).await?;
        Ok(())
    }

    #[tokio::test]
    #[traced_test]
    #[ignore = "can only test with openrgb running"]
    async fn test_download_profile() -> OpenRgbResult<()> {
        let client = OpenRgbProtocol::connect_to(DEFAULT_ADDR, DEFAULT_PROTOCOL).await?;
        let profile = client.download_profile("test").await?;
        println!("profile: {0:?}", profile);
        Ok(())
    }

    #[tokio::test]
    // #[traced_test]
    #[ignore = "can only test with openrgb running"]
    async fn test_get_settings() -> OpenRgbResult<()> {
        let client = OpenRgbProtocol::connect_to(DEFAULT_ADDR, DEFAULT_PROTOCOL).await?;
        // let autostart = client.get_autostart_settings().await?;
        // println!("autostart: {0:?}", autostart);
        let mut detector_settings = client.get_detector_settings().await?;
        println!("detector_settings: {detector_settings:?}");
        // let log_manager_settings = client.get_log_manager_settings().await?;
        // println!("log_manager_settings: {log_manager_settings:?}");
        // let profile_manager_settings = client.get_profile_manager_settings().await?;
        // println!("profile_manager_settings: {profile_manager_settings:?}");
        // let server_settings = client.get_server_settings().await?;
        // println!("server_settings: {server_settings:?}");
        // let theme_settings = client.get_theme_settings().await?;
        // println!("theme_settings: {theme_settings:?}");
        // let user_interface_settings = client.get_user_interface_settings().await?;
        // println!("user_interface_settings: {user_interface_settings:?}");

        let mut controllers = Vec::new();
        for i in 0..client.get_controller_count().await? {
            controllers.push(client.get_controller(i).await?);
        }

        for c in &controllers {
            if let Some(v) = detector_settings.detectors.get_mut(c.name()) {
                println!("{} found", c.name());
                *v = true;
            }
        }

        client
            .set_settings(&crate::SettingsKey::Detectors, detector_settings)
            .await?;

        Ok(())
    }

    #[tokio::test]
    #[traced_test]
    #[ignore = "can only test with openrgb running"]
    async fn test_set_settings() -> OpenRgbResult<()> {
        let client = OpenRgbProtocol::connect_to(DEFAULT_ADDR, DEFAULT_PROTOCOL).await?;
        let mut user_interface_settings = client.get_user_interface_settings().await?;
        println!("user_interface_settings: {user_interface_settings:?}");
        user_interface_settings.numerical_labels = !user_interface_settings.numerical_labels;
        client
            .modify_user_interface_settings(&user_interface_settings)
            .await?;
        client.save_settings().await?;
        Ok(())
    }

    #[tokio::test]
    #[traced_test]
    #[ignore = "can only test with openrgb running"]
    async fn test_get_schema() -> OpenRgbResult<()> {
        let client = OpenRgbProtocol::connect_to(DEFAULT_ADDR, DEFAULT_PROTOCOL).await?;
        let schema = client
            .get_schema(&crate::SettingsKey::UserInterface)
            .await?;
        println!("schema: {:?}", schema);
        Ok(())
    }

    #[tokio::test]
    #[traced_test]
    #[ignore = "can only test with openrgb running"]
    async fn test_get_usb_device_info() -> OpenRgbResult<()> {
        let client = OpenRgbProtocol::connect_to(DEFAULT_ADDR, DEFAULT_PROTOCOL).await?;
        let info = client.get_usb_device_info().await?;
        println!("info: {0:?}", info);
        Ok(())
    }

    #[tokio::test]
    #[traced_test]
    #[ignore = "can only test with openrgb running"]
    async fn test_get_hid_device_info() -> OpenRgbResult<()> {
        let client = OpenRgbProtocol::connect_to(DEFAULT_ADDR, DEFAULT_PROTOCOL).await?;
        let info = client.get_hid_device_info().await?;
        println!("info: {0:?}", info);
        Ok(())
    }

    #[tokio::test]
    #[traced_test]
    #[ignore = "can only test with openrgb running"]
    async fn test_get_i2c_bus_info() -> OpenRgbResult<()> {
        let client = OpenRgbProtocol::connect_to(DEFAULT_ADDR, DEFAULT_PROTOCOL).await?;
        let info = client.get_i2c_bus_info().await?;
        println!("info: {0:?}", info);
        Ok(())
    }

    #[tokio::test]
    #[traced_test]
    #[ignore = "can only test with openrgb running"]
    async fn test_get_log_level() -> OpenRgbResult<()> {
        let client = OpenRgbProtocol::connect_to(DEFAULT_ADDR, DEFAULT_PROTOCOL).await?;
        let level = client.get_log_level().await?;
        println!("info: {0:?}", level);
        Ok(())
    }

    #[tokio::test]
    #[traced_test]
    #[ignore = "can only test with openrgb running"]
    async fn test_get_log_buffer() -> OpenRgbResult<()> {
        let client = OpenRgbProtocol::connect_to(DEFAULT_ADDR, DEFAULT_PROTOCOL).await?;
        let data = client.get_log_buffer().await?;
        println!("info: {0:?}", data);
        Ok(())
    }

    #[tokio::test]
    #[traced_test]
    #[ignore = "can only test with openrgb running"]
    async fn test_update_led() -> OpenRgbResult<()> {
        let client = OpenRgbProtocol::connect_to(DEFAULT_ADDR, DEFAULT_PROTOCOL).await?;
        client.update_led(5, 1, &Color::new(255, 0, 0)).await?;
        Ok(())
    }

    #[tokio::test]
    #[traced_test]
    #[ignore = "can only test with openrgb running"]
    async fn test_update_leds() -> OpenRgbResult<()> {
        let client = OpenRgbProtocol::connect_to(DEFAULT_ADDR, DEFAULT_PROTOCOL).await?;
        client.update_leds(5, &[Color::new(255, 0, 0); 20]).await?;
        // client.update_led(4, 0, &Color::new(255, 0, 0)).await?;
        Ok(())
    }

    #[tokio::test]
    #[traced_test]
    #[ignore = "can only test with openrgb running"]
    async fn test_effects_plugin() -> OpenRgbResult<()> {
        let client = OpenRgbProtocol::connect_to(DEFAULT_ADDR, DEFAULT_PROTOCOL).await?;
        let plugins = client.get_plugins().await?;
        println!("plugins: {0:?}", plugins);

        // let effects = client.effect_plugin_get_effects(0).await?;
        // println!("effects: {0:?}", effects);

        // client.effect_plugin_stop_effect(0, effects[0].name()).await?;
        // tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        // client.effect_plugin_start_effect(0, effects[0].name()).await?;

        Ok(())
    }

    #[tokio::test]
    #[traced_test]
    #[ignore = "can only test with openrgb running"]
    async fn test_configure_zone() -> OpenRgbResult<()> {
        let client = OpenRgbProtocol::connect_to(DEFAULT_ADDR, DEFAULT_PROTOCOL).await?;

        for i in 0..client.get_controller_count().await? {
            let controller = client.get_controller(i).await?;
            if let Some(z) = controller
                .zones()
                .iter()
                .inspect(|z| println!("{:?} ({}) ({})", z.flags, z.flags.is_some(), z.name()))
                .find(|z| {
                    z.flags
                        .value()
                        .is_some_and(|f| f.contains(ZoneFlags::ManuallyConfigurableName))
                })
            {
                println!("{}", z.name);
                let mut zone = z.clone();
                zone.display_name = crate::ProtocolOption::Some("zone1".into());
                println!(
                    "Configuring controller {}, zone {}",
                    controller.name(),
                    zone.id
                );
                client.configure_zone(controller.id(), &zone).await?;
                let controller_again = client.get_controller(controller.id()).await?;
                let zone = controller_again
                    .zones()
                    .get(zone.id)
                    .expect("Id must be valid");
                assert_eq!(zone.name(), "zone1");

                // restore zone
                client.configure_zone(controller.id(), z).await?;
            }
        }

        Ok(())
    }
}
