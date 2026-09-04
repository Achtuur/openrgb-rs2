mod acknowledge;
mod bus_info;
mod controller;
mod device_type;
mod led;
mod logger;
mod matrix;
mod misc;
mod mode;
mod plugin;
mod profile;
mod segment;
mod settings;
mod zone;

pub(crate) use acknowledge::*;
pub use bus_info::*;
pub use controller::*;
pub use device_type::*;
pub use led::*;
pub use logger::*;
pub use matrix::*;
pub use misc::*;
pub use mode::*;
pub use plugin::*;
pub use profile::*;
pub use segment::*;
pub use settings::*;
pub use zone::*;

pub(crate) const ENABLE_SERDE_LOGGING: bool = false;
/// Macro to log (de)serialisation. Enabled only when debugging.
#[macro_export]
#[doc(hidden)]
macro_rules! log_serde {
    ($($args:tt)*) => {
        if $crate::ENABLE_SERDE_LOGGING {
            tracing::trace!($($args)*)
        }
    }
}
