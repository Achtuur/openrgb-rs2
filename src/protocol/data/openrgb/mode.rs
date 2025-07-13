use flagset::{FlagSet, flags};

use crate::data::ProtocolOption;
use crate::protocol::{DeserFromBuf, SerToBuf, WriteMessage};
use crate::{OpenRgbResult, protocol::data::Color};
use crate::{ReceivedMessage, impl_enum_discriminant};

flags! {
    /// RGB controller mode flags.
    ///
    /// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation) for more information.
    pub enum ModeFlag: u32 {
        /// Mode has speed parameter.
        HasSpeed = 1 << 0,

        /// Mode has left/right parameter.
        HasDirectionLR = 1 << 1,

        /// Mode has up/down parameter.
        HasDirectionUD = 1 << 2,

        /// Mode has horiz/vert parameter.
        HasDirectionHV = 1 << 3,

        /// Mode has direction parameter.
        HasDirection = (ModeFlag::HasDirectionLR | ModeFlag::HasDirectionUD | ModeFlag::HasDirectionHV).bits(),

        /// Mode has brightness parameter.
        HasBrightness = 1 << 4,

        /// Mode has per-LED colors.
        HasPerLEDColor = 1 << 5,

        /// Mode has mode specific colors.
        HasModeSpecificColor = 1 << 6,

        /// Mode has random color option.
        HasRandomColor = 1 << 7,

        /// Mode can manually be saved.
        ManualSave = 1 << 8,

        /// Mode automatically saves.
        AutomaticSave = 1 << 9,
    }
}

/// Direction for [ModeData].
#[derive(Eq, PartialEq, Debug, Copy, Clone, Default)]
pub enum Direction {
    /// Left direction.
    #[default]
    Left = 0,

    /// Right direction.
    Right = 1,

    /// Up direction.
    Up = 2,

    /// Down direction.
    Down = 3,

    /// Horizontal direction.
    Horizontal = 4,

    /// Vertical direction.
    Vertical = 5,
}

impl_enum_discriminant!(
    Direction,
    Left: 0,
    Right: 1,
    Up: 2,
    Down: 3,
    Horizontal: 4,
    Vertical: 5
);

/// RGB controller color mode.
///
/// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation) for more information.
#[derive(Eq, PartialEq, Debug, Copy, Clone, Default)]
pub enum ColorMode {
    /// No color mode.
    #[default]
    None = 0,

    /// Per LED colors.
    PerLED = 1,

    /// Mode specific colors.
    ModeSpecific = 2,

    /// Random colors.
    Random = 3,
}

impl_enum_discriminant!(ColorMode, None: 0, PerLED: 1, ModeSpecific: 2, Random: 3);

/// RGB controller mode.
///
/// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation#mode-data) for more information.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ModeData {
    /// Mode name.
    name: String,

    /// Device specific mode value
    value: i32,

    /// Mode flags set.
    flags: FlagSet<ModeFlag>,

    /// Mode minimum speed (if mode has [ModeFlag::HasSpeed] flag).
    speed_min: u32,

    /// Mode maximum speed (if mode has [ModeFlag::HasSpeed] flag).
    speed_max: u32,

    /// Mode maximum speed (if mode has [ModeFlag::HasSpeed] flag).
    speed: u32,

    /// Mode minimum brightness (if mode has [ModeFlag::HasBrightness] flag).
    ///
    /// Minimum protocol version: 3
    brightness_min: ProtocolOption<3, u32>,

    /// Mode maximum brightness (if mode has [ModeFlag::HasBrightness] flag).
    ///
    /// Minimum protocol version: 3
    brightness_max: ProtocolOption<3, u32>,

    /// Mode brightness (if mode has [ModeFlag::HasBrightness] flag).
    ///
    /// Minimum protocol version: 3
    brightness: ProtocolOption<3, u32>,

    /// Mode color mode.
    color_mode: ColorMode,

    /// Mode colors.
    colors: Vec<Color>,

    /// Mode minimum colors (if mode has non empty [ModeData::colors] list).
    colors_min: u32,

    /// Mode minimum colors (if mode has non empty [ModeData::colors] list).
    colors_max: u32,

    /// Mode direction.
    direction: Direction,

    /// Index of this mode, not part of received packet but set right after reading
    id: u32,
}

