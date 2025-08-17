use std::io::Read;

use crate::protocol::{DeserFromBuf, ReceivedMessage, SerToBuf, WriteMessage};
use crate::{OpenRgbError, OpenRgbResult};

impl DeserFromBuf for String {
    fn deserialize(buf: &mut ReceivedMessage<'_>) -> OpenRgbResult<Self>
    where
        Self: Sized,
    {
        let len = buf.read_u16()? as usize;
        let mut bytes = vec![0u8; len];
        buf.read_exact(&mut bytes)?;
        bytes.pop(); // null byte?
        String::from_utf8(bytes).map_err(|e| {
            OpenRgbError::ProtocolError(format!("Failed decoding string as UTF-8: {e}"))
        })
    }
}

impl SerToBuf for String {
    fn serialize(&self, buf: &mut WriteMessage) -> OpenRgbResult<()> {
        self.as_str().serialize(buf)
    }
}

impl SerToBuf for &str {
    fn serialize(&self, buf: &mut WriteMessage) -> OpenRgbResult<()> {
        buf.write_u16(self.len() as u16 + 1); // +1 for null terminator
        buf.write_value(&RawString(self))?;
        Ok(())
    }
}

/// A raw string that does not include the length in its serialized form.
///
/// If the length is needed, serialize a `&str` or `String` instead.
#[doc(hidden)]
pub struct RawString<'a>(pub &'a str);

impl SerToBuf for RawString<'_> {
    fn serialize(&self, buf: &mut WriteMessage) -> OpenRgbResult<()> {
        buf.write_slice(self.0.as_bytes());
        buf.write_u8(b'\0');
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use crate::WriteMessage;
    use crate::protocol::data::implement::string::RawString;

    #[tokio::test]
    async fn test_read_001() -> Result<(), Box<dyn Error>> {
        let mut buf = WriteMessage::new(crate::DEFAULT_PROTOCOL);
        let mut msg = buf
            .push_value(&5_u16)?
            .push_value(&RawString("test"))?
            .to_received_msg();

        assert_eq!(msg.read_value::<String>()?, "test".to_owned());
        Ok(())
    }

    #[tokio::test]
    async fn test_write_001() -> Result<(), Box<dyn Error>> {
        let mut buf = WriteMessage::new(crate::DEFAULT_PROTOCOL);
        buf.write_value(&"test")?;
        let mut msg = buf.to_received_msg();
        assert_eq!(msg.read_value::<String>()?, "test".to_owned());
        Ok(())
    }
}
