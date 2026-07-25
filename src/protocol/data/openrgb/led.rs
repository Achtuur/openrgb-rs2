use crate::protocol::{DeserFromBuf, ReceivedMessage};
use crate::{OpenRgbResult, SerToBuf};

/// A single LED.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct LedData {
    /// LED name.
    pub name: String,

    /// LED value.
    ///
    /// This is some internal flag, basically of no use to us
    value: Option<u32>,
}

impl DeserFromBuf for LedData {
    fn deserialize(buf: &mut ReceivedMessage<'_>) -> OpenRgbResult<Self>
    where
        Self: Sized,
    {
        let name = buf.read_value()?;
        let value = if buf.protocol_version() < 6 {
            Some(buf.read_value()?)
        } else {
            None
        };

        Ok(LedData { name, value })
    }
}

impl SerToBuf for LedData {
    fn serialize(&self, buf: &mut crate::WriteMessage) -> OpenRgbResult<()> {
        buf.write_value(&self.name)?;
        if buf.protocol_version() < 6 {
            buf.write_value(
                self.value
                    .expect("LED value must be set for protocol versions lower than 6"),
            )?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use crate::{LedData, ReceivedMessage};

    #[tokio::test]
    async fn test_read_v5() -> Result<(), Box<dyn Error>> {
        let mut buf = Vec::<u8>::new();
        buf.extend(((String::from("Led name 123").len() + 1) as u16).to_le_bytes()); // + 1 for null terminator
        buf.extend(b"Led name 123\0");
        buf.extend(46_u32.to_le_bytes());
        let mut reader = ReceivedMessage::new(&buf, 5);

        let led: LedData = reader.read_value()?;

        assert_eq!(led.name, "Led name 123");
        assert_eq!(led.value, Some(46));

        Ok(())
    }

    #[tokio::test]
    async fn test_read_v6() -> Result<(), Box<dyn Error>> {
        let mut buf = Vec::<u8>::new();
        buf.extend(((String::from("Led name 123").len() + 1) as u16).to_le_bytes()); // + 1 for null terminator
        buf.extend(b"Led name 123\0");
        let mut reader = ReceivedMessage::new(&buf, 6);

        let led: LedData = reader.read_value()?;

        assert_eq!(led.name, "Led name 123");
        assert_eq!(led.value, None);

        Ok(())
    }
}
