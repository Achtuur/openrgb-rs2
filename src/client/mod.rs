//! Wrapper around the `OpenRGB` client to make it friendlier to use.

mod command;
mod controller;
mod group;
mod mode;
mod plugin;
mod segment;
mod zone;

pub use {command::*, controller::*, group::*, mode::*, segment::*, zone::*};

use tokio::net::ToSocketAddrs;

use crate::{
    DEFAULT_PROTOCOL, OpenRgbError, PluginData,
    data::DeviceType,
    error::OpenRgbResult,
    protocol::{DEFAULT_ADDR, OpenRgbProtocol},
};

/// Client for the `OpenRGB` SDK server that provides methods to interact with `OpenRGB`.
///
/// By default, a connection is opened to the `OpenRGB` server at `127.0.0.1:6742`, using protocol version 5.
/// At the time of writing, the latest release (0.9) supports version 4, while the (1.0rc) release supports version 5.0.
///
///
/// # Example
pub struct OpenRgbClient {
    proto: OpenRgbProtocol,
}

impl OpenRgbClient {
    /// Connect to default `OpenRGB` server.
    ///
    /// Use [`OpenRgbClient::connect_to`] to connect to a specific server.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use openrgb2::{OpenRgbClient, OpenRgbResult};
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> OpenRgbResult<()> {
    /// let client = OpenRgbClient::connect().await?;
    /// #
    /// # Ok(())
    /// # }
    /// ```
    pub async fn connect() -> OpenRgbResult<Self> {
        Self::connect_to(DEFAULT_ADDR, DEFAULT_PROTOCOL).await
    }

    /// Connect to `OpenRGB` server at given coordinates.
    ///
    /// Use [`OpenRgbClient::connect`] to connect to default server.
    ///
    /// # Arguments
    /// * `addr` - A socket address (eg: a `(host, port)` tuple)
    ///
    /// # Example
    /// ```no_run
    /// # use openrgb2::{OpenRgbClient, OpenRgbResult};
    /// #
    /// # #[tokio::main]
    /// # async fn main() -> OpenRgbResult<()> {
    /// let client = OpenRgbClient::connect_to("127.0.0.1:6379", 5).await?;
    /// #
    /// # Ok(())
    /// # }
    /// ```
    pub async fn connect_to(
        addr: impl ToSocketAddrs + std::fmt::Debug + Copy,
        protocol_version: u32,
    ) -> OpenRgbResult<Self> {
        let client = OpenRgbProtocol::connect_to(addr, protocol_version).await?;
        Ok(Self { proto: client })
    }
}

impl OpenRgbClient {
    /// Returns all available `OpenRGB` controllers as a `ControllerGroup`.
    ///
    /// # Errors
    ///
    /// This function returns an error if communication with the `OpenRGB` SDK server fails.
    pub async fn get_all_controllers(&self) -> OpenRgbResult<ControllerGroup> {
        let count = self.proto.get_controller_count().await? as usize;
        let mut controllers = Vec::with_capacity(count);
        for id in 0..count {
            let controller = self.get_controller(id).await?;
            controllers.push(controller);
        }
        Ok(ControllerGroup::new(controllers))
    }

    /// Returns all controllers of a specific type.
    ///
    /// Use `ControllerGrou::split_per_type` to get all controllers per type.
    ///
    /// # Errors
    ///
    /// This function returns an error if communication with the `OpenRGB` SDK server fails.
    pub async fn get_controllers_of_type(
        &self,
        device_type: DeviceType,
    ) -> OpenRgbResult<ControllerGroup> {
        let group = self.get_all_controllers().await?;
        group.split_per_type().remove(&device_type).ok_or_else(|| {
            OpenRgbError::CommandError(format!("No controllers of type {device_type:?} found"))
        })
    }

    /// Gets a controller by its index.
    ///
    /// # Errors
    ///
    /// This function returns an error if communication with the `OpenRGB` SDK server fails.
    pub async fn get_controller(&self, i: usize) -> OpenRgbResult<Controller> {
        let c_data = self.proto.get_controller(i as u32).await?;
        Ok(Controller::new(i, self.proto.clone(), c_data))
    }
}

// delegation if it would exist
impl OpenRgbClient {
    /// Returns the protocol version for this client.
    pub fn get_protocol_version(&mut self) -> u32 {
        self.proto.get_protocol_version()
    }

    /// Sets the name for this client's connection.
    ///
    /// This is viewable in the `OpenRGB` SDK server tab
    pub async fn set_name(&mut self, name: impl Into<String>) -> OpenRgbResult<()> {
        self.proto.set_name(name).await
    }

    /// Returns the available profiles on `OpenRGB`
    pub async fn get_profiles(&self) -> OpenRgbResult<Vec<String>> {
        self.proto.get_profiles().await
    }

    /// Saves the current profile with the given name.
    pub async fn save_profile(&self, name: impl Into<String>) -> OpenRgbResult<()> {
        self.proto.save_profile(name).await
    }

    /// Load the profile with the given name.
    pub async fn load_profile(&self, name: impl Into<String>) -> OpenRgbResult<()> {
        self.proto.load_profile(name).await
    }

    /// Deletes the profile with the given name.
    pub async fn delete_profile(&self, name: impl Into<String>) -> OpenRgbResult<()> {
        self.proto.delete_profile(name).await
    }

    /// Returns the number of controllers connected to `OpenRGB`.
    pub async fn get_controller_count(&mut self) -> OpenRgbResult<u32> {
        self.proto.get_controller_count().await
    }

    /// Returns a list of available plugins installed on `OpenRGB`.
    pub async fn get_plugins(&self) -> OpenRgbResult<Vec<PluginData>> {
        self.proto.get_plugins().await
    }

    /// Forces the `OpenRGB` instance to rescan for devices.
    pub async fn rescan_devices(&self) -> OpenRgbResult<()> {
        self.proto.rescan_devices().await
    }
}
