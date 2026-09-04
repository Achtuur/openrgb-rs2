use std::any::type_name;

use crate::protocol::{DeserFromBuf, ReceivedMessage, SerToBuf, WriteMessage};
use crate::{OpenRgbResult, log_serde};

impl<T: DeserFromBuf> DeserFromBuf for Vec<T> {
    fn deserialize(buf: &mut ReceivedMessage<'_>) -> OpenRgbResult<Self>
    where
        Self: Sized,
    {
        let len = buf.read_value::<u16>()? as usize;
        log_serde!("Reading Vec<{:?}> of length {len}", type_name::<T>());
        let mut vec = Vec::with_capacity(len);
        for _ in 0..len {
            vec.push(buf.read_value()?);
        }
        Ok(vec)
    }
}

impl<T: SerToBuf> SerToBuf for Vec<T> {
    fn serialize(&self, buf: &mut WriteMessage) -> OpenRgbResult<()> {
        buf.write_u16(self.len() as u16);
        for t in self {
            buf.write_value(t)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use crate::WriteMessage;

    #[tokio::test]
    async fn test_read_001() -> Result<(), Box<dyn Error>> {
        let mut buf = WriteMessage::new(crate::DEFAULT_PROTOCOL);
        let mut msg = buf
            .push_value(3_u16)? // length
            .push_value(37_u8)?
            .push_value(54_u8)?
            .push_value(126_u8)?
            .to_received_msg();

        assert_eq!(msg.read_value::<Vec<u8>>()?, vec![37_u8, 54_u8, 126_u8]);

        Ok(())
    }

    #[tokio::test]
    async fn test_write_001() -> Result<(), Box<dyn Error>> {
        let mut buf = WriteMessage::new(crate::DEFAULT_PROTOCOL);
        buf.write_value(vec![1_u8, 2_u8, 3_u8])?;
        let mut msg = buf.to_received_msg();

        assert_eq!(msg.read_value::<u16>()?, 3);
        assert_eq!(msg.read_value::<u8>()?, 1);
        assert_eq!(msg.read_value::<u8>()?, 2);
        assert_eq!(msg.read_value::<u8>()?, 3);

        Ok(())
    }
}
