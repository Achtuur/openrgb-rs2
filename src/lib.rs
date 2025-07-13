//! Client library for [OpenRGB](https://gitlab.com/CalcProgrammer1/OpenRGB/-/blob/master/Documentation/OpenRGBSDK.md) SDK server.
//!
//! This client is async and requires a [tokio](https://tokio.rs) runtime to run.
//!
//! # Example
//!
//! ```no_run
//! use openrgb2::{OpenRgbResult, OpenRgbClient};
//!
//! #[tokio::main]
//! async fn main() -> OpenRgbResult<()> {
//!     // connect to default server at localhost
//!     let client = OpenRgbClient::connect().await?;
//!     let controllers = client.get_all_controllers().await?;
//!     controllers.init().await?;
//!     Ok(())
//! }
//! ```
//!
//! This crates provides the `OpenRgbClient` and `Controller` structs.
//! The client is used to connect to the OpenRGB server and retrieve `Controller`s,
//! after which RGB control can be done using the `Controller`.
//!
//! See [examples](https://github.com/Achtuur/openrgb-rs2/tree/master/examples), and [`OpenRgbClient`] for client API.

#![warn(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

#[doc(inline)]
pub use {
    client::*,
    data::*,
    error::{OpenRgbError, OpenRgbResult},
};

pub(crate) use protocol::*;

mod client;
mod error;
pub(crate) mod protocol;
