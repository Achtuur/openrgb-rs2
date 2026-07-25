use crate::OpenRgbResult;
use crate::protocol::{DeserFromBuf, ReceivedMessage, SerToBuf, WriteMessage};

macro_rules! impl_tuple {
    ($($idx:tt $t:tt),+) => {
        impl<$($t: DeserFromBuf),+> DeserFromBuf for ($($t,)+) {
            fn deserialize(buf: &mut ReceivedMessage<'_>) -> OpenRgbResult<Self> {
                Ok((
                    $(
                        buf.read_value::<$t>()?,
                    )+
                ))
            }
        }

        impl<$($t: SerToBuf),+> SerToBuf for ($($t,)+) {
            fn serialize(&self, buf: &mut WriteMessage) -> OpenRgbResult<()> {
                $(
                    buf.write_value(&self.$idx)?;
                )+
                Ok(())
            }
        }
    }
}

impl_tuple!(0 A);
impl_tuple!(0 A, 1 B);
impl_tuple!(0 A, 1 B, 2 C);
impl_tuple!(0 A, 1 B, 2 C, 3 D);
impl_tuple!(0 A, 1 B, 2 C, 3 D, 4 E);

#[cfg(test)]
mod tests {
    use std::error::Error;

    use crate::WriteMessage;
    use crate::protocol::data::DeviceType;

    #[tokio::test]
    async fn test_ser_deser_tuple() -> Result<(), Box<dyn Error>> {
        let mut buf = WriteMessage::new(crate::DEFAULT_PROTOCOL);
        let mut msg = buf
            .push_value(37_u8)?
            .push_value(1337_u32)?
            .push_value(-1337_i32)?
            .push_value(4_u32)?
            .to_received_msg();

        assert_eq!(
            msg.read_value::<(u8, u32, i32, DeviceType)>()?,
            (37, 1337, -1337, DeviceType::LEDStrip)
        );

        Ok(())
    }
}
