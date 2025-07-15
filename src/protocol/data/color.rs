use rgb::RGB8;

use crate::OpenRgbResult;
use crate::protocol::{DeserFromBuf, ReceivedMessage, SerToBuf, WriteMessage};

/// RGB controller color, aliased to [rgb] crate's [RGB8] type.
///
/// See [Open SDK documentation](https://gitlab.com/CalcProgrammer1/OpenRGB/-/wikis/OpenRGB-SDK-Documentation) for more information.
pub type Color = RGB8;

impl DeserFromBuf for Color {
    fn deserialize(buf: &mut ReceivedMessage<'_>) -> OpenRgbResult<Self> {
        let r = buf.read_u8()?;
        let g = buf.read_u8()?;
        let b = buf.read_u8()?;
        let _ = buf.read_u8()?; // Skip the alpha channel
        Ok(Color { r, g, b })
    }
}

impl SerToBuf for Color {
    fn serialize(&self, buf: &mut WriteMessage) -> OpenRgbResult<()> {
        buf.write_u8(self.r);
        buf.write_u8(self.g);
        buf.write_u8(self.b);
        buf.write_u8(0u8); // Skip the alpha channel
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::protocol::data::Color;
    use crate::{OpenRgbResult, WriteMessage};

    #[tokio::test]
    async fn test_read_001() -> OpenRgbResult<()> {
        let mut buf = WriteMessage::new(0);
        buf.write_slice(&[37_u8, 54_u8, 126_u8, 0_u8]);
        let mut msg = buf.to_received_msg();

        assert_eq!(
            msg.read_value::<Color>()?,
            Color {
                r: 37,
                g: 54,
                b: 126
            }
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_write_001() -> OpenRgbResult<()> {
        let mut buf = WriteMessage::new(0);
        let c = Color {
            r: 37,
            g: 54,
            b: 126,
        };
        buf.write_value(&c)?;
        let mut msg = buf.to_received_msg();

        assert_eq!(&msg.read_n_values::<u8>(4)?, &[37_u8, 54_u8, 126_u8, 0_u8]);

        Ok(())
    }
}
