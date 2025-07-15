use crate::{OpenRgbError, OpenRgbResult};

/// Deserialize an object from a byte buffer.
pub(crate) trait DeserFromBuf {
    fn deserialize(buf: &mut ReceivedMessage<'_>) -> OpenRgbResult<Self>
    where
        Self: Sized;
}

pub(crate) struct ReceivedMessage<'a> {
    protocol_version: u32,
    buf: &'a [u8],
    idx: usize,
}

impl std::fmt::Display for ReceivedMessage<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Received (protocol: {}, offset: {}): {:?}",
            self.protocol_version,
            self.idx,
            self.available_buf()
        )
    }
}

impl<'a> ReceivedMessage<'a> {
    pub fn new(buf: &'a [u8], protocol_version: u32) -> Self {
        Self {
            protocol_version,
            buf,
            idx: 0,
        }
    }

    pub fn protocol_version(&self) -> u32 {
        self.protocol_version
    }

    fn available_buf(&self) -> &[u8] {
        &self.buf[self.idx..]
    }

    #[inline]
    pub fn read_u8(&mut self) -> OpenRgbResult<u8> {
        let b = self.available_buf();
        if b.len() < size_of::<u8>() {
            return Err(OpenRgbError::ProtocolError(
                "Not enough bytes to read u8".to_string(),
            ));
        }
        let byte = self.buf[self.idx];
        self.idx += size_of::<u8>();
        Ok(byte)
    }

    pub fn read_u16(&mut self) -> OpenRgbResult<u16> {
        let b = self.available_buf();
        if b.len() < size_of::<u16>() {
            return Err(OpenRgbError::ProtocolError(
                "Not enough bytes to read u16".to_string(),
            ));
        }
        let value = u16::from_le_bytes([b[0], b[1]]);
        self.idx += size_of::<u16>();
        Ok(value)
    }

    pub fn read_u32(&mut self) -> OpenRgbResult<u32> {
        let b = self.available_buf();
        if b.len() < size_of::<u32>() {
            return Err(OpenRgbError::ProtocolError(
                "Not enough bytes to read u32".to_string(),
            ));
        }
        let value = u32::from_le_bytes([b[0], b[1], b[2], b[3]]);
        self.idx += size_of::<u32>();
        Ok(value)
    }

    pub fn read_value<T: DeserFromBuf>(&mut self) -> OpenRgbResult<T> {
        T::deserialize(self)
    }

    /// Reads the next `n` values as type `T` from the buffer.
    ///
    /// If there's a `[len, [..data]]` format, use `read_value::<Vec<T>>()` instead.
    pub fn read_n_values<T: DeserFromBuf>(&mut self, n: usize) -> OpenRgbResult<Vec<T>> {
        let mut values = Vec::with_capacity(n);
        for _ in 0..n {
            values.push(T::deserialize(self)?);
        }
        Ok(values)
    }
}

impl std::io::Read for ReceivedMessage<'_> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let available = &self.buf[self.idx..];
        let len = buf.len().min(available.len());
        buf[..len].copy_from_slice(&available[..len]);
        self.idx += len;
        Ok(len)
    }
}