impl ModeData {
    /// Returns the name of this mode.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the ID of this mode.
    pub fn id(&self) -> usize {
        self.id as usize
    }

    pub(crate) fn set_id(&mut self, id: usize) {
        self.id = id as u32;
    }

    /// Returns the flags of this mode.
    pub fn flags(&self) -> FlagSet<ModeFlag> {
        self.flags
    }

    /// Returns the brightness setting of this mode, minimum protocol version: 3.
    ///
    /// If protocol version is less than 3 or the mode flags does not include `HasBrightness`, returns `None`.
    pub fn brightness(&self) -> Option<u32> {
        match self.flags.contains(ModeFlag::HasBrightness) {
            true => self.brightness.value().copied(),
            false => None,
        }
    }

    /// Set the brightness setting of this mode, minimum protocol version: 3.
    pub fn set_brightness(&mut self, b: u32) {
        if self.flags.contains(ModeFlag::HasBrightness) {
            self.brightness.replace(b);
        }
    }

    /// Returns the minimum brightness setting of this mode, minimum protocol version: 3.
    ///
    /// If protocol version is less than 3 or the mode flags does not include `HasBrightness`, returns `None`.
    pub fn brightness_min(&self) -> Option<u32> {
        match self.flags.contains(ModeFlag::HasBrightness) {
            true => self.brightness_min.value().copied(),
            false => None,
        }
    }

    /// Returns the maximum brightness setting of this mode, minimum protocol version: 3.
    ///
    /// If protocol version is less than 3 or the mode flags does not include `HasBrightness`, returns `None`.
    pub fn brightness_max(&self) -> Option<u32> {
        match self.flags.contains(ModeFlag::HasBrightness) {
            true => self.brightness_max.value().copied(),
            false => None,
        }
    }

    /// Returns the speed setting of this mode.
    ///
    /// If `ModeFlag::HasSpeed` is not set, returns `None`.
    pub fn speed(&self) -> Option<u32> {
        self.flags
            .contains(ModeFlag::HasSpeed)
            .then_some(self.speed)
    }

    /// Set the speed setting of this mode.
    pub fn set_speed(&mut self, sp: u32) {
        if self.flags.contains(ModeFlag::HasSpeed) {
            self.speed = sp;
        }
    }

    /// Returns the minimum speed setting of this mode.
    ///
    /// If `ModeFlag::HasSpeed` is not set, returns `None`.
    pub fn speed_min(&self) -> Option<u32> {
        self.flags
            .contains(ModeFlag::HasSpeed)
            .then_some(self.speed_min)
    }

    /// Returns the maximum speed setting of this mode.
    ///
    /// If `ModeFlag::HasSpeed` is not set, returns `None`.
    pub fn speed_max(&self) -> Option<u32> {
        self.flags
            .contains(ModeFlag::HasSpeed)
            .then_some(self.speed_max)
    }

    /// Returns the direction of this mode.
    ///
    /// If `ModeFlag::HasDirection` is not set, returns `None`.
    pub fn direction(&self) -> Option<Direction> {
        self.flags
            .contains(ModeFlag::HasDirection)
            .then_some(self.direction)
    }

    /// Returns the color mode of this mode.
    pub fn color_mode(&self) -> ColorMode {
        self.color_mode
    }

    /// Returns the colors of this mode
    pub fn colors(&self) -> &[Color] {
        &self.colors
    }

