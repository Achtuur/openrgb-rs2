use crate::protocol::{SerToBuf, WriteMessage};

impl<T: SerToBuf> SerToBuf for &[T] {
    fn serialize(&self, buf: &mut WriteMessage) -> crate::OpenRgbResult<()> {
        let len = u16::try_from(self.len()).map_err(|e| {
            crate::OpenRgbError::ProtocolError(format!("Slice is too large to encode: {e}"))
        })?;
        buf.write_u16(len);
        for item in *self {
            item.serialize(buf)?;
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
            .push_value(&3_u16)? // length
            .push_value(&37_u8)?
            .push_value(&54_u8)?
            .push_value(&126_u8)?
            .to_received_msg();

        assert_eq!(msg.read_value::<Vec<u8>>()?, vec![37_u8, 54_u8, 126_u8]);

        Ok(())
    }

    #[tokio::test]
    async fn test_write_001() -> Result<(), Box<dyn Error>> {
        let mut buf = WriteMessage::new(crate::DEFAULT_PROTOCOL);
        buf.write_value(&[1_u8, 2_u8, 3_u8])?;
        let mut msg = buf.to_received_msg();

        assert_eq!(msg.read_value::<u16>()?, 3);
        assert_eq!(msg.read_value::<u8>()?, 1);
        assert_eq!(msg.read_value::<u8>()?, 2);
        assert_eq!(msg.read_value::<u8>()?, 3);

        Ok(())
    }
}
