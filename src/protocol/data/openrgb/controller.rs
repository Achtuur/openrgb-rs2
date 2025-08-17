use flagset::{FlagSet, flags};

use crate::OpenRgbResult;
use crate::data::ProtocolOption;
use crate::protocol::data::{Color, DeviceType, Led, ModeData, ZoneData};
use crate::protocol::{DeserFromBuf, ReceivedMessage};

flags! {
    /// RGB Controller flags.
    ///
    /// The definition was taken from OpenRGB/RGBController.h:223-231 (11/07/2025)
    pub enum ControllerFlags: u32 {
        /// Controller is local to this instance.
        IsLocal = 1 << 0,
        /// Controller is on a remote instance,
        IsRemote = 1 << 1,
        /// Controller is a virtual device.
        IsVirtual = 1 << 2,

        /// Flag that is reset before update is called.
        ///
        /// Likely an internal thing
        ResetBeforeUpdate = 1 << 8,
    }
}

/// RGB controller.
///
/// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation#net_packet_id_request_controller_data) for more information.
#[derive(Debug, Eq, PartialEq)]
pub(crate) struct ControllerData {
    /// Controller type.
    device_type: DeviceType,

    /// Controller name.
    name: String,

    /// Controller vendor.
    vendor: String,

    /// Controller description.
    description: String,

    /// Controller version.
    version: String,

    /// Controller serial.
    serial: String,

    /// Controller location.
    location: String,

    /// Controller active mode index.
    active_mode: i32,

    /// Controller modes.
    modes: Vec<ModeData>,

    /// Controller zones.
    zones: Vec<ZoneData>,

    /// Controller LEDs.
    leds: Vec<Led>,

    /// Controller colors.
    colors: Vec<Color>,

    /// Alternate names for LEDs (?)
    ///
    /// Minimum protocol version: 5
    led_alt_names: ProtocolOption<5, Vec<String>>,

    /// flags
    ///
    /// Minimum protocol version: 5
    flags: ProtocolOption<5, FlagSet<ControllerFlags>>,

    /* NOT IN PROTOCOL, BUT USEFUL */
    /// Id of this controller, which is the id used to make the request.
    id: u32,
    /// Number of LEDs in this controller.
    ///
    /// Computed by adding up the zone's lengths.
    num_leds: usize,
}

impl ControllerData {
    pub fn name(&self) -> &str {
        &self.name
    }

    #[allow(unused, reason = "might be useful later")]
    pub fn id(&self) -> u32 {
        self.id
    }

    pub(crate) fn set_id(&mut self, id: u32) {
        self.id = id;
    }

    pub fn device_type(&self) -> DeviceType {
        self.device_type
    }

