use crate::OpenRgbResult;
use crate::protocol::{DeserFromBuf, ReceivedMessage};

/// A single LED.
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Led {
    /// LED name.
    name: String,

    /// LED value.
    ///
    /// This is some internal flag, basically of no use to us
    value: u32,
}

impl DeserFromBuf for Led {
    fn deserialize(buf: &mut ReceivedMessage<'_>) -> OpenRgbResult<Self>
    where
        Self: Sized,
    {
        Ok(Led {
            name: buf.read_value()?,
            value: buf.read_value()?,
        })
    }
}

// #[cfg(test)]
// mod tests {
//     use std::error::Error;

//     use tokio_test::io::Builder;

//     use crate::protocol::data::Led;
//     use crate::protocol::tests::setup;

//     #[tokio::test]
//     async fn test_read_001() -> Result<(), Box<dyn Error>> {
//         setup()?;

//         let mut stream = Builder::new()
//             .read(&5_u16.to_le_bytes())
//             .read(b"test\0")
//             .read(&45_u32.to_le_bytes())
//             .build();

//         assert_eq!(
//             stream.read_value::<Led>().await?,
//             Led {
//                 name: "test".to_string(),
//                 value: 45
//             }
//         );

//         Ok(())
//     }
// }
