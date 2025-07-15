use crate::{data::SegmentData, Color, Command, OpenRgbError, OpenRgbResult, Zone};

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

    /// Sets a single LED in this segment to the given `color`.
    ///
    /// # Errors
    ///
    /// Returns an error if the index is out of bounds for this zone.
    pub async fn set_led<C: Into<Color>>(&self, idx: usize, color: C) -> OpenRgbResult<()> {
        if idx >= self.num_leds() {
            return Err(OpenRgbError::CommandError(format!(
                "Index {idx} out of bounds for zone {} with {} LEDs",
                self.name(),
                self.num_leds()
            )));
        }
        let idx = self.offset() + idx;
        self.zone.set_led(idx, color).await
    }

    /// Sets all LEDs in this segment to the given `color`.
    ///
    /// # Limitation
    ///
    /// This will set every other LED in the zone to black, as those colors are not specified.
    /// To get around this, use `[Self::cmd()]` instead and specify the zone color.
    pub async fn set_all_leds<C: Into<Color>>(&self, color: C) -> OpenRgbResult<()> {
        let color = color.into();
        let colors = (0..self.offset()).map(|_| Color::default())
            .chain((0..self.num_leds()).map(|_| color));

        self.zone.set_leds(colors).await
    }

    /// Creates a new [`Command`] for the controller of this segment's zone.
    #[must_use]
    pub fn cmd(&'a self) -> Command<'a> {
        self.zone.cmd()
    }

    /// Returns a command to update the LEDs for this Zone to `colors`.
    ///
    /// The command must be executed by calling `.execute()`
    pub fn cmd_with_set_leds<C: Into<Color>>(
        &'a self,
        colors: impl IntoIterator<Item = C>,
    ) -> OpenRgbResult<Command<'a>> {
        let mut cmd = self.cmd();
        cmd.set_segment_leds(self.zone_id(), self.segment_id(), colors)?;
        Ok(cmd)
    }
}
