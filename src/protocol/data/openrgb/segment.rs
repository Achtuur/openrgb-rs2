use crate::{
    OpenRgbError, OpenRgbResult, ZoneType,
    protocol::{DeserFromBuf, ReceivedMessage, SerToBuf, WriteMessage},
};

/// Data for `OpenRGB` segments
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SegmentData {
    name: String,
    /// Segment type should be the same as its parent's zone type.
    ///
    /// For now it is always `ZoneType::Linear`.
    seg_type: ZoneType,
    start_idx: u32,
    led_count: u32,

    // Not part of protocol, but set immediately after reading
    id: usize,
}

impl SegmentData {
    pub(crate) fn new(name: impl Into<String>, start_idx: u32, led_count: u32) -> Self {
        Self {
            name: name.into(),
            seg_type: ZoneType::Linear,
            start_idx,
            led_count,
            id: usize::MAX,
        }
    }

    /// Returns the name of this segment.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the number of LEDs in this segment.
    pub fn led_count(&self) -> u32 {
        self.led_count
    }

    /// Returns the offset of this segment in the zone. This is its starting index.
    pub fn offset(&self) -> u32 {
        self.start_idx
    }

    /// Returns the id of this segment.
    pub fn id(&self) -> usize {
        self.id
    }

    pub(crate) fn set_id(&mut self, id: usize) {
        self.id = id;
    }
}

impl DeserFromBuf for SegmentData {
    fn deserialize(buf: &mut ReceivedMessage<'_>) -> OpenRgbResult<Self> {
        if buf.protocol_version() < 4 {
            return Err(OpenRgbError::ProtocolError(
                "SegmentData is not supported in protocol version < 4".to_owned(),
            ));
        }

        let name = buf.read_value()?;
        let seg_type = buf.read_value()?;
        let start_idx = buf.read_value()?;
        let led_count = buf.read_value()?;

        Ok(Self {
            name,
            seg_type,
            start_idx,
            led_count,
            id: usize::MAX,
        })
    }
}

impl SerToBuf for SegmentData {
    fn serialize(&self, buf: &mut WriteMessage) -> OpenRgbResult<()> {
        if buf.protocol_version() < 4 {
            return Err(OpenRgbError::ProtocolError(
                "SegmentData is not supported in protocol version < 4".to_owned(),
            ));
        }
        buf.write_value(&self.name)?;
        buf.write_value(&self.seg_type)?;
        buf.write_value(&self.start_idx)?;
        buf.write_value(&self.led_count)?;
        Ok(())
    }
}
