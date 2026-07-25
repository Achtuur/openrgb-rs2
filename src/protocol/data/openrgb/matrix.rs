use crate::{DeserFromBuf, RawSlice, ReceivedMessage, SerToBuf};

/// Should be followed by a
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct MatrixMapData {
    height: u32,
    width: u32,
    // data is always of length `self.width * self.height`
    data: Vec<u32>,
}
impl SerToBuf for Option<MatrixMapData> {
    fn serialize(&self, buf: &mut crate::WriteMessage) -> crate::OpenRgbResult<()> {
        match self {
            Self::Some(m) => {
                let len = (m.width * m.height) as u16;
                buf.write_value(len)?;
                if len > 0 {
                    buf.write_value(m)?;
                }
                Ok(())
            }
            // should be unreachable, treat it as matrix with dimension = 0
            Self::None => buf.write_value(0_u16),
        }
    }
}

impl DeserFromBuf for Option<MatrixMapData> {
    /// Deserializes [`zone_matrix_len` + `matrix_map_data`]
    fn deserialize(buf: &mut ReceivedMessage<'_>) -> crate::OpenRgbResult<Self> {
        let len = buf.read_value::<u16>()?;
        if len == 0 {
            return Ok(None);
        }
        buf.read_value::<MatrixMapData>().map(Some)
    }
}

impl SerToBuf for MatrixMapData {
    fn serialize(&self, buf: &mut crate::WriteMessage) -> crate::OpenRgbResult<()> {
        buf.push_value(self.height)?
            .push_value(self.width)?
            .push_value(RawSlice(&self.data))?;
        Ok(())
    }
}

impl DeserFromBuf for MatrixMapData {
    fn deserialize(buf: &mut ReceivedMessage<'_>) -> crate::OpenRgbResult<Self> {
        let height = buf.read_value()?;
        let width = buf.read_value()?;
        let data = buf.read_n_values(height as usize * width as usize)?;

        Ok(Self {
            height,
            width,
            data,
        })
    }
}
