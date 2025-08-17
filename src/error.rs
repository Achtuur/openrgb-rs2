use thiserror::Error;

/// Type alias for `Result<T, OpenRgbError>`
pub type OpenRgbResult<T> = std::result::Result<T, OpenRgbError>;

/// Errors that can occur while communicating with the `OpenRGB` server or creating commands.
#[derive(Error, Debug)]
pub enum OpenRgbError {
    /// Failed opening connection to `OpenRGB` server.
    #[error("Failed opening connection to OpenRGB server at {addr:?}")]
    ConnectionError {
        /// `OpenRGB` server address.
        addr: String,

        /// Source error.
        #[source]
        source: std::io::Error,
    },

    /// Communication failure with `OpenRGB` server.
    #[error("Failed exchanging data with OpenRGB server")]
    CommunicationError {
        /// Source error.
        #[source]
        #[from]
        source: std::io::Error,
    },

    /// Invalid encountered while communicating with `OpenRGB` server.
    #[error("Invalid data encountered while communicating with OpenRGB server: {0}")]
    ProtocolError(String),

    /// Server does not support operation.
    #[error(
        "{operation:?} is only supported since protocol version {min_protocol_version:?}, but version {current_protocol_version:?} is in use. Try upgrading the OpenRGB server."
    )]
    UnsupportedOperation {
        /// Operation name.
        operation: String,

        /// Protocol version in use by client.
        current_protocol_version: u32,

        /// Minimum required protocol version to use operation.
        min_protocol_version: u32,
    },

    /// Command was given invalid parameters
    #[error("Invalid command: {0}")]
    CommandError(String),
}
