use crate::impl_enum_discriminant;

/// RGB controller device type.
///
/// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation) for more information.
#[derive(Eq, PartialEq, Debug, Copy, Clone, Hash)]
pub enum DeviceType {
    /// Motherboard.
    Motherboard = 0,
    /// DRAM
    DRam = 1,
    /// GPU
    Gpu = 2,
    /// Cooler
    Cooler = 3,
    /// LED strip
    LEDStrip = 4,
    /// Keyboard
    Keyboard = 5,
    /// Mouse
    Mouse = 6,
    /// Mouse mat
    MouseMat = 7,
    /// Headset
    Headset = 8,
    /// Headset stand
    HeadsetStand = 9,
    /// Gamepad
    Gamepad = 10,
    /// Light
    Light = 11,
    /// Speaker
    Speaker = 12,
    /// Virtual
    Virtual = 13,
    /// Unknown
    Unknown = 14,
}

impl_enum_discriminant!(DeviceType,
    Motherboard: 0,
    DRam: 1,
    Gpu: 2,
    Cooler: 3,
    LEDStrip: 4,
    Keyboard: 5,
    Mouse: 6,
    MouseMat: 7,
    Headset: 8,
    HeadsetStand: 9,
    Gamepad: 10,
    Light: 11,
    Speaker: 12,
    Virtual: 13,
    Unknown: 14
);

#[cfg(test)]
mod tests {
    use std::error::Error;

    use crate::{WriteMessage, data::DeviceType};

    #[tokio::test]
    async fn test_read_001() -> Result<(), Box<dyn Error>> {
        let mut buf = WriteMessage::new(crate::DEFAULT_PROTOCOL);
        let mut msg = buf.push_value(3_u32)?.to_received_msg();

        assert_eq!(msg.read_value::<DeviceType>()?, DeviceType::Cooler);
        Ok(())
    }

    #[tokio::test]
    async fn test_write_001() -> Result<(), Box<dyn Error>> {
        let mut buf = WriteMessage::new(crate::DEFAULT_PROTOCOL);
        let mut msg = buf.push_value(DeviceType::Cooler)?.to_received_msg();
        assert_eq!(msg.read_value::<u32>()?, 3);
        Ok(())
    }
}
