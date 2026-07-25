use crate::{
    DeserBorrowed, log_serde,
    protocol::{DeserFromBuf, ReceivedMessage, SerToBuf, WriteMessage},
};
use crate::{OpenRgbError, OpenRgbResult};

impl SerToBuf for String {
    fn serialize(&self, buf: &mut WriteMessage) -> OpenRgbResult<()> {
        self.as_str().serialize(buf)
    }
}

impl DeserFromBuf for String {
    fn deserialize(buf: &mut ReceivedMessage<'_>) -> OpenRgbResult<Self> {
        buf.read_borrowed::<&str>().map(move |s| s.to_owned())
    }
}

impl SerToBuf for &str {
    fn serialize(&self, buf: &mut WriteMessage) -> OpenRgbResult<()> {
        buf.write_u16(self.len() as u16 + 1); // +1 for null terminator
        buf.write_value(RawString(self))?;
        Ok(())
    }
}

impl<'de> DeserBorrowed<'de> for &'de str {
    /// Deserializes to a &str. This removes the null byte at the end.
    fn deserialize_borrowed(buf: &mut ReceivedMessage<'de>) -> OpenRgbResult<Self>
    where
        Self: 'de,
    {
        let len = buf.read_value::<u16>()? as usize;
        if len == 0 {
            return Err(OpenRgbError::ProtocolError(
                "Received empty string".to_owned(),
            ));
        }
        log_serde!("Reading string of length {len}");
        let str_with_null = buf.read_slice(len)?;
        let (last, str_bytes) = str_with_null
            .split_last()
            .expect("len must be nonzero, this is never None");

        if *last != 0 {
            return Err(OpenRgbError::ProtocolError(
                format!("Received string ({str_with_null:02X?}) does not end with null byte")
                    .to_owned(),
            ));
        }

        str::from_utf8(str_bytes).map_err(|e: std::str::Utf8Error| {
            OpenRgbError::ProtocolError(format!("Failed decoding string as UTF-8: {e}"))
        })
    }
}

/// A raw string that does not include the length in its serialized form.
///
/// If the length is needed, serialize a `&str` or `String` instead.
#[doc(hidden)]
#[derive(Debug)]
pub struct RawString<'a>(pub &'a str);

impl SerToBuf for RawString<'_> {
    fn serialize(&self, buf: &mut WriteMessage) -> OpenRgbResult<()> {
        buf.write_slice(self.0.as_bytes());
        buf.write_u8(b'\0');
        Ok(())
    }
}

impl<'de> DeserBorrowed<'de> for RawString<'de> {
    fn deserialize_borrowed(buf: &mut ReceivedMessage<'de>) -> OpenRgbResult<Self> {
        let c_str = buf.available_buf();
        let (last, str_bytes) = c_str
            .split_last()
            .ok_or_else(|| OpenRgbError::ProtocolError("Received empty string".to_owned()))?;
        if *last != 0 {
            return Err(OpenRgbError::ProtocolError(
                "Received string does not end with null byte".to_owned(),
            ));
        }
        str::from_utf8(str_bytes)
            .map_err(|e| {
                OpenRgbError::ProtocolError(format!("Failed decoding string as UTF-8: {e}"))
            })
            .map(Self)
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
            .push_value(5_u16)?
            .push_value(RawString("test"))?
            .to_received_msg();

        assert_eq!(msg.read_value::<String>()?, "test".to_owned());
        Ok(())
    }

    #[tokio::test]
    async fn test_write_001() -> Result<(), Box<dyn Error>> {
        let mut buf = WriteMessage::new(crate::DEFAULT_PROTOCOL);
        buf.write_value("test")?;
        let mut msg = buf.to_received_msg();
        assert_eq!(msg.read_value::<String>()?, "test".to_owned());
        Ok(())
    }
}
