use std::marker::PhantomData;

use crate::{Controller, Direction, ModeData, ModeFlag, OpenRgbError, OpenRgbResult};

pub use flagset::FlagSet;

/// Kinds of modes that exist. `Direct` is usually the mode you want to use.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum ControllerModeKind<'c> {
    /// Direct mode, leds can be controlled directly. This is the mode you probably want.
    Direct,
    /// Static mode, l
    Static,
    /// Some device specific custom mode, like a gradient effect.
    Custom(&'c str),
}

impl<'c> ControllerModeKind<'c> {
    pub(crate) fn new(mode: &'c ModeData) -> Self {
        match mode.name() {
            "Direct" | "direct" => Self::Direct,
            "Static" | "static" => Self::Static,
            _ => Self::Custom(mode.name()),
        }
    }
}

/// Mode of a controller.
pub struct ControllerMode<'c> {
    data: &'c ModeData,
    kind: ControllerModeKind<'c>,
    is_active: bool,
}

impl<'c> ControllerMode<'c> {
    pub(crate) fn new(data: &'c ModeData, is_active: bool) -> Self {
        let kind = ControllerModeKind::new(data);
        Self {
            data,
            kind,
            is_active,
        }
    }

    pub(crate) fn into_data(self) -> &'c ModeData {
        self.data
    }

    /// The kind of mode this is.
    pub fn kind(&self) -> ControllerModeKind<'c> {
        self.kind
    }

    /// Whether this mode is currently active.
    pub fn is_active(&self) -> bool {
        self.is_active
    }

    /// Creates a [`ControllerModeBuilder`] for this mode.
    ///
    /// This lets you configure the mode and change it on the controller.
    pub fn builder(&self) -> ControllerModeBuilder<'c> {
        ControllerModeBuilder::new(self.data)
    }

    delegate::delegate! {
        to self.data {
            /// The name of this mode.
            pub fn name(&self) -> &str;
            /// The flags of this mode.
            pub(crate) fn flags(&self) -> FlagSet<ModeFlag>;
            /// The speed of this mode, if available.
            pub fn speed(&self) -> Option<u32>;
            /// The minimum speed of this mode, if available.
            pub fn speed_min(&self) -> Option<u32>;
            /// The maximum speed of this mode, if available.
            pub fn speed_max(&self) -> Option<u32>;
            /// The brightness of this mode, if available.
            pub fn brightness(&self) -> Option<u32>;
            /// The minimum brightness of this mode, if available.
            pub fn brightness_min(&self) -> Option<u32>;
            /// The maximum brightness of this mode, if available.
            pub fn brightness_max(&self) -> Option<u32>;
            /// The direction of this mode, if available.
            pub fn direction(&self) -> Option<Direction>;
        }
    }
}

/// Builder for a controller mode.
///
/// This lets you configure the mode, call `execute()` to apply the changes.
pub struct ControllerModeBuilder<'c> {
    data: ModeData,
    _phantom: PhantomData<&'c ()>,
}

impl ControllerModeBuilder<'_> {
    pub(crate) fn new(data: &ModeData) -> Self {
        Self {
            data: data.clone(),
            _phantom: PhantomData,
        }
    }

    /// Applies the changes made in this builder to the controller.
    ///
    /// Remember to call [`Controller::sync_controller_data()`] to sync the changes back.
    pub async fn execute(&self, controller: &Controller) -> OpenRgbResult<()> {
        controller.update_mode(&self.data).await?;
        Ok(())
    }

    /// Sets the speed of this mode.
    ///
    /// # Errors
    ///
    /// Returns an error if the speed is out of bounds, or if this mode does not support speed.
    pub fn set_speed(&mut self, speed: u32) -> OpenRgbResult<&mut Self> {
        if !self.data.flags().contains(ModeFlag::HasSpeed) {
            return Err(OpenRgbError::CommandError(
                "Mode does not support speed".to_string(),
            ));
        }

        let (Some(speed_min), Some(speed_max)) = (self.data.speed_min(), self.data.speed_max()) else {
            return Err(OpenRgbError::CommandError(
                "Mode does not support speed".to_string(),
            ));
        };

        if speed < speed_min || speed > speed_max {
            return Err(OpenRgbError::CommandError(format!(
                "Speed must be between {speed_min} and {speed_max}, got: {speed}",
            )));
        }

        self.data.set_speed(speed);
        Ok(self)
    }

    /// Sets the speed of this mode to the minimum speed.
    ///
    /// # Errors
    ///
    /// Returns an error if this mode does not support speed.
    pub fn set_min_speed(&mut self) -> OpenRgbResult<&mut Self> {
        match self.data.speed_min() {
            Some(max_speed) => self.set_speed(max_speed),
            None => Err(OpenRgbError::CommandError(
                "Mode does not support speed".to_string(),
            )),
        }
    }

    /// Sets the speed of this mode to the maximum speed.
    /// 
    /// # Errors
    ///
    /// Returns an error if this mode does not support speed.
    pub fn set_max_speed(&mut self) -> OpenRgbResult<&mut Self> {
        match self.data.speed_max() {
            Some(max_speed) => self.set_speed(max_speed),
            None => Err(OpenRgbError::CommandError(
                "Mode does not support speed".to_string(),
            )),
        }
    }

    /// Sets the brightness of this mode.
    ///
    /// # Errors
    ///
    /// Returns an error if the brightness is out of bounds, or if this mode does not support brightness.
    pub fn set_brightness(&mut self, brightness: u32) -> OpenRgbResult<&mut Self> {
        if !self.data.flags().contains(ModeFlag::HasBrightness) {
            return Err(OpenRgbError::CommandError(
                "Mode does not support brightness".to_string(),
            ));
        }

        let (Some(brightness_min), Some(brightness_max)) = (self.data.brightness_min(), self.data.brightness_max()) else {
            return Err(OpenRgbError::CommandError(
                "Mode does not support brightness".to_string(),
            ));
        };

        if brightness < brightness_min || brightness > brightness_max {
            return Err(OpenRgbError::CommandError(format!(
                "Brightness must be between {brightness_min} and {brightness_max}, got: {brightness}",
            )));
        }

        self.data.set_brightness(brightness);
        Ok(self)
    }

    /// Sets the brightness of this mode to the minimum brightness.
    ///
    /// # Errors
    ///
    /// Returns an error if this mode does not support brightness.
    pub fn set_max_brightness(&mut self) -> OpenRgbResult<&mut Self> {
        match self.data.brightness_max() {
            Some(max_brightness) => self.set_brightness(max_brightness),
            None => Err(OpenRgbError::CommandError(
                "Mode does not support brightness".to_string(),
            )),
        }
    }

    /// Sets the brightness of this mode to the minimum brightness.
    ///
    /// # Errors
    ///
    /// Returns an error if this mode does not support brightness.
    pub fn set_min_brightness(&mut self) -> OpenRgbResult<&mut Self> {
        match self.data.brightness_min() {
            Some(min_brightness) => self.set_brightness(min_brightness),
            None => Err(OpenRgbError::CommandError(
                "Mode does not support brightness".to_string(),
            )),
        }
    }

    /// Sets the direction of this mode.
    ///
    /// # Errors
    ///
    /// Returns an error if this mode does not support a direction.
    pub fn set_direction(&mut self, dir: Direction) -> OpenRgbResult<&mut Self> {
        if !self.data.flags().contains(ModeFlag::HasDirection) {
            return Err(OpenRgbError::CommandError(
                "Mode does not support direction".to_string(),
            ));
        }

        self.data.set_direction(dir);
        Ok(self)
    }
}