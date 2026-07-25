use crate::{DeserFromBuf, PacketId, impl_enum_discriminant};

#[derive(Debug)]
pub(crate) enum AckStatusCode {
    Ok = 0,
    GenericError = 1,
    Unsupported = 2,
    NotAllowed = 3,
    InvalidId = 4,
    InvalidData = 5,
}

impl_enum_discriminant!(AckStatusCode,
    Ok: 0,
    GenericError: 1,
    Unsupported: 2,
    NotAllowed: 3,
    InvalidId: 4,
    InvalidData: 5
);

impl AckStatusCode {
    pub fn is_ok(&self) -> bool {
        matches!(self, Self::Ok)
    }
}

#[derive(Debug)]
pub(crate) struct Acknowledge {
    pub packet_id: PacketId,
    pub status_code: AckStatusCode,
}

impl DeserFromBuf for Acknowledge {
    fn deserialize(buf: &mut crate::ReceivedMessage<'_>) -> crate::OpenRgbResult<Self> {
        let packet_id = buf.read_value()?;
        let status_code = buf.read_value()?;
        Ok(Self {
            packet_id,
            status_code,
        })
    }
}