    /// Returns the minimum number of colors for this mode.
    ///
    /// Returns `None` if the mode does not have any colors.
    pub fn colors_min(&self) -> Option<u32> {
        (!self.colors.is_empty()).then_some(self.colors_min)
    }

    /// Returns the maximum number of colors for this mode.
    ///
    /// Returns `None` if the mode does not have any colors.
    pub fn colors_max(&self) -> Option<u32> {
        (!self.colors.is_empty()).then_some(self.colors_max)
    }
}

impl DeserFromBuf for ModeData {
    fn deserialize(buf: &mut ReceivedMessage<'_>) -> OpenRgbResult<Self> {
        let name = buf.read_value()?;
        let value = buf.read_value()?;
        let flags = buf.read_value()?;
        let speed_min = buf.read_value()?;
        let speed_max = buf.read_value()?;
        let brightness_min = buf.read_value()?;
        let brightness_max = buf.read_value()?;
        let brightness = buf.read_value()?;
        let colors_min = buf.read_value()?;
        let colors_max = buf.read_value()?;
        let speed = buf.read_value()?;
        let direction = buf.read_value::<Direction>()?;
        let color_mode = buf.read_value()?;
        let colors = buf.read_value::<Vec<Color>>()?;

        Ok(ModeData {
            id: u32::MAX,
            name,
            value,
            flags,
            speed_min,
            speed_max,
            speed,
            brightness_min,
            brightness_max,
            brightness,
            colors_min,
            colors_max,
            direction,
            color_mode,
            colors,
        })
    }
}

