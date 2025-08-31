use array2d::Array2D;

use crate::{
    Color, Command, Controller, Led, OpenRgbError, OpenRgbResult, ZoneType,
    client::segment::Segment,
    data::{SegmentData, ZoneData},
};

/// A zone in a controller, which contains one or more LEDs.
///
/// Zones can also contain segments, which are user-created subdivisions of the zone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Zone<'c> {
    controller: &'c Controller,
    zone_data: &'c ZoneData,
}

impl<'c> Zone<'c> {
    pub(crate) fn new(controller: &'c Controller, zone_data: &'c ZoneData) -> Self {
        Self {
            controller,
            zone_data,
        }
    }

    /// Returns the ID of the controller this zone belongs to.
    pub fn controller_id(&self) -> usize {
        self.controller.id()
    }

    /// Returns the ID of this zone.
    pub fn zone_id(&self) -> usize {
        self.zone_data.id()
    }

    delegate::delegate! {
        to self.zone_data {
            /// Returns the ID of this zone.
            pub fn id(&self) -> usize;

            /// Returns the name of this zone.
            pub fn name(&self) -> &str;

            /// Returns the type of this zone.
            pub fn zone_type(&self) -> ZoneType;

            /// Returns the minimum number of LEDs for this zone if it is resizable.
            pub fn leds_min(&self) -> usize;

            /// Returns the maximum number of LEDs for this zone if it is resizable.
            pub fn leds_max(&self) -> usize;

            /// Returns the number of LEDs in this zone.
            #[call(leds_count)]
            pub fn num_leds(&self) -> usize;

            #[call(segments)]
            pub(crate) fn segment_data(&self) -> Option<&[SegmentData]>;
            #[expect(unused, reason = "Api not finalised yet")]
            #[call(matrix)]
            pub(crate) fn matrix_data(&self) -> Option<&Array2D<u32>>;
        }
    }

    /// Returns the segment with the given `segment_id`.
    pub fn get_segment(&'c self, segment_id: usize) -> OpenRgbResult<Segment<'c>> {
        let Some(segments) = self.segment_data() else {
            return Err(OpenRgbError::CommandError(
                "Segments not supported in protocol version < 4".to_owned(),
            ));
        };
        let data = segments.get(segment_id).ok_or_else(|| {
            OpenRgbError::CommandError(format!(
                "Segment with id {segment_id} not found in zone {}",
                self.name()
            ))
        })?;
        Ok(Segment::new(self, data))
    }

    /// Returns the offset of this zone in the controller's LED array.
    pub fn offset(&self) -> usize {
        self.controller
            .get_zone_led_offset(self.zone_id())
            .expect("Zone id should be valid")
    }

    /// Returns an iterator over the Leds of this zone
    pub fn led_iter(&self) -> impl Iterator<Item = Led<'c>> {
        self.controller
            .led_iter()
            .skip(self.offset())
            .take(self.num_leds())
    }

    /// Creates a new [`Command`] for the controller of this zone.
    ///
    /// The command must be executed by calling `.execute()`
    #[must_use]
    pub fn cmd(&self) -> Command<'c> {
        Command::new(self.controller)
    }

    /// Creates a new [`Command`] for the controller of this zone
    /// and sets the LED colors using the provided closure.
    #[must_use]
    pub fn cmd_with_leds<F>(&self, led_clr: F) -> Command<'c>
    where
        F: Fn(Led<'c>) -> Color,
    {
        let mut cmd = self.cmd();
        for led in self.led_iter() {
            cmd.set_led(led.id(), led_clr(led))
                .expect("Led index incorrect");
        }
        cmd
    }

    /// Returns a command to update the LEDs for this Zone to `colors`.
    /// Equivalent to `cmd().set_zone_leds(self.zone_id(), colors)`.
    ///
    /// The command must be executed by calling `.execute()`
    pub fn cmd_with_set_leds<C: Into<Color>>(
        &'c self,
        colors: impl IntoIterator<Item = C>,
    ) -> OpenRgbResult<Command<'c>> {
        let mut cmd = self.cmd();
        cmd.set_zone_leds(self.zone_id(), colors)?;
        Ok(cmd)
    }

    /// Sets a single LED in this zone to the given `color`.
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
        self.controller.set_led(idx, color).await
    }

    /// Sets all LEDs in this zone to the given `color`.
    pub async fn set_all_leds<C: Into<Color>>(&self, color: C) -> OpenRgbResult<()> {
        let color = color.into();
        let colors = (0..self.num_leds()).map(|_| color);
        self.set_leds(colors).await
    }

    /// Sets the LEDs in this zone to the given colors.
    pub async fn set_leds<C: Into<Color>>(
        &self,
        colors: impl IntoIterator<Item = C>,
    ) -> OpenRgbResult<()> {
        let mut color_v = colors.into_iter().map(Into::into).collect::<Vec<_>>();
        if color_v.len() >= self.num_leds() {
            tracing::warn!(
                "Zone {} for controller {} was given {} colors, while its length is {}. This might become a hard error in the future.",
                self.name(),
                self.controller.name(),
                color_v.len(),
                self.num_leds()
            );
        } else if color_v.len() < self.num_leds() {
            color_v.extend((color_v.len()..self.num_leds()).map(|_| Color::default()))
        }

        self.controller.set_zone_leds(self.zone_id(), color_v).await
    }

    /// Adds a segment to this zone.
    ///
    /// Controller data must be resynced using [`Controller::sync_controller_data()`]
    pub async fn add_segment(
        &self,
        name: impl Into<String>,
        start_idx: usize,
        led_count: usize,
    ) -> OpenRgbResult<()> {
        if start_idx + led_count > self.num_leds() {
            return Err(OpenRgbError::CommandError(format!(
                "Segment start index {} + count {} exceeds zone LED count {}",
                start_idx,
                led_count,
                self.num_leds()
            )));
        }

        let data = SegmentData::new(name.into(), start_idx as u32, led_count as u32);
        self.controller
            .proto()
            .add_segment(self.controller.id() as u32, self.zone_id() as u32, &data)
            .await
    }

    /// Clears the segments for this CONTROLLER.
    /// This clears all segments for all zones of the controller, not just this zone.
    ///
    /// Controller data must be resynced using [`Controller::sync_controller_data()`]
    pub async fn clear_segments(&self) -> OpenRgbResult<()> {
        self.controller.clear_segments().await
    }

    /// Returns an iterator over all segments in this zone.
    pub fn segment_iter(&'c self) -> impl Iterator<Item = Segment<'c>> {
        self.segment_data()
            .into_iter()
            .flatten()
            .map(move |s| Segment::new(self, s))
    }

    /// Resizes this zone to a new size.
    ///
    /// Controller data must be resynced using [`Controller::sync_controller_data()`]
    pub async fn resize(&self, new_size: usize) -> OpenRgbResult<()> {
        self.controller
            .proto()
            .resize_zone(
                self.controller.id() as u32,
                self.zone_id() as u32,
                new_size as u32,
            )
            .await
    }
}
