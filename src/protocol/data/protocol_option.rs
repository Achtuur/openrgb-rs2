use crate::{
    SerToBuf, WriteMessage,
    protocol::serialize::{DeserFromBuf, ReceivedMessage},
};

/// Option that can be used to represent values not supported by the current protocol version.
/// `VER` is the _minimum_ supported version. `ProtocolOption<5, T>` means that from sdk version 5 onwards, this will always be `Some(T)`
///
/// If protocol version is suppported, this is just an `T`.
/// If not, then this is always `ProtocolOption::UnsupportedVersion`.
///
/// Useful when determining sizes of data structures that contains fields that may not be supported by the current protocol version.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ProtocolOption<const VER: usize, T> {
    /// Value is supported by the current protocol version.
    Some(T),
    /// Value is not supported by the current protocol version.
    UnsupportedVersion,
}

impl<const VER: usize, T> std::ops::Deref for ProtocolOption<VER, T> {
    type Target = Option<T>;

    fn deref(&self) -> &Self::Target {
        let transmute: &Option<T> = unsafe { std::mem::transmute(self) };
        transmute
    }
}

impl<const VER: usize, T: Default> std::default::Default for ProtocolOption<VER, T> {
    fn default() -> Self {
        ProtocolOption::Some(T::default())
    }
}

impl<const VER: usize, T> From<ProtocolOption<VER, T>> for Option<T> {
    fn from(value: ProtocolOption<VER, T>) -> Self {
        match value {
            ProtocolOption::Some(v) => Some(v),
            ProtocolOption::UnsupportedVersion => None,
        }
    }
}

impl<const VER: usize, T> ProtocolOption<VER, T> {
    /// Creates a new `ProtocolOption` with the given value if the protocol version is supported.
    pub fn new(val: T, version: usize) -> Self {
        if version < VER {
            return Self::UnsupportedVersion;
        }
        Self::Some(val)
    }

    /// Replaces the value in this `ProtocolOption`
    pub fn replace(&mut self, val: T) {
        match self {
            Self::Some(_) => *self = Self::Some(val),
            Self::UnsupportedVersion => (),
        }
    }

    /// Returns `Some(&T)` if the value is supported by the current protocol version, otherwise `None`.
    pub fn value(&self) -> Option<&T> {
        match self {
            Self::Some(v) => Some(v),
            Self::UnsupportedVersion => None,
        }
    }

    /// Returns `Some(&mut T)` if the value is supported by the current protocol version, otherwise `None`.
    pub fn value_mut(&mut self) -> Option<&mut T> {
        match self {
            Self::Some(v) => Some(v),
            Self::UnsupportedVersion => None,
        }
    }
}

impl<const VER: usize, T> DeserFromBuf for ProtocolOption<VER, T>
where
    T: DeserFromBuf,
{
    fn deserialize(buf: &mut ReceivedMessage<'_>) -> crate::OpenRgbResult<Self> {
        if buf.protocol_version() < VER as u32 {
            return Ok(ProtocolOption::UnsupportedVersion);
        }
        let val = T::deserialize(buf)?;
        Ok(ProtocolOption::Some(val))
    }
}

impl<const VER: usize, T> SerToBuf for ProtocolOption<VER, T>
where
    T: SerToBuf,
{
    fn serialize(&self, buf: &mut WriteMessage) -> crate::OpenRgbResult<()> {
        if buf.protocol_version() < VER as u32 {
            return Ok(());
        }

        match self {
            Self::Some(v) => v.serialize(buf),
            Self::UnsupportedVersion => Ok(()), // No write if this came from unsupported read
        }
    }
}
