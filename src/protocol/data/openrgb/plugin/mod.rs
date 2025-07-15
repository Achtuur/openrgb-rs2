mod effect;

pub use effect::*;

use crate::{DeserFromBuf, ReceivedMessage};

/// Data for OpenRGB plugins.
#[derive(Debug)]
pub struct PluginData {
    /// Plugin name
    name: String,
    /// Description of plugin
    description: String,
    /// Plugin version
    version: String,
    /// Index of this plugin. This is its id in `plugin_specific` commands.
    index: u32,
    /// Plugin's protocol version.
    plugin_protocol_version: u32,
}

impl PluginData {
    /// Returns the name of this plugin.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the description of this plugin.
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Returns the version of this plugin.
    pub fn version(&self) -> &str {
        &self.version
    }

    /// Returns the index of this plugin.
    pub fn index(&self) -> u32 {
        self.index
    }

    /// Returns the protocol version of this plugin.
    pub fn plugin_protocol_version(&self) -> u32 {
        self.plugin_protocol_version
    }
}

impl DeserFromBuf for PluginData {
    fn deserialize(buf: &mut ReceivedMessage<'_>) -> crate::OpenRgbResult<Self> {
        let name = buf.read_value()?;
        let description = buf.read_value()?;
        let version = buf.read_value()?;
        let index = buf.read_value()?;
        let plugin_protocol_version = buf.read_value()?;
        Ok(PluginData {
            name,
            description,
            version,
            index,
            plugin_protocol_version,
        })
    }
}
