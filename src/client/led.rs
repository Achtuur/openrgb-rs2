use crate::{Color, Command, Controller, OpenRgbResult};

/// A single LED of a controller
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Led<'c> {
    id: usize,
    controller: &'c Controller,
    name: &'c str,
    color: Color,
}

impl<'c> Led<'c> {
    pub(crate) fn new(id: usize, parent: &'c Controller) -> Self {
        let name = parent.led_data().get(id).map(|ld| ld.name()).expect("Led::new() called with invalid parameters");
        let color = parent.colors().get(id).copied().expect("Led::new() called with invalid parameters");
        Self {
            id,
            controller: parent,
            name,
            color,
        }
    }

    /// Returns the ID of this LED. This is equal to the index of this LED in the controller's color array
    pub fn id(&self) -> usize {
        self.id
    }

    /// Returns the name of this LED.
    ///
    /// It depends on the controller what kind of name is here,
    /// for keyboards this is usually the name of the keys.
    pub fn name(&self) -> &str {
        self.name
    }

    /// Returns color of this LED after the last [`Controller:sync_controller_data()`] call.
    pub fn color(&self) -> Color {
        self.color
    }

    /// Creates a command with the given `color`
    pub fn cmd_with_color<C: Into<Color>>(&self, color: C) -> Command<'_> {
        let mut cmd = self.controller.cmd();
        cmd.set_led(self.id, color.into()).expect("Failed to set LED color");
        cmd
    }

    /// Sets this LED to the given `color`.
    ///
    /// It's recommended to use the `Command` API (See `[Controller::cmd()]`) instead
    /// when doing many successive writes to many leds.
    pub async fn set_led<C: Into<Color>>(&self, color: C) -> OpenRgbResult<()> {
        self.controller.set_led(self.id, color).await
    }
}