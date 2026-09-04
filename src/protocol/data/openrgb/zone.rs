use flagset::{FlagSet, flags};

use crate::protocol::{DeserFromBuf, ReceivedMessage};
use crate::{
    ModeData, SerToBuf, data::openrgb::matrix::MatrixMapData, protocol::data::ProtocolOption,
};
use crate::{OpenRgbResult, impl_enum_discriminant};

use super::SegmentData;

/// Type of zones available.
///
/// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation#zone-data) for more information.
#[derive(Eq, PartialEq, Debug, Copy, Clone)]
pub enum ZoneType {
    /// Single zone.
    Single = 0,

    /// Linear zone.
    Linear = 1,

    /// Matrix zone.
    Matrix = 2,

    /// Linear loop zone
    LinearLoop = 3,

    /// Matrix Loop in X direction
    MatrixLoopX = 4,

    /// Matrix Loop in Y direction
    MatrixLoopY = 5,

    /// Segmented zone
    Segmented = 6,
}

impl_enum_discriminant!(ZoneType,
    Single: 0,
    Linear: 1,
    Matrix: 2,
    LinearLoop: 3,
    MatrixLoopX: 4,
    MatrixLoopY: 5,
    Segmented: 6
);

flags! {
    /// Flags for RGB controller zones
    ///
    /// Taken from OpenRGB/RGBController.h:122-126 (11/07/2025)
    pub enum ZoneFlags: u32 {
        /// Zone is resizable, but only for effects. Treat as single LED
        ResizableForEffectsOnly = 1 << 0,
        ///  Zone size is manually configurable
        ManuallyConfigurableSize = 1 << 1,
        ///  Zone name is manually configurable
        ManuallyConfigurableName = 1 << 2,
        ///  Zone type is manually configurable
        ManuallyConfigurableType = 1 << 3,
        ///  Zone matrix map is manually configurable
        ManuallyConfigurableMatrixMap = 1 << 4,
        ///  Zone segments are manually configurable
        ManuallyConfigurableSegments = 1 << 5,
        ///  Zone dev-specific cfg manually configurable
        ManuallyConfigurableDeviceSpecific = 1 << 6,
        ///  Zone size has been manually configured
        ManuallyConfiguredSize = 1 << 12,
        ///  Zone name has been manually configured
        ManuallyConfiguredName = 1 << 13,
        ///  Zone type has been manually configured
        ManuallyConfiguredType = 1 << 14,
        ///  Zone matrix map has been manually
        ManuallyConfiguredMatrixMap = 1 << 15,
        ///  Zone segments have been manually configured
        ManuallyConfiguredSegments = 1 << 16,
        ///  Zone device-specific cfg manually
        ManuallyConfiguredDeviceSpecific = 1 << 17,
        ///  Zone geometry may
        ZoneGeometryMayChange = 1 << 24,

    }
}

/// RGB controller zone.
///
/// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation#zone-data) for more information.
#[derive(Debug, Clone, Eq, PartialEq)]
pub(crate) struct ZoneData {
    /// Id of this zone.
    ///
    /// Not part of the packet, but set right after reading
    /// since the sender knows the zone id.
    pub id: usize,
    /// Zone name.
    pub name: String,

    /// Zone type.
    pub zone_type: ZoneType,

    /// Zone minimum LED number.
    ///
    /// Minimum number of LEDs if this zone is resizable.
    pub leds_min: u32,

    /// Zone maximum LED number.
    ///
    /// Maximum number of LEDs if this zone is resizable.
    pub leds_max: u32,

    /// Zone LED count.
    pub leds_count: u32,

    /// Segments in this zone
    ///
    /// Minimum version: 4
    pub segments: ProtocolOption<4, Vec<SegmentData>>,

    /// Flags for this zone.
    ///
    /// Minimum version: 5
    pub flags: ProtocolOption<5, FlagSet<ZoneFlags>>,

    /// Zone LED matrix (if [`ZoneData::zone_type`] is [`ZoneType::Matrix`]).
    ///
    /// Matrix is the "position" of the LEDs in the zone relative to the top left corner.
    ///
    /// The value represents the LED id of the LED at that position.
    /// A value of `u32::MAX` means that there is no led present.
    pub matrix: Option<MatrixMapData>,
    /// Currently active mode, points to `zone_modes`.
    pub active_mode: ProtocolOption<6, i32>,
    pub zone_modes: ProtocolOption<6, Vec<ModeData>>,
    pub display_name: ProtocolOption<6, String>,
}

impl ZoneData {
    /// Id of this zone.
    pub fn id(&self) -> usize {
        self.id
    }

    /// Returns the name of this zone.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// [`ZoneType`] of this zone.
    pub fn zone_type(&self) -> ZoneType {
        self.zone_type
    }

    /// Minimum number of LEDs for this zone if it is resizable.
    pub fn leds_min(&self) -> usize {
        self.leds_min as usize
    }

    /// Maximum number of LEDs for this zone if it is resizable.
    pub fn leds_max(&self) -> usize {
        self.leds_max as usize
    }

    /// Number of LEDs in this zone.
    pub fn leds_count(&self) -> usize {
        self.leds_count as usize
    }

    /// Returns the segments in this zone.
    ///
    /// If the protocol version is lower than 4, this will return `None`, otherwise, this is always `Some`
    pub fn segments(&self) -> Option<&[SegmentData]> {
        self.segments.value().map(|s| s.as_slice())
    }

    /// LED matrix of this zone.
    ///
    /// If [`Self::zone_type()`] is [`ZoneType::Matrix`], this will return `Some`.
    pub fn matrix(&self) -> Option<&MatrixMapData> {
        self.matrix.as_ref()
    }
}

