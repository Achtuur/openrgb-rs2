//! Support for Effects plugin packets.

use crate::impl_enum_discriminant;
use crate::{DeserFromBuf, ReceivedMessage};

pub(crate) enum EffectsPluginPacket {
    RequestEffectList = 0,
    StartEffect = 20,
    StopEffect = 21,
}

impl_enum_discriminant!(EffectsPluginPacket,
    RequestEffectList: 0,
    StartEffect: 20,
    StopEffect: 21
);

#[derive(Debug)]
pub(crate) struct PluginEffect {
    name: String,
    description: String,
    enabled: bool,
}

impl PluginEffect {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn description(&self) -> &str {
        &self.description
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }
}

impl DeserFromBuf for PluginEffect {
    fn deserialize(buf: &mut ReceivedMessage<'_>) -> crate::OpenRgbResult<Self> {
        let name = buf.read_value()?;
        let description = buf.read_value()?;
        tracing::trace!("buf after desc: {}", buf);
        let enabled = buf.read_u8()? != 0;
        Ok(PluginEffect {
            name,
            description,
            enabled,
        })
    }
}