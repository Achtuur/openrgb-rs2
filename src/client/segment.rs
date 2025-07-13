use crate::{data::SegmentData, Color, OpenRgbResult, Command, Zone};

/// A segment in a zone, which can contain multiple LEDs.
pub struct Segment<'a> {
    zone: &'a Zone<'a>,
    segment_data: &'a SegmentData,
}

impl<'a> Segment<'a> {
    pub(crate) fn new(zone: &'a Zone<'a>, segment_data: &'a SegmentData) -> Self {
        Self { zone, segment_data }
    }

    /// Returns the ID of this segment.
    pub fn segment_id(&self) -> usize {
        self.segment_data.id()
    }

    /// Returns the ID of the the controller this segment's zone belongs to.
    pub fn controller_id(&self) -> usize {
        self.zone.controller_id()
    }

    /// Returns the ID of the zone this segment belongs to.
    pub fn zone_id(&self) -> usize {
        self.zone.zone_id()
    }

    /// Returns the name of this segment.
    pub fn name(&self) -> &str {
        self.segment_data.name()
    }

    /// Returns the number of LEDs in this segment.
    ///
    /// `Zone.leds[offset()..offset() + num_leds()]` will return the LEDs in this segment.
    pub fn num_leds(&self) -> usize {
        self.segment_data.led_count() as usize
    }

    /// Returns the index offset of this segment in the zone.
    ///
    /// `Zone.leds[offset()..offset() + num_leds()]` will return the LEDs in this segment.
    pub fn offset(&self) -> usize {
        self.segment_data.offset() as usize
    }

    /// Creates a new [`UpdateLedCommand`] for the controller of this segment's zone.
    #[must_use]
    pub fn cmd(&'a self) -> Command<'a> {
        self.zone.cmd()
    }

    /// Returns a command to update the LEDs for this Zone to `colors`.
    ///
    /// The command must be executed by calling `.execute()`
    pub fn cmd_with_set_leds(&'a self, colors: impl IntoIterator<Item = Color>) -> OpenRgbResult<Command<'a>> {
        let mut cmd = self.cmd();
        cmd.set_segment_leds(self.zone_id(), self.segment_id(), colors)?;
        Ok(cmd)
    }
}