    pub fn vendor(&self) -> &str {
        &self.vendor
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn serial(&self) -> &str {
        &self.serial
    }

    pub fn location(&self) -> &str {
        &self.location
    }

    /// Returns the mode of this controller that is currently active.
    ///
    /// Currently unsure if this can ever be `None`
    pub fn active_mode(&self) -> Option<&ModeData> {
        self.modes.get(self.active_mode as usize)
    }

    /// Returns the modes this controller has.
    pub fn modes(&self) -> &[ModeData] {
        &self.modes
    }

    /// Returns the zones of this controller.
    pub fn zones(&self) -> &[ZoneData] {
        &self.zones
    }

    /// Returns the LEDs of this controller.
    pub fn leds(&self) -> &[Led] {
        &self.leds
    }

    /// The number of LEDs in all zones of this controller summed together.
    ///
    /// This is not necessarily the same as [`Self::leds()`]'s length.
    pub fn num_leds(&self) -> usize {
        self.num_leds
    }

    /// Returns the colors of this controller.
    pub fn colors(&self) -> &[Color] {
        &self.colors
    }

    /// Returns the alternate names for LEDs, only supported in protocol version 5 and above.
    #[allow(unused, reason = "api not finalised yet")]
    pub fn led_alt_names(&self) -> Option<&[String]> {
        self.led_alt_names.value().map(|v| v.as_slice())
    }

    /// Returns the flags for this controller. Only supported in protocol version 5 and above.
    #[allow(unused, reason = "flags might be used later")]
    pub fn flags(&self) -> Option<FlagSet<ControllerFlags>> {
        self.flags.value().copied()
    }
}

impl DeserFromBuf for ControllerData {
    fn deserialize(buf: &mut ReceivedMessage<'_>) -> OpenRgbResult<Self> {
        let _data_size = buf.read_u32()?;
        let device_type = buf.read_value()?;
        let name = buf.read_value()?;
        let vendor = buf.read_value()?;
        let description = buf.read_value()?;
        let version = buf.read_value()?;
        let serial = buf.read_value()?;
        let location = buf.read_value()?;
        let num_modes = buf.read_value::<u16>()?;
        let active_mode = buf.read_value()?;

        let mut modes = buf.read_n_values::<ModeData>(num_modes as usize)?;
        for (idx, mode) in modes.iter_mut().enumerate() {
            mode.set_id(idx);
        }

        let mut zones = buf.read_value::<Vec<ZoneData>>()?;
        let mut num_leds = 0;
        for (idx, zone) in zones.iter_mut().enumerate() {
            zone.id = idx;
            num_leds += zone.leds_count as usize;
        }

        let leds = buf.read_value()?;
        let colors = buf.read_value()?;
        let led_alt_names = buf.read_value()?;
        let flags = buf.read_value()?;

        Ok(Self {
            device_type,
            name,
            vendor,
            description,
            version,
            serial,
            location,
            active_mode,
            modes,
            zones,
            leds,
            colors,
            led_alt_names,
            flags,
            id: u32::MAX,
            num_leds,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use crate::WriteMessage;
    use crate::data::ProtocolOption;
    use crate::protocol::data::ControllerData;

    #[test]
    fn test_read_001() -> Result<(), Box<dyn Error>> {
        // this message is protocol version 3
        let mut buf = WriteMessage::new(3);
        buf.write_u32(760);
        buf.write_slice(&[
            3, 0, 0, 0, 18, 0, 84, 104, 101, 114, 109, 97, 108, 116, 97, 107, 101, 32, 82, 105,
            105, 110, 103, 0, 12, 0, 84, 104, 101, 114, 109, 97, 108, 116, 97, 107, 101, 0, 25, 0,
            84, 104, 101, 114, 109, 97, 108, 116, 97, 107, 101, 32, 82, 105, 105, 110, 103, 32, 68,
            101, 118, 105, 99, 101, 0, 1, 0, 0, 1, 0, 0, 19, 0, 72, 73, 68, 58, 32, 47, 100, 101,
            118, 47, 104, 105, 100, 114, 97, 119, 49, 48, 0, 8, 0, 0, 0, 0, 0, 7, 0, 68, 105, 114,
            101, 99, 116, 0, 24, 0, 0, 0, 32, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 7, 0,
            83, 116, 97, 116, 105, 99, 0, 25, 0, 0, 0, 64, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0,
            0, 1, 0, 0, 0, 0, 0, 5, 0, 70, 108, 111, 119, 0, 0, 0, 0, 0, 1, 0, 0, 0, 3, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 9, 0, 83, 112, 101, 99, 116, 114, 117, 109, 0, 4, 0, 0, 0, 1,
            0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 7, 0, 82, 105, 112, 112, 108, 101, 0,
            8, 0, 0, 0, 33, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 6, 0, 66, 108, 105, 110,
            107, 0, 12, 0, 0, 0, 33, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 6, 0, 80, 117,
            108, 115, 101, 0, 16, 0, 0, 0, 33, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 5,
            0, 87, 97, 118, 101, 0, 20, 0, 0, 0, 33, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0,
            0, 5, 0, 16, 0, 82, 105, 105, 110, 103, 32, 67, 104, 97, 110, 110, 101, 108, 32, 49, 0,
            1, 0, 0, 0, 0, 0, 0, 0, 20, 0, 0, 0, 0, 0, 0, 0, 0, 0, 16, 0, 82, 105, 105, 110, 103,
            32, 67, 104, 97, 110, 110, 101, 108, 32, 50, 0, 1, 0, 0, 0, 0, 0, 0, 0, 20, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 16, 0, 82, 105, 105, 110, 103, 32, 67, 104, 97, 110, 110, 101, 108, 32,
            51, 0, 1, 0, 0, 0, 0, 0, 0, 0, 20, 0, 0, 0, 0, 0, 0, 0, 0, 0, 16, 0, 82, 105, 105, 110,
            103, 32, 67, 104, 97, 110, 110, 101, 108, 32, 52, 0, 1, 0, 0, 0, 0, 0, 0, 0, 20, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 16, 0, 82, 105, 105, 110, 103, 32, 67, 104, 97, 110, 110, 101,
            108, 32, 53, 0, 1, 0, 0, 0, 0, 0, 0, 0, 20, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        ]);
        let mut msg = buf.to_received_msg();
        let c_data = msg.read_value::<ControllerData>()?;

        assert_eq!(c_data.name, "Thermaltake Riing".to_owned());
        assert_eq!(c_data.vendor, "Thermaltake".to_owned());
        assert_eq!(c_data.description, "Thermaltake Riing Device".to_owned());
        assert_eq!(c_data.version, String::new());
        assert_eq!(c_data.serial, String::new());
        assert_eq!(c_data.location, "HID: /dev/hidraw10".to_owned());
        assert_eq!(c_data.active_mode, 0);
        assert_eq!(c_data.modes.len(), 8);
        assert_eq!(c_data.zones.len(), 5);
        assert_eq!(c_data.leds.len(), 0);
        assert_eq!(c_data.colors.len(), 0);
        assert_eq!(c_data.led_alt_names, ProtocolOption::UnsupportedVersion);
        assert_eq!(c_data.flags, ProtocolOption::UnsupportedVersion);

        Ok(())
    }
}
