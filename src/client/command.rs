use std::collections::HashMap;

use crate::{
    Color, Controller, OpenRgbError, OpenRgbResult,
    client::group::{ControllerGroup, ControllerIndex},
};

/// The different type of LED updates that are possible
///
/// For use with `UpdateLedCommand` and `UpdateLedCommandGroup`.
enum SetLedCommand {
    /// Update LEDs in a controller
    Controller {
        /// The ID of the controller
        controller_id: usize,
        /// The colors to set
        colors: Vec<Color>,
    },
    /// Update LEDs in a zone in a controller
    Zone {
        /// The ID of the controller
        controller_id: usize,
        /// The ID of the zone
        zone_id: usize,
        /// The colors to set in the zone
        colors: Vec<Color>,
    },
    /// Update LEDs in a segment in a zone in a controller
    Segment {
        /// The ID of the controller
        controller_id: usize,
        /// The ID of the zone
        zone_id: usize,
        /// The ID of the segment
        segment_id: usize,
        /// The colors to set in the segment
        colors: Vec<Color>,
    },
    /// Update a single LED in a controller
    Single {
        /// The ID of the controller
        controller_id: usize,
        /// LED index in the controller
        led_id: usize,
        /// The color to set the LED to
        color: Color,
    },
}

/// An `UpdateLedCommand` for a `ControllerGroup`, which allow you to update multiple controllers.
///
/// This is useful when doing updates for multiple controllers at once.
pub struct CommandGroup<'a> {
    group: &'a ControllerGroup,
    commands: HashMap<usize, Command<'a>>,
}

impl<'a> CommandGroup<'a> {
    pub(crate) fn new(group: &'a ControllerGroup) -> Self {
        let map = group
            .controllers()
            .iter()
            .map(|c| (c.id(), Command::new(c)))
            .collect();
        Self {
            group,
            commands: map,
        }
    }

    /// Executes all commands in this group one after another.
    pub async fn execute(self) -> OpenRgbResult<()> {
        for cmd in self.commands.into_values() {
            cmd.execute().await?;
        }
        Ok(())
    }

    /// Returns a mutable reference to the command for the given controller ID.
    fn get_cmd_mut(
        &mut self,
        controller_id: impl ControllerIndex,
    ) -> OpenRgbResult<&mut Command<'a>> {
        let c = self.group.get_controller(controller_id)?;
        self.commands
            .get_mut(&c.id())
            .ok_or(OpenRgbError::CommandError(format!(
                "Controller with id {} not found in group",
                c.id()
            )))
    }

    /// Add a command to update a single LED in a controller.
    ///
    /// # Errors
    ///
    /// Returns an error if the controller is not found in this group.
    pub fn set_controller_led(
        &mut self,
        controller_id: impl ControllerIndex,
        led_id: usize,
        color: Color,
    ) -> OpenRgbResult<()> {
        let cmd = self.get_cmd_mut(controller_id)?;
        cmd.set_led(led_id, color)
    }

    /// Add a command to update a multiple LEDs in a controller
    ///
    /// # Errors
    ///
    /// Returns an error if the controller is not found in this group.
    pub fn set_controller_leds(
        &mut self,
        controller_id: impl ControllerIndex,
        colors: impl IntoIterator<Item = Color>,
    ) -> OpenRgbResult<()> {
        let cmd = self.get_cmd_mut(controller_id)?;
        cmd.set_leds(colors)
    }

    /// Add a command to update a zone in a controller.
    ///
    /// # Errors
    ///
    /// Returns an error if the controller is not found in this group.
    pub fn set_controller_zone_leds(
        &mut self,
        controller_id: impl ControllerIndex,
        zone_id: usize,
        colors: impl IntoIterator<Item = Color>,
    ) -> OpenRgbResult<()> {
        let cmd = self.get_cmd_mut(controller_id)?;
        cmd.set_zone_leds(zone_id, colors)
    }

    /// Add a command to update a single LED in a zone in a controller.
    ///
    /// # Errors
    ///
    /// Returns an error if the controller is not found in this group.
    pub fn set_controller_zone_led(
        &mut self,
        controller_id: impl ControllerIndex,
        zone_id: usize,
        led_idx: usize,
        color: Color,
    ) -> OpenRgbResult<()> {
        let cmd = self.get_cmd_mut(controller_id)?;
        cmd.set_zone_led(zone_id, led_idx, color)
    }

    /// Add a command to update a segment in a zone in a controller.
    ///
    /// # Errors
    ///
    /// Returns an error if the controller is not found in this group.
    pub fn set_controller_segment_leds(
        &mut self,
        controller_id: impl ControllerIndex,
        zone_id: usize,
        segment_id: usize,
        colors: impl IntoIterator<Item = Color>,
    ) -> OpenRgbResult<()> {
        let cmd = self.get_cmd_mut(controller_id)?;
        cmd.set_segment_leds(zone_id, segment_id, colors)
    }
}

/// A command to update the LEDs in a controller.
/// Commands must be executed using `[Self::execute()]`. Use `[Controller::cmd()]` to create a command.
///
/// When executed, all commands are combined into a single `UpdateCommand::Controller`,
/// meaning only a single update is actually sent to the controller.
///
/// When two commands write to the same LED, the last command will overwrite the previous one.
#[derive(Debug)]
pub struct Command<'a> {
    controller: &'a Controller,
    colors: Vec<Color>,
}

