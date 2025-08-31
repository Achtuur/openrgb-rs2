use std::io::Write;

use crate::OpenRgbResult;
#[cfg(test)]
use crate::ReceivedMessage;

/// Serialize an object to a byte buffer.
pub(crate) trait SerToBuf {
    fn serialize(&self, buf: &mut WriteMessage) -> OpenRgbResult<()>;
}

impl<T: SerToBuf> SerToBuf for &T {
    fn serialize(&self, buf: &mut WriteMessage) -> OpenRgbResult<()> {
        (*self).serialize(buf)
    }
}

pub(crate) struct WriteMessage {
    protocol_version: u32,
    buf: Vec<u8>,
}

impl std::fmt::Display for WriteMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "WriteMessage (protocol: {}, len: {}): {:?}",
            self.protocol_version,
            self.buf.len(),
            &*self.buf
        )
    }
}

impl WriteMessage {
    pub fn new(protocol_version: u32) -> Self {
        Self::with_capacity(protocol_version, 8)
    }

    pub fn with_capacity(protocol_version: u32, capacity: usize) -> Self {
        Self {
            protocol_version,
            buf: Vec::with_capacity(capacity),
        }
    }

    pub fn len(&self) -> usize {
        self.buf.len()
    }

    pub fn bytes(&self) -> &[u8] {
        &self.buf
    }

    pub fn protocol_version(&self) -> u32 {
        self.protocol_version
    }

    pub fn write_u8(&mut self, value: u8) {
        self.buf.push(value);
    }

    pub fn write_u16(&mut self, value: u16) {
        let _ = self.write(&value.to_le_bytes());
    }

    pub fn write_u32(&mut self, value: u32) {
        let _ = self.write(&value.to_le_bytes());
    }

    pub fn write_value<T: SerToBuf>(&mut self, value: &T) -> OpenRgbResult<()> {
        value.serialize(self)
    }

    pub fn write_slice(&mut self, slice: &[u8]) {
        self.buf.extend_from_slice(slice);
    }

    pub fn push_value<T: SerToBuf>(&mut self, value: &T) -> OpenRgbResult<&mut Self> {
        self.write_value(value)?;
        Ok(self)
    }

    #[cfg(test)]
    pub fn to_received_msg(&self) -> ReceivedMessage<'_> {
        ReceivedMessage::new(&self.buf, self.protocol_version)
    }
}

impl std::io::Write for WriteMessage {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buf.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
