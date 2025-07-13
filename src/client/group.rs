use std::collections::HashMap;

use crate::{
    Controller, OpenRgbError, OpenRgbResult, client::command::CommandGroup, data::DeviceType,
};

/// Trait for things that can index into a `ControllerGroup`.
///
/// Currently includes `usize` (`Controller::id()`) and `&Controller`.
pub trait ControllerIndex {
    /// Returns a reference to the controller with the given index.
    fn index(self, group: &ControllerGroup) -> OpenRgbResult<&Controller>;
    /// Removes the controller with the given index from the group and returns it.
    fn remove(self, group: &mut ControllerGroup) -> OpenRgbResult<Controller>;
}

impl ControllerIndex for usize {
    fn index(self, group: &ControllerGroup) -> OpenRgbResult<&Controller> {
        group
            .controllers
            .get(self)
            .ok_or(OpenRgbError::CommandError(format!(
                "Controller with index {self} not found"
            )))
    }

    fn remove(self, group: &mut ControllerGroup) -> OpenRgbResult<Controller> {
        if self >= group.controllers.len() {
            return Err(OpenRgbError::CommandError(format!(
                "Controller with index {self} not found"
            )));
        }
        Ok(group.controllers.remove(self))
    }
}

impl ControllerIndex for &Controller {
    fn index(self, group: &ControllerGroup) -> OpenRgbResult<&Controller> {
        self.id().index(group)
    }

    fn remove(self, group: &mut ControllerGroup) -> OpenRgbResult<Controller> {
        self.id().remove(group)
    }
}

impl ControllerIndex for Controller {
    fn index(self, group: &ControllerGroup) -> OpenRgbResult<&Controller> {
        (&self).index(group)
    }

    fn remove(self, group: &mut ControllerGroup) -> OpenRgbResult<Controller> {
        (&self).remove(group)
    }
}

/// A group of controllers, this is used to manage multiple controllers at once.
#[derive(Debug)]
pub struct ControllerGroup {
    controllers: Vec<Controller>,
}

impl ControllerGroup {
    pub(crate) fn new(controllers: Vec<Controller>) -> Self {
        Self { controllers }
    }

    fn empty() -> Self {
        Self {
            controllers: Vec::new(),
        }
    }

    /// Returns a reference to the controllers in this group.
    pub fn controllers(&self) -> &[Controller] {
        &self.controllers
    }

    /// Returns true if this group has no controllers.
    pub fn is_empty(&self) -> bool {
        self.controllers.is_empty()
    }

    /// Returns the number of controllers in this group.
    pub fn len(&self) -> usize {
        self.controllers.len()
    }

    /// Returns an iterator over the controllers in this group.
    pub fn iter(&self) -> impl Iterator<Item = &Controller> {
        self.controllers.iter()
    }

    /// Returns a reference to the controller with the given index.
    ///
    /// The index can be either a `usize` or a `Controller` reference.
    pub fn get_controller<I>(&self, idx: I) -> OpenRgbResult<&Controller>
    where
        I: ControllerIndex,
    {
        idx.index(self)
    }

    /// Splits the controllers in this group by their device type.
    /// Returns one group per device type.
    pub fn split_per_type(self) -> HashMap<DeviceType, ControllerGroup> {
        self.controllers
            .into_iter()
            .fold(HashMap::new(), |mut acc, controller| {
                let entry = acc
                    .entry(controller.device_type())
                    .or_insert_with(ControllerGroup::empty);
                entry.controllers.push(controller);
                acc
            })
    }

    /// Returns an iterator over controllers of the given device type.
    pub fn get_by_type(&self, device_type: DeviceType) -> impl Iterator<Item = &Controller> {
        self.controllers
            .iter()
            .filter(move |c| c.device_type() == device_type)
    }

    /// Returns the first controller in this group.
    ///
    /// This is useful when you know there is only one controller in the group,
    /// such as after calling `[split_per_type()]`.
    pub fn into_first(self) -> OpenRgbResult<Controller> {
        self.controllers
            .into_iter()
            .next()
            .ok_or(OpenRgbError::CommandError(
                "No controllers in group".to_string(),
            ))
    }

    /// Creates a new `CommandGroup` for this controller group.
    ///
    /// See `Controller::cmd()` for more information.
    pub fn cmd(&self) -> CommandGroup {
        CommandGroup::new(self)
    }

    /// Initializes all controllers in this group.
    pub async fn init(&self) -> OpenRgbResult<()> {
        for controller in &self.controllers {
            controller.init().await?;
        }
        Ok(())
    }

    /// Set all controllers in this group to controllable mode.
    pub async fn set_controllable_mode(&self) -> OpenRgbResult<()> {
        for controller in &self.controllers {
            controller.set_controllable_mode().await?;
        }
        Ok(())
    }

    /// Turns off all LEDs in all controllers in this group.
    pub async fn turn_off_leds(&self) -> OpenRgbResult<()> {
        for controller in &self.controllers {
            controller.turn_off_leds().await?;
        }
        Ok(())
    }
}

impl IntoIterator for ControllerGroup {
    type Item = Controller;
    type IntoIter = <Vec<Controller> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.controllers.into_iter()
    }
}

impl<'a> IntoIterator for &'a ControllerGroup {
    type Item = &'a Controller;
    type IntoIter = <&'a Vec<Controller> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.controllers.iter()
    }
}

#[cfg(test)]
mod tests {
    use crate::OpenRgbClient;

    use super::*;

    #[tokio::test]
    #[ignore = "can only test with openrgb running"]
    async fn test_group() -> OpenRgbResult<()> {
        let client = OpenRgbClient::connect().await?;
        let group = client.get_all_controllers().await?;
        group.init().await?;
        Ok(())
    }

    #[tokio::test]
    #[ignore = "can only test with openrgb running"]
    async fn test_per_type() -> OpenRgbResult<()> {
        let client = OpenRgbClient::connect().await?;
        let group = client.get_all_controllers().await?;
        let split = group.split_per_type();
        for (device_type, controllers) in split {
            println!("Device type: {device_type:?}");
            for controller in controllers.controllers() {
                println!("  Controller: {} ({})", controller.name(), controller.id());
            }
        }
        Ok(())
    }
}
