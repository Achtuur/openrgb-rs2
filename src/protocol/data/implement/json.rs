use serde::{Serialize, de::DeserializeOwned};

use crate::{DeserFromBuf, RawString, ReceivedMessage, SerToBuf};

#[derive(Debug)]
pub struct Json<T>(pub T);

impl<T: DeserializeOwned + std::fmt::Debug> DeserFromBuf for Json<T> {
    fn deserialize(buf: &mut ReceivedMessage<'_>) -> crate::OpenRgbResult<Self> {
        let json_str = buf.read_borrowed::<RawString<'_>>()?.0;
        let inner: T = serde_json::from_str(json_str)?;
        Ok(Self(inner))
    }
}

impl<T: Serialize + std::fmt::Debug> SerToBuf for Json<T> {
    fn serialize(&self, buf: &mut crate::WriteMessage) -> crate::OpenRgbResult<()> {
        let mut bytes = serde_json::to_vec(&self.0)?;
        bytes.push(b'\0'); // json strings must end with a null byte
        buf.write_slice(&bytes);
        Ok(())
    }
}

#[macro_export]
#[doc(hidden)]
macro_rules! impl_bufserde_json {
    ($struct:ty) => {
        impl $crate::DeserFromBuf for $struct {
            fn deserialize(buf: &mut $crate::ReceivedMessage<'_>) -> $crate::OpenRgbResult<Self>
            where
                Self: Sized,
            {
                // profile_data is a json string
                let data = buf.read_value::<$crate::data::implement::Json<Self>>()?;
                Ok(data.0)
            }
        }

        impl $crate::SerToBuf for $struct {
            fn serialize(&self, buf: &mut $crate::WriteMessage) -> $crate::OpenRgbResult<()> {
                buf.write_value($crate::data::implement::Json(self))
            }
        }
    };
}
