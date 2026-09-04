use crate::OpenRgbResult;
use crate::protocol::{DeserFromBuf, ReceivedMessage, SerToBuf, WriteMessage};

impl DeserFromBuf for () {
    fn deserialize(_buf: &mut ReceivedMessage<'_>) -> OpenRgbResult<Self> {
        Ok(())
    }
}

impl SerToBuf for () {
    fn serialize(&self, _buf: &mut WriteMessage) -> OpenRgbResult<()> {
        Ok(())
    }
}

impl DeserFromBuf for u8 {
    fn deserialize(buf: &mut ReceivedMessage<'_>) -> OpenRgbResult<Self> {
        buf.read_u8()
    }
}

impl SerToBuf for u8 {
    fn serialize(&self, buf: &mut WriteMessage) -> OpenRgbResult<()> {
        buf.write_u8(*self);
        Ok(())
    }
}

impl DeserFromBuf for u16 {
    fn deserialize(buf: &mut ReceivedMessage<'_>) -> OpenRgbResult<Self> {
        buf.read_u16()
    }
}

impl SerToBuf for u16 {
    fn serialize(&self, buf: &mut WriteMessage) -> OpenRgbResult<()> {
        buf.write_u16(*self);
        Ok(())
    }
}

impl DeserFromBuf for u32 {
    fn deserialize(buf: &mut ReceivedMessage<'_>) -> OpenRgbResult<Self> {
        buf.read_u32()
    }
}

impl SerToBuf for u32 {
    fn serialize(&self, buf: &mut WriteMessage) -> OpenRgbResult<()> {
        buf.write_u32(*self);
        Ok(())
    }
}

impl DeserFromBuf for i32 {
    fn deserialize(buf: &mut ReceivedMessage<'_>) -> OpenRgbResult<Self> {
        let x = buf.read_value::<u32>()?;
        Ok(x as i32)
    }
}

impl SerToBuf for i32 {
    fn serialize(&self, buf: &mut WriteMessage) -> OpenRgbResult<()> {
        buf.write_u32(*self as u32);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{DEFAULT_PROTOCOL, ReceivedMessage, WriteMessage};
    use std::error::Error;

    #[tokio::test]
    async fn test_read_void_001() -> Result<(), Box<dyn Error>> {
        let mut msg = ReceivedMessage::new(&[0, 1, 2, 3, 4], DEFAULT_PROTOCOL);
        let _: () = msg.read_value()?;
        Ok(())
    }

    #[tokio::test]
    async fn test_write_void_001() -> Result<(), Box<dyn Error>> {
        let mut msg = WriteMessage::new(DEFAULT_PROTOCOL);
        msg.write_value(())?;
        Ok(())
    }

    #[tokio::test]
    async fn test_read_u8_001() -> Result<(), Box<dyn Error>> {
        let mut msg = ReceivedMessage::new(&[0, 1, 2, 3, 4], DEFAULT_PROTOCOL);
        assert_eq!(msg.read_value::<u8>()?, 0);
        assert_eq!(msg.read_value::<u8>()?, 1);
        Ok(())
    }

    #[tokio::test]
    async fn test_write_u8_001() -> Result<(), Box<dyn Error>> {
        let mut msg = WriteMessage::new(DEFAULT_PROTOCOL);
        msg.write_u8(37);
        msg.write_value(37)?;
        Ok(())
    }

    #[tokio::test]
    async fn test_read_u16_001() -> Result<(), Box<dyn Error>> {
        let mut msg = ReceivedMessage::new(&[0, 1, 2, 3, 4], DEFAULT_PROTOCOL);
        assert_eq!(msg.read_value::<u16>()?, u16::from_le_bytes([0, 1]));
        assert_eq!(msg.read_value::<u16>()?, u16::from_le_bytes([2, 3]));
        assert!(msg.read_value::<u16>().is_err()); // not enough data
        Ok(())
    }

    #[tokio::test]
    async fn test_write_u16_001() -> Result<(), Box<dyn Error>> {
        let mut msg = WriteMessage::new(DEFAULT_PROTOCOL);
        msg.write_u16(37);
        msg.write_value(37_u16)?;
        Ok(())
    }

    #[tokio::test]
    async fn test_read_u32_001() -> Result<(), Box<dyn Error>> {
        let mut msg = ReceivedMessage::new(&[0, 1, 2, 3, 4], DEFAULT_PROTOCOL);
        assert_eq!(msg.read_value::<u32>()?, u32::from_le_bytes([0, 1, 2, 3]));
        assert!(msg.read_value::<u32>().is_err()); // not enough data
        Ok(())
    }

    #[tokio::test]
    async fn test_write_u32_001() -> Result<(), Box<dyn Error>> {
        let mut msg = WriteMessage::new(DEFAULT_PROTOCOL);
        msg.write_u32(37);
        msg.write_value(37_u32)?;
        Ok(())
    }

    #[tokio::test]
    async fn test_read_i32_001() -> Result<(), Box<dyn Error>> {
        let mut msg = ReceivedMessage::new(&[0, 1, 2, 3, 4], DEFAULT_PROTOCOL);
        assert_eq!(
            msg.read_value::<u32>()? as i32,
            i32::from_le_bytes([0, 1, 2, 3])
        );
        assert!(msg.read_value::<i32>().is_err()); // not enough data
        Ok(())
    }

    #[tokio::test]
    async fn test_write_i32_001() -> Result<(), Box<dyn Error>> {
        let mut msg = WriteMessage::new(DEFAULT_PROTOCOL);
        msg.write_u32(37_i32 as u32);
        msg.write_value(37_i32)?;
        Ok(())
    }
}
