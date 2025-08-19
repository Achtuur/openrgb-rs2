use crate::{
    client::command::Command, data::{ModeData, ModeFlag}, protocol::{
        data::{Color, ControllerData}, OpenRgbProtocol
    }, ControllerMode, ControllerModeKind, DeviceType, Led, LedData, OpenRgbError, OpenRgbResult, ZoneData
};

use super::Zone;

/// An `RGBController`, which represents a single RGB device that can be controlled.
///
/// # Example
///
/// see `examples/controller.rs` for example usage
pub struct Controller {
    id: usize,
    proto: OpenRgbProtocol,
    data: ControllerData,
}

impl PartialEq for Controller {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.data == other.data
    }
}

impl Eq for Controller {}

impl std::fmt::Debug for Controller {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Controller")
            .field("id", &self.id)
            .field("name", &self.name())
            .field("num_leds", &self.num_leds())
            .field("modes", &self.modes().len())
            .finish()
    }
}

impl Controller {
    pub(crate) fn new(id: usize, proto: OpenRgbProtocol, data: ControllerData) -> Self {
        Self { id, proto, data }
    }

    pub(crate) fn proto(&self) -> &OpenRgbProtocol {
        &self.proto
    }

    /// Returns the ID of this controller.
    pub fn id(&self) -> usize {
        self.id
    }

    delegate::delegate! {
        to self.data {
            /// Returns the name of this controller.
            pub fn name(&self) -> &str;
            /// Returns the type of this controller.
            pub fn device_type(&self) -> DeviceType;
            /// Returns the vendor of this controller.
            pub fn vendor(&self) -> &str;
            /// Returns a description for this controller.
            pub fn description(&self) -> &str;
            /// Returns the version of this controller.
            pub fn version(&self) -> &str;
            /// Returns the serial number of this controller.
            pub fn serial(&self) -> &str;
            /// Returns the location of this controller.
            pub fn location(&self) -> &str;
            /// Returns the currently set colors of this controller.
            ///
            /// These have to be manually refreshed using [`Self::sync_controller_data()`].
            pub fn colors(&self) -> &[Color];
            /// Returns the number of LEDs in this controller.
            pub fn num_leds(&self) -> usize;

            /// Returns the modes supported by this controller.
            ///
            /// [`Self::set_controllable_mode()`] will set the controller to the mode named "direct"
            pub fn modes(&self) -> &[ModeData];
            /// Returns the LEDs in this controller
            // #[expect(unused, reason = "Api not finalised yet")]
            // pub(crate) fn leds(&self) -> &[LedData];
            pub(crate) fn zones(&self) -> &[ZoneData];
            #[call(leds)]
            pub(crate) fn led_data(&self) -> &[LedData];
        }
    }

    /// Initialises a controller by setting it to a controllable mode.
    /// This function also changes the LEDs to a rainbow, so you can see if it worked.
    pub async fn init(&self) -> OpenRgbResult<()> {
        const RAINBOW_COLORS: [Color; 7] = [
            Color::new(255, 0, 0),   // Red
            Color::new(255, 127, 0), // Orange
            Color::new(255, 255, 0), // Yellow
            Color::new(0, 255, 0),   // Green
            Color::new(0, 0, 255),   // Blue
            Color::new(75, 0, 130),  // Indigo
            Color::new(148, 0, 211), // Violet
        ];
        self.set_controllable_mode().await?;
        let colors = (0..self.num_leds())
            .map(|i| i * RAINBOW_COLORS.len() / self.num_leds())
            .map(|i| RAINBOW_COLORS[i]);
        self.set_leds(colors).await?;
        Ok(())
    }