impl<'a> Command<'a> {
    pub(crate) fn new(controller: &'a Controller) -> Self {
        Self {
            controller,
            colors: Vec::with_capacity(controller.num_leds()),
        }
    }

    /// Executes this command, sending the update to the controller.
    pub async fn execute(self) -> OpenRgbResult<()> {
        self.controller.set_leds(self.colors).await?;
        // self.controller.sync_controller_data().await?;
        Ok(())
    }

    /// Adds a command to update a single LED in this controller.
    pub fn set_led(&mut self, led_id: usize, color: Color) -> OpenRgbResult<()> {
        self.add_command(SetLedCommand::Single {
            controller_id: self.controller.id(),
            led_id,
            color,
        })
    }

    /// Adds a command to update multiple LEDs in this controller.
    pub fn set_leds(&mut self, colors: impl IntoIterator<Item = Color>) -> OpenRgbResult<()> {
        self.add_command(SetLedCommand::Controller {
            controller_id: self.controller.id(),
            colors: colors.into_iter().collect(),
        })
    }

    /// Adds a command to update a single LED in a zone in this controller.
    pub fn set_zone_led(
        &mut self,
        zone_id: usize,
        led_idx: usize,
        color: Color,
    ) -> OpenRgbResult<()> {
        self.add_command(SetLedCommand::Single {
            controller_id: self.controller.id(),
            led_id: self.controller.get_zone_led_offset(zone_id)? + led_idx,
            color,
        })
    }

    /// Adds a command to update multiple LEDs in a zone in this controller.
    pub fn set_zone_leds(&mut self, zone_id: usize, colors: impl IntoIterator<Item = Color>) -> OpenRgbResult<()> {
        self.add_command(SetLedCommand::Zone {
            controller_id: self.controller.id(),
            zone_id,
            colors: colors.into_iter().collect(),
        })
    }

    /// Adds a command to update a single led in a segment in a zone in this controller.
    pub fn set_segment_led(
        &mut self,
        zone_id: usize,
        segment_id: usize,
        led_idx: usize,
        color: Color,
    ) -> OpenRgbResult<()> {
        let led_id = led_idx
            + self
                .controller
                .get_zone(zone_id)?
                .get_segment(segment_id)?
                .offset();
        self.add_command(SetLedCommand::Single {
            controller_id: self.controller.id(),
            led_id,
            color,
        })
    }

    /// Adds a command to update multiple LEDs in a segment in a zone in this controller.
    pub fn set_segment_leds(
        &mut self,
        zone_id: usize,
        segment_id: usize,
        colors: impl IntoIterator<Item = Color>,
    ) -> OpenRgbResult<()> {
        self.add_command(SetLedCommand::Segment {
            controller_id: self.controller.id(),
            zone_id,
            segment_id,
            colors: colors.into_iter().collect(),
        })
    }

    /// Adds an `UpdateCommand` to this command.
    fn add_command(&mut self, cmd: SetLedCommand) -> OpenRgbResult<()> {
        match cmd {
            SetLedCommand::Controller {
                controller_id: _,
                colors,
            } => {
                if colors.len() > self.controller.num_leds() {
                    tracing::warn!(
                        "Controller {} was given {} colors, while its length is {}. This might become a hard error in the future.",
                        self.controller.name(),
                        colors.len(),
                        self.controller.num_leds()
                    )
                }

                self.set_colors(0, &colors)?;
            }
            SetLedCommand::Zone {
                controller_id: _,
                zone_id,
                colors,
            } => {
                let zone = self.controller.get_zone(zone_id)?;
                if colors.len() >= zone.num_leds() {
                    tracing::warn!(
                        "Zone {} for controller {} was given {} colors, while its length is {}. This might become a hard error in the future.",
                        zone_id,
                        self.controller.name(),
                        colors.len(),
                        zone.num_leds()
                    )
                }

                let offset = self.controller.get_zone_led_offset(zone_id)?;
                let len = colors.len().min(zone.num_leds());
                self.set_colors(offset, &colors[..len])?;
            }
            SetLedCommand::Segment {
                controller_id: _,
                zone_id,
                segment_id,
                colors,
            } => {
                let zone = self.controller.get_zone(zone_id)?;
                let seg = zone.get_segment(segment_id)?;
                if colors.len() >= seg.num_leds() {
                    tracing::warn!(
                        "Segment {} for zone {} in controller {} was given {} colors, while its length is {}. This might become a hard error in the future.",
                        seg.name(),
                        zone_id,
                        self.controller.name(),
                        colors.len(),
                        seg.num_leds()
                    )
                }

                let offset = zone.offset() + seg.offset();
                self.set_colors(offset, &colors)?;
            }
            SetLedCommand::Single {
                controller_id: _,
                led_id,
                color,
            } => {
                if led_id >= self.controller.num_leds() {
                    tracing::warn!(
                        "LED id {} is out of bounds for controller {} with {} LEDs",
                        led_id,
                        self.controller.name(),
                        self.controller.num_leds()
                    );
                }
                self.set_colors(led_id, &[color])?;
            }
        }
        Ok(())
    }

    /// This is only called internally, so it is safe to assume that the colors are properly bounded
    fn set_colors(&mut self, offset: usize, colors: &[Color]) -> OpenRgbResult<()> {
        let len = offset + colors.len();
        if self.colors.len() < len {
            self.colors.resize(len, Color::default());
        }
        self.colors[offset..len].copy_from_slice(colors);
        Ok(())
    }
}
