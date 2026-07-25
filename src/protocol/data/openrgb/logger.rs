#![allow(missing_docs, unused, reason = "Module might be in use later")]

use crate::{DeserFromBuf, OpenRgbLogLevel};

#[derive(Debug)]
pub struct LogEntry {
    log_level: OpenRgbLogLevel,
    /// Line number of log entry
    line: u32,
    /// Timestamp, relative to _start of `OpenRGB`_. NOT a unix timestamp
    timestamp: u32,
    filename: String,
    text: String,
}

impl DeserFromBuf for LogEntry {
    fn deserialize(buf: &mut crate::ReceivedMessage<'_>) -> crate::OpenRgbResult<Self> {
        let _data_size = buf.read_value::<u32>()?;
        let log_level = buf.read_value()?;
        let line = buf.read_value()?;
        let timestamp = buf.read_value()?;
        let filename = buf.read_value()?;
        let text = buf.read_value()?;
        Ok(Self {
            log_level,
            line,
            timestamp,
            filename,
            text,
        })
    }
}
