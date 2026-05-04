#![allow(unused_imports)]

#[cfg(feature = "defmt")]
pub use defmt::{debug, error, info, trace, warn};

#[cfg(not(feature = "defmt"))]
pub use log::{debug, error, info, trace, warn};