impl SerToBuf for ModeData {
    fn serialize(&self, buf: &mut WriteMessage) -> OpenRgbResult<()> {
        buf.push_value(&self.name)?
            .push_value(&self.value)?
            .push_value(&self.flags)?
            .push_value(&self.speed_min)?
            .push_value(&self.speed_max)?
            .push_value(&self.brightness_min)?
            .push_value(&self.brightness_max)?
            .push_value(&self.brightness)?
            .push_value(&self.colors_min)?
            .push_value(&self.colors_max)?
            .push_value(&self.speed)?
            .push_value(&self.direction)?
            .push_value(&self.color_mode)?
            .push_value(&self.colors)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use flagset::FlagSet;

    use crate::{
        Color, ModeData, ProtocolOption, WriteMessage,
        data::{ColorMode, Direction},
        protocol::data::ModeFlag,
    };
    use ModeFlag::*;

    #[tokio::test]
    async fn test_read_flag() -> Result<(), Box<dyn Error>> {
        let mut buf = WriteMessage::new(crate::DEFAULT_PROTOCOL);
        let mut msg = buf.push_value(&154_u32)?.to_received_msg();

        assert_eq!(
            msg.read_value::<FlagSet<ModeFlag>>()?,
            HasDirectionLR | HasDirectionHV | HasBrightness | HasRandomColor
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_write_flag() -> Result<(), Box<dyn Error>> {
        let mut buf = WriteMessage::new(crate::DEFAULT_PROTOCOL);
        let mut msg = buf
            .push_value(&(ModeFlag::HasBrightness | ModeFlag::HasSpeed))?
            .to_received_msg();

        assert_eq!(msg.read_value::<u32>()?, (1 << 4) | (1 << 0));

        Ok(())
    }

    #[tokio::test]
    async fn test_read_dir() -> Result<(), Box<dyn Error>> {
        let mut buf = WriteMessage::new(crate::DEFAULT_PROTOCOL);
        let mut msg = buf.push_value(&3_u32)?.to_received_msg();

        assert_eq!(msg.read_value::<Direction>()?, Direction::Down);
        Ok(())
    }

    #[tokio::test]
    async fn test_write_dir() -> Result<(), Box<dyn Error>> {
        let mut buf = WriteMessage::new(crate::DEFAULT_PROTOCOL);
        let mut msg = buf.push_value(&Direction::Down)?.to_received_msg();
        assert_eq!(msg.read_value::<u32>()?, 3);
        Ok(())
    }

    #[tokio::test]
    async fn test_read_color_mode() -> Result<(), Box<dyn Error>> {
        let mut buf = WriteMessage::new(crate::DEFAULT_PROTOCOL);
        let mut msg = buf.push_value(&3_u32)?.to_received_msg();

        assert_eq!(msg.read_value::<ColorMode>()?, ColorMode::Random);
        Ok(())
    }

    #[tokio::test]
    async fn test_write_color_mode() -> Result<(), Box<dyn Error>> {
        let mut buf = WriteMessage::new(crate::DEFAULT_PROTOCOL);
        let mut msg = buf.push_value(&ColorMode::Random)?.to_received_msg();
        assert_eq!(msg.read_value::<u32>()?, 3);
        Ok(())
    }

    #[test]
    fn test_read_001() -> Result<(), Box<dyn Error>> {
        let mut buf = WriteMessage::new(3);
        let mut msg = buf
            .push_value(&"test")? // name
            .push_value(&46_i32)? // value
            .push_value(&31_u32)? // flags
            .push_value(&10_u32)? // speed_min
            .push_value(&1000_u32)? // speed_max
            .push_value(&1_u32)? // brightness_min
            .push_value(&1024_u32)? // brightness_max
            .push_value(&512_u32)? // brightness
            .push_value(&0_u32)? // colors_min
            .push_value(&256_u32)? // colors_max
            .push_value(&51_u32)? // speed
            .push_value(&4_u32)? // direction
            .push_value(&1_u32)? // color_mode
            .push_value(&[
                Color {
                    r: 37,
                    g: 54,
                    b: 126,
                },
                Color {
                    r: 37,
                    g: 54,
                    b: 255,
                },
            ])?
            .to_received_msg();

        let mode_data = msg.read_value::<ModeData>()?;

        assert_eq!(mode_data.name(), "test");
        assert_eq!(mode_data.speed_min(), Some(10));
        assert_eq!(mode_data.speed_max(), Some(1000));
        assert_eq!(mode_data.brightness_min(), Some(1));
        assert_eq!(mode_data.brightness_max(), Some(1024));
        assert_eq!(mode_data.colors_min(), Some(0));
        assert_eq!(mode_data.colors_max(), Some(256));
        assert_eq!(mode_data.speed(), Some(51));
        assert_eq!(mode_data.brightness(), Some(512));
        assert_eq!(mode_data.direction(), Some(Direction::Horizontal));
        assert_eq!(mode_data.color_mode(), ColorMode::PerLED);
        assert_eq!(
            mode_data.colors(),
            &[
                Color {
                    r: 37,
                    g: 54,
                    b: 126
                },
                Color {
                    r: 37,
                    g: 54,
                    b: 255
                }
            ]
        );

        Ok(())
    }

    #[test]
    fn test_write_001() -> Result<(), Box<dyn Error>> {
        let mode = ModeData {
            id: u32::MAX,
            name: "test".to_string(),
            value: 46,
            flags: HasDirection | HasSpeed | HasBrightness,
            speed_min: 10,
            speed_max: 1000,
            brightness_min: ProtocolOption::Some(1),
            brightness_max: ProtocolOption::Some(231),
            colors_min: 0,
            colors_max: 256,
            speed: 51,
            brightness: ProtocolOption::Some(50),
            direction: Direction::Horizontal,
            color_mode: ColorMode::PerLED,
            colors: vec![
                Color {
                    r: 37,
                    g: 54,
                    b: 126,
                },
                Color {
                    r: 37,
                    g: 54,
                    b: 255,
                },
            ],
        };

        let mut buf = WriteMessage::new(crate::DEFAULT_PROTOCOL);
        buf.write_value(&mode)?;
        let mut msg = buf.to_received_msg();
        assert_eq!(mode, msg.read_value::<ModeData>()?);
        Ok(())
    }
}