impl DeserFromBuf for ZoneData {
    fn deserialize(buf: &mut ReceivedMessage<'_>) -> OpenRgbResult<Self> {
        let name = buf.read_value()?;
        let zone_type = buf.read_value()?;
        let leds_min = buf.read_value()?;
        let leds_max = buf.read_value()?;
        let leds_count = buf.read_value()?;
        let matrix = buf.read_value()?;

        let mut segments: ProtocolOption<4, Vec<SegmentData>> = buf.read_value()?;
        if let Some(seg) = segments.value_mut() {
            for (i, s) in seg.iter_mut().enumerate() {
                s.set_id(i);
            }
        }

        let flags = buf.read_value()?;

        // theory
        let active_mode = buf.read_value()?;
        let zone_modes = buf.read_value()?;
        let display_name = buf.read_value()?;

        Ok(Self {
            id: usize::MAX,
            name,
            zone_type,
            leds_min,
            leds_max,
            leds_count,
            matrix,
            segments,
            flags,
            active_mode,
            zone_modes,
            display_name,
        })
    }
}

impl SerToBuf for ZoneData {
    fn serialize(&self, buf: &mut crate::WriteMessage) -> OpenRgbResult<()> {
        buf.push_value(&self.name)?
            .push_value(self.zone_type)?
            .push_value(self.leds_min)?
            .push_value(self.leds_max)?
            .push_value(self.leds_count)?
            .push_value(&self.matrix)?
            .push_value(&self.segments)?
            .push_value(&self.flags)?
            .push_value(&self.active_mode)?
            .push_value(&self.zone_modes)?
            .push_value(&self.display_name)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use crate::{WriteMessage, data::ZoneType};

    #[tokio::test]
    async fn test_read_001() -> Result<(), Box<dyn Error>> {
        let mut buf = WriteMessage::new(crate::DEFAULT_PROTOCOL);
        let mut msg = buf.push_value(1_u32)?.to_received_msg();

        assert_eq!(msg.read_value::<ZoneType>()?, ZoneType::Linear);
        Ok(())
    }

    #[tokio::test]
    async fn test_write_001() -> Result<(), Box<dyn Error>> {
        let mut buf = WriteMessage::new(crate::DEFAULT_PROTOCOL);
        let mut msg = buf.push_value(ZoneType::Linear)?.to_received_msg();
        assert_eq!(msg.read_value::<u32>()?, 1);
        Ok(())
    }
}

// #[cfg(test)]
// mod tests {
//     use std::error::Error;

//     use array2d::Array2D;
//     use tokio_test::io::Builder;

//     use crate::data::ProtocolOption;
//     use crate::protocol::data::{ZoneData, ZoneType};
//     use crate::protocol::tests::setup;

//     #[tokio::test]
//     async fn test_read_001() -> Result<(), Box<dyn Error>> {
//         setup()?;

//         let mut stream = Builder::new()
//             .read(&5_u16.to_le_bytes()) // name len
//             .read(b"test\0") // name
//             .read(&1_u32.to_le_bytes()) // type
//             .read(&3_u32.to_le_bytes()) // leds_min
//             .read(&18_u32.to_le_bytes()) // leds_max
//             .read(&15_u32.to_le_bytes()) // leds_count
//             .read(&0_u16.to_le_bytes()) // matrix_len
//             .build();

//         assert_eq!(
//             stream.read_value::<ZoneData>().await?,
//             ZoneData {
//                 name: "test".to_string(),
//                 zone_type: ZoneType::Linear,
//                 leds_min: 3,
//                 leds_max: 18,
//                 leds_count: 15,
//                 matrix: None,
//                 segments: ProtocolOption::Some(vec![]),
//                 flags: ProtocolOption::Some(0),
//                 id: u32::MAX,
//             }
//         );

//         Ok(())
//     }

//     #[tokio::test]
//     async fn test_read_002() -> Result<(), Box<dyn Error>> {
//         setup()?;

//         let mut stream = Builder::new()
//             .read(&5_u16.to_le_bytes()) // name len
//             .read(b"test\0") // name
//             .read(&1_u32.to_le_bytes()) // type
//             .read(&3_u32.to_le_bytes()) // leds_min
//             .read(&18_u32.to_le_bytes()) // leds_max
//             .read(&15_u32.to_le_bytes()) // leds_count
//             .read(&32_u16.to_le_bytes()) // matrix_len
//             .read(&2_u32.to_le_bytes()) // matrix_height
//             .read(&3_u32.to_le_bytes()) // matrix_width
//             .read(&0_u32.to_le_bytes()) // matrix[0]
//             .read(&1_u32.to_le_bytes()) // matrix[1]
//             .read(&2_u32.to_le_bytes()) // matrix[2]
//             .read(&3_u32.to_le_bytes()) // matrix[3]
//             .read(&4_u32.to_le_bytes()) // matrix[4]
//             .read(&5_u32.to_le_bytes()) // matrix[5]
//             .build();

//         assert_eq!(
//             stream.read_value::<ZoneData>().await?,
//             ZoneData {
//                 name: "test".to_string(),
//                 zone_type: ZoneType::Linear,
//                 leds_min: 3,
//                 leds_max: 18,
//                 leds_count: 15,
//                 matrix: Some(Array2D::from_rows(&[vec![0, 1, 2], vec![3, 4, 5]]).unwrap()),
//                 segments: ProtocolOption::Some(vec![]),
//                 flags: ProtocolOption::Some(0),
//                 id: u32::MAX,
//             }
//         );

//         Ok(())
//     }
// }
