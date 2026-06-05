//! Error types for the crate.

use thiserror::Error;

/// Anything that can go wrong while building or sending a Mystic Light command.
#[derive(Debug, Error)]
pub enum Error {
    /// More colors were supplied than a single header can address.
    #[error("too many LEDs: {got} provided, max is {max}")]
    TooManyLeds {
        /// Number of colors the caller supplied.
        got: usize,
        /// Maximum the protocol allows ([`crate::MAX_LEDS`]).
        max: usize,
    },

    /// No matching Mystic Light HID device was found.
    #[error("Mystic Light device not found (vid {vid:#06x}, pid {pid:#06x})")]
    DeviceNotFound {
        /// USB vendor id that was searched for.
        vid: u16,
        /// USB product id that was searched for.
        pid: u16,
    },

    /// The underlying transport failed to deliver a report.
    #[error(transparent)]
    Transport(#[from] TransportError),
}

/// A failure in the byte-level transport (e.g. the HID layer).
///
/// Kept deliberately backend-agnostic so the core crate never has to name
/// `hidapi` in its public error type.
#[derive(Debug, Error)]
#[error("transport error: {0}")]
pub struct TransportError(pub String);

impl TransportError {
    /// Wrap any displayable error as a [`TransportError`].
    pub fn new(source: impl std::fmt::Display) -> Self {
        TransportError(source.to_string())
    }
}
