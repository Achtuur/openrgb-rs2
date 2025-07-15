use std::fmt::Debug;
use std::net::Ipv4Addr;
use std::sync::Arc;

use tokio::net::ToSocketAddrs;
use tokio::sync::Mutex;

use super::data::{Color, ControllerData, ModeData, RawString, SegmentData};
use crate::{EffectsPluginPacket, OpenRgbError, OpenRgbResult, PluginData, PluginEffect};

/// Default protocol version used by the [`crate::OpenRgbClient::connect`].
pub const DEFAULT_PROTOCOL: u32 = 5;

/// Default address used by [`crate::OpenRgbClient::connect`].
pub const DEFAULT_ADDR: (Ipv4Addr, u16) = (Ipv4Addr::LOCALHOST, 6742);

/// Device ID to use when no specific device is targeted.
const NO_DEVICE_ID: u32 = 0;

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

        Ok(Self {
            protocol_id: protocol,
            stream: Arc::new(Mutex::new(stream)),
        })
    }

    /// Get protocol version negotiated with server.
    ///
    /// This is the lowest between this client maximum supported version ([`DEFAULT_PROTOCOL`]) and server version.
    ///
    /// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation#protocol-versions) for more information.
    pub fn get_protocol_version(&self) -> u32 {
        self.protocol_id
    }

    /// Helper method to write a packet to the server.
    async fn write_packet<T: SerToBuf>(
        &self,
        device_id: u32,
        packet_id: PacketId,
        data: &T,
    ) -> OpenRgbResult<()> {
        self.stream
            .lock()
            .await
            .write_packet(device_id, packet_id, data)
            .await
    }

    /// Helper method to write a packet to the server and parse the response.
    async fn request<I: SerToBuf, O: DeserFromBuf>(
        &self,
        device_id: u32,
        packet_id: PacketId,
        data: &I,
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
        self.write_packet(
            NO_DEVICE_ID,
            PacketId::SetClientName,
            &RawString(&name.into()),
        )
        .await
    }

    /// Get number of controllers.
    ///
    /// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation#net_packet_id_request_controller_count) for more information.
    pub async fn get_controller_count(&self) -> OpenRgbResult<u32> {
        self.request(NO_DEVICE_ID, PacketId::RequestControllerCount, &())
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
        self.write_packet(
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
        self.write_packet(
            controller_id,
            PacketId::RGBControllerUpdateSingleLed,
            &(led_id, color),
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
        let packet = OpenRgbPacket::new(colors);
        self.write_packet(controller_id, PacketId::RGBControllerUpdateLeds, &packet)
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
        let packet = OpenRgbPacket::new((zone_id, colors));
        self.write_packet(
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
        let packet = OpenRgbPacket::new((mode.id() as u32, mode));
        self.write_packet(controller_id, PacketId::RGBControllerUpdateMode, &packet)
            .await
    }

    /// Set custom mode.
    ///
    /// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation#net_packet_id_rgbcontroller_setcustommode) for more information.
    #[allow(unused)] // unused on purpose
    #[allow(clippy::pedantic)]
    pub async fn set_custom_mode(&self, controller_id: u32) -> OpenRgbResult<()> {
        unimplemented!(
            "Not implemented as per recommendation from OpenRGB devs (https://discord.com/channels/699861463375937578/709998213310054490/1372954035581096158)"
        );
        // self
        //     .write_packet(controller_id, PacketId::RGBControllerSetCustomMode, &())
        //     .await
    }

    /// Get profiles.
    ///
    /// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation#net_packet_id_request_profile_list) for more information.
    pub async fn get_profiles(&self) -> OpenRgbResult<Vec<String>> {
        self.check_protocol_version(2, "Get profiles")?;
        self.request::<_, (u32, Vec<String>)>(0, PacketId::RequestProfileList, &())
            .await
            .map(|(_size, profiles)| profiles)
    }

    /// Load a profile.
    ///
    /// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation#net_packet_id_request_load_profile) for more information.
    pub async fn load_profile(&self, name: impl Into<String>) -> OpenRgbResult<()> {
        self.check_protocol_version(2, "Load profiles")?;
        self.write_packet(0, PacketId::RequestLoadProfile, &RawString(&name.into()))
            .await
    }

    /// Save a profile.
    ///
    /// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation#net_packet_id_request_save_profile) for more information.
    pub async fn save_profile(&self, name: impl Into<String>) -> OpenRgbResult<()> {
        self.check_protocol_version(2, "Save profiles")?;
        self.write_packet(0, PacketId::RequestSaveProfile, &name.into())
            .await
    }

    /// Delete a profile.
    ///
    /// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation#net_packet_id_request_delete_profile) for more information.
    pub async fn delete_profile(&self, name: impl Into<String>) -> OpenRgbResult<()> {
        self.check_protocol_version(2, "Delete profiles")?;
        self.write_packet(0, PacketId::RequestDeleteProfile, &name.into())
            .await
    }

    /// Save a mode.
    ///
    /// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation#net_packet_id_rgbcontroller_savemode) for more information.
    pub async fn save_mode(&self, controller_id: u32, mode: &ModeData) -> OpenRgbResult<()> {
        self.check_protocol_version(3, "Save mode")?;
        let packet = OpenRgbPacket::new((mode.id() as u32, mode));
        self.write_packet(controller_id, PacketId::RGBControllerSaveMode, &packet)
            .await
    }

    /// Returns a a list of names of installed plugins.
    pub async fn get_plugins(&self) -> OpenRgbResult<Vec<PluginData>> {
        self.check_protocol_version(4, "Request Plugin List")?;
        // response contains length of data in the packet
        let resp: (u32, Vec<_>) = self
            .request(NO_DEVICE_ID, PacketId::RequestPluginList, &())
            .await?;
        Ok(resp.1)
    }

    /// Performs a plugin specific command. Depends on the plugin what this does.
    ///
    /// In this case, the `pkt_dev_idx` (`controller_id`) is used as the Plugin ID.
    pub async fn plugin_specific_receive<I, O>(&self, plugin_id: u32, header: u32, data: &I) -> OpenRgbResult<O>
    where
        I: SerToBuf,
        O: DeserFromBuf,
    {
        self.check_protocol_version(4, "Plugin Specific Command")?;
        let (recv_header, resp): (u32, O) = self.request(plugin_id, PacketId::PluginSpecific, &(header, data)).await?;
        if header != recv_header {
            return Err(OpenRgbError::ProtocolError(format!(
                "Plugin Specific Command header mismatch: expected {header}, got {recv_header}"
            )));
        }
        Ok(resp)
    }

    pub async fn plugin_specific_write_packet<I>(&self, plugin_id: u32, header: u32, data: &I) -> OpenRgbResult<()> 
    where I: SerToBuf
    {
        self.check_protocol_version(4, "Plugin Specific Command")?;
        self.write_packet(plugin_id, PacketId::PluginSpecific, &(header, data)).await
    }

    pub async fn add_segment(
        &self,
        controller_id: u32,
        zone_id: u32,
        segment: &SegmentData,
    ) -> OpenRgbResult<()> {
        // segments are version 4, segments commands are version 5
        self.check_protocol_version(5, "Add Segment")?;
        let packet = OpenRgbPacket::new((zone_id, segment));
        self.write_packet(controller_id, PacketId::RGBControllerAddSegment, &packet)
            .await
    }

    pub async fn clear_segments(&self, controller_id: u32) -> OpenRgbResult<()> {
        self.check_protocol_version(5, "Clear segment")?;
        self.write_packet(controller_id, PacketId::RgbControllerClearSegments, &())
            .await
    }

    /// Request a device rescan.
    pub async fn rescan_devices(&self) -> OpenRgbResult<()> {
        self.check_protocol_version(5, "Rescan devices")?;
        self.write_packet(NO_DEVICE_ID, PacketId::RequestDeviceRescan, &())
            .await
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

    pub async fn effect_plugin_get_effects(&self, effects_plugin_id: u32) -> OpenRgbResult<Vec<PluginEffect>> {
        self.plugin_specific_receive(
            effects_plugin_id,
            EffectsPluginPacket::RequestEffectList.into(),
            &(),
        ).await
    }

    pub async fn effect_plugin_start_effect(&self, effect_plugin_id: u32, effect_name: &str) -> OpenRgbResult<()> {
        self.plugin_specific_write_packet(
            effect_plugin_id,
            EffectsPluginPacket::StartEffect.into(),
            &effect_name
        ).await
    }

    pub async fn effect_plugin_stop_effect(&self, effect_plugin_id: u32, effect_name: &str) -> OpenRgbResult<()> {
        self.plugin_specific_write_packet(
            effect_plugin_id,
            EffectsPluginPacket::StopEffect.into(),
            &effect_name
        ).await
    }
}

#[cfg(test)]
mod tests {

    use crate::SegmentData;
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
    async fn test_get_controller_count() -> OpenRgbResult<()> {
        let client = OpenRgbProtocol::connect_to(DEFAULT_ADDR, DEFAULT_PROTOCOL).await?;
        let count = client.get_controller_count().await?;
        assert!(count > 0);
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
            assert_eq!(controller.id(), 0);
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
        let _ = client.get_profiles().await?;
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
    #[traced_test]
    #[ignore = "can only test with openrgb running"]
    async fn test_rescan_devices() -> OpenRgbResult<()> {
        let client = OpenRgbProtocol::connect_to(DEFAULT_ADDR, DEFAULT_PROTOCOL).await?;
        let _ = client.rescan_devices().await;
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
        client.update_leds(1, &[Color::new(255, 0, 0); 20]).await?;
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

        let effects = client.effect_plugin_get_effects(0).await?;
        println!("effects: {0:?}", effects);

        client.effect_plugin_stop_effect(0, effects[0].name()).await?;
        // tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        // client.effect_plugin_start_effect(0, effects[0].name()).await?;

        Ok(())
    }
}