    /// Returns the active mode of this controller.
    pub fn active_mode(&self) -> ControllerMode<'_> {
        let mode = self.data.active_mode().expect(
            "OpenRGB controller has no active mode. Create an issue for this if you encouter this.",
        );
        ControllerMode::new(mode, true)
    }

    /// Returns an iterator over all available modes in this controller.
    pub fn mode_iter(&self) -> impl Iterator<Item = ControllerMode<'_>> {
        let active_mode = self.data.active_mode().expect(
            "OpenRGB controller has no active mode. Create an issue for this if you encouter this.",
        );
        self.data
            .modes()
            .iter()
            .map(|m| ControllerMode::new(m, active_mode.id() == m.id()))
    }

    /// Sets this controller to a controllable mode.
    pub async fn set_controllable_mode(&self) -> OpenRgbResult<()> {
        let mode = self
            .mode_iter()
            .find(|m| m.kind() == ControllerModeKind::Direct)
            .ok_or_else(|| OpenRgbError::ProtocolError("No controllable mode found".to_owned()))?;

        // set max brightness if possible
        if let Ok(b) = mode.builder().set_max_brightness() {
            b.execute(self).await?;
        }

        tracing::debug!("Setting {} to {} mode", self.name(), mode.name());
        self.proto
            .update_mode(self.id as u32, mode.into_data())
            .await?;
        Ok(())
    }

    pub(crate) async fn update_mode(&self, mode: &ModeData) -> OpenRgbResult<()> {
        self.proto.update_mode(self.id as u32, mode).await
    }

    /// Returns the zone with the given `zone_id`.
    pub fn get_zone(&self, zone_id: usize) -> OpenRgbResult<Zone<'_>> {
        let zone_data = self.zones().get(zone_id).ok_or_else(|| {
            OpenRgbError::CommandError(format!("Zone {zone_id} not found for {}", self.name()))
        })?;
        let zone = Zone::new(self, zone_data);
        Ok(zone)
    }

    /// Returns an iterator over all available zones in this controller.
    pub fn get_all_zones(&self) -> impl Iterator<Item = Zone<'_>> {
        self.zones().iter().map(|z| Zone::new(self, z))
    }

    /// Sets a single LED to the given `color`.
    ///
    /// When doing many writes in rapid succession, it is recommended to use the [`Self::cmd()`] method instead.
    pub async fn set_led<C: Into<Color>>(&self, led: usize, color: C) -> OpenRgbResult<()> {
        self.proto
            .update_led(self.id as u32, led as i32, &color.into())
            .await
    }

    /// Sets all LEDs of this controller to a given `color`.
    pub async fn set_all_leds<C: Into<Color>>(&self, color: C) -> OpenRgbResult<()> {
        let color = color.into();
        let colors = (0..self.num_leds()).map(|_| color);
        self.set_leds(colors).await?;
        Ok(())
    }

    /// Sets the LEDs of this controller to the given `colors`.
    pub async fn set_leds<C: Into<Color>>(
        &self,
        colors: impl IntoIterator<Item = C>,
    ) -> OpenRgbResult<()> {
        let color_v = colors.into_iter().map(Into::into).collect::<Vec<_>>();
        self.proto.update_leds(self.id as u32, &color_v).await
    }

    /// Sets the LEDs of a specific zone to the given `colors`.
    pub async fn set_zone_leds<C: Into<Color>>(
        &self,
        zone_id: usize,
        colors: impl IntoIterator<Item = C>,
    ) -> OpenRgbResult<()> {
        let color_v = colors.into_iter().map(Into::into).collect::<Vec<_>>();
        self.proto
            .update_zone_leds(self.id as u32, zone_id as u32, &color_v)
            .await
    }

    /// Turns off all LEDs of this controller.
    pub async fn turn_off_leds(&self) -> OpenRgbResult<()> {
        self.set_controllable_mode().await?;
        self.set_all_leds(Color { r: 0, g: 0, b: 0 }).await
    }

    /// Returns an iterator over the Leds of this controller.
    pub fn led_iter(&self) -> impl Iterator<Item = Led<'_>> {
        // assumption: controller_data led_data and colors agree
        self.led_data()
            .iter()
            .enumerate()
            .map(move |(id, _)| Led::new(id, self))
    }

    /// Creates a [`Command`] for this controller.
    ///
    /// Controller LEDs can be updated in three ways:
    ///  * per led: `self.set_led()`
    ///  * per zone: `self.set_zone_leds()`
    ///  * all at once: `self.set_leds()`
    ///
    /// From my testing, the most efficient way is to always update all LEDs at once.
    /// The `Command` API lets you build a command using updates to individual LEDs, zones or segments
    /// and then executes them as a single `set_led()` call.
    ///
    /// # Example
    /// ```no_run
    /// # use openrgb2::{OpenRgbClient, OpenRgbResult, Color};
    /// // let's say we have a controller with 5 LEDs
    /// # async fn example() -> OpenRgbResult<()> {
    /// let mut client = OpenRgbClient::connect().await?;
    /// let controller = client.get_controller(0).await?;
    /// // direct write
    /// controller.set_leds([Color::new(255, 0, 0); 5]).await?;
    /// // equivalent with command
    /// let mut cmd = controller.cmd();
    /// cmd.set_led(0, Color::new(255, 0, 0))?;
    /// cmd.set_led(2, Color::new(255, 0, 0))?; // order doesn't matter
    /// cmd.set_led(4, Color::new(255, 0, 0))?;
    /// cmd.set_led(1, Color::new(255, 0, 0))?;
    /// cmd.set_led(5, Color::new(255, 0, 0))?;
    /// // this is just a single api call
    /// cmd.execute().await
    /// # }
    /// ```
    ///
    /// This is especially useful for devices with multiple zones that should animate separately.
    #[must_use]
    pub fn cmd(&self) -> Command<'_> {
        Command::new(self)
    }

    /// Creates a new [`Command`] for this controller
    /// and sets the LED colors using the provided closure.
    #[must_use]
    pub fn cmd_with_leds<'a, F>(&'a self, led_clr: F) -> Command<'a>
    where F: Fn(Led<'a>) -> Color
    {
        let mut cmd = self.cmd();
        for led in self.led_iter() {
            // this cannot fail, since we know the led id will be in bounds
            cmd.set_led(led.id(), led_clr(led)).expect("Led index incorrect");
        }
        cmd
    }

    pub(crate) fn get_zone_led_offset(&self, zone_id: usize) -> OpenRgbResult<usize> {
        if zone_id >= self.zones().len() {
            return Err(OpenRgbError::ProtocolError(format!(
                "zone {zone_id} not found in controller {}",
                self.id
            )));
        }

        let offset = self
            .zones()
            .iter()
            .filter(|z| z.id < zone_id)
            .map(|z| z.leds_count as usize)
            .sum::<usize>();
        Ok(offset)
    }

    /// Fetches controller data again. This updates the state of the controller data.
    ///
    /// Currently this has to be called manually.
    pub async fn sync_controller_data(&mut self) -> OpenRgbResult<()> {
        let data = self.proto.get_controller(self.id as u32).await?;
        self.data = data;
        Ok(())
    }

    /// Saves the current mode of this controller to the flash memory of the controller.
    ///
    /// # Important
    ///
    /// Using this frequently can cause wear on the flash memory, use this sparingly.
    pub async fn save_mode(&self) -> OpenRgbResult<()> {
        let active_mode = self.active_mode();
        if !active_mode.flags().contains(ModeFlag::ManualSave) {
            return Err(OpenRgbError::CommandError(format!(
                "Controller {} mode {} cannot be saved",
                self.name(),
                active_mode.name()
            )));
        }
        self.proto
            .save_mode(self.id as u32, active_mode.into_data())
            .await
    }

    /// Clears all segments of this controller.
    pub async fn clear_segments(&self) -> OpenRgbResult<()> {
        self.proto.clear_segments(self.id as u32).await
    }
}

#[cfg(test)]
mod tests {
    use crate::OpenRgbClient;

    use super::*;

    #[tokio::test]
    #[ignore = "can only test with openrgb running"]
    async fn test_update_leds() -> OpenRgbResult<()> {
        let client = OpenRgbClient::connect().await?;
        let controller = client.get_controller(0).await?;
        controller.set_controllable_mode().await?;
        controller.set_leds([Color::new(255, 0, 50); 96]).await?;
        Ok(())
    }

    #[tokio::test]
    #[ignore = "can only test with openrgb running"]
    async fn test_cmd() -> OpenRgbResult<()> {
        let client = OpenRgbClient::connect().await?;
        let controller = client.get_controller(5).await?;
        controller.set_controllable_mode().await?;

        println!("controller: {0:#?}", controller.data.led_alt_names());

        let mut cmd = controller.cmd();
        cmd.set_led(19, Color::new(255, 0, 255))?;
        cmd.set_zone_leds(0, vec![Color::new(255, 255, 0); 19])?;
        cmd.set_zone_leds(1, vec![Color::new(0, 255, 255); 75])?;
        cmd.execute().await?;
        Ok(())
    }
}
