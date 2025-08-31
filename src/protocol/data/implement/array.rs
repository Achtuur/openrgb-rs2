use std::mem::MaybeUninit;

use crate::{
    OpenRgbResult, SerToBuf, WriteMessage,
    protocol::{DeserFromBuf, ReceivedMessage},
};

impl<T: SerToBuf, const N: usize> SerToBuf for [T; N] {
    fn serialize(&self, buf: &mut WriteMessage) -> OpenRgbResult<()> {
        self.as_slice().serialize(buf)
    }
}

impl<T: DeserFromBuf, const N: usize> DeserFromBuf for [T; N] {
    fn deserialize(buf: &mut ReceivedMessage<'_>) -> OpenRgbResult<Self> {
        let mut arr = [const { MaybeUninit::<T>::uninit() }; N];

        for item in &mut arr {
            let d = T::deserialize(buf)?;
            item.write(d);
        }

        // the for loop either writes to every element of the array or returns an error
        unsafe { Ok(std::mem::transmute_copy(&arr)) }
    }
}

#[cfg(test)]
mod tests {
    use crate::DEFAULT_PROTOCOL;

    use super::*;

    #[test]
    fn test_read_array() {
        let mut message = ReceivedMessage::new(&[0, 1, 2, 3, 4, 5], DEFAULT_PROTOCOL);
        let arr: [u8; 3] = message.read_value().unwrap();
        assert_eq!(arr, [0, 1, 2]);
        let arr2: [u8; 3] = message.read_value().unwrap();
        assert_eq!(arr2, [3, 4, 5]);
        assert!(message.read_value::<[u8; 3]>().is_err());
    }

    #[tokio::test]
    async fn test_write_array() -> OpenRgbResult<()> {
        let mut msg = WriteMessage::new(DEFAULT_PROTOCOL);
        msg.write_value(&[42; 5])?;
        Ok(())
    }
}
