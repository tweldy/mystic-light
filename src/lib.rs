//! Direct per-LED control of MSI Mystic Light addressable-RGB (JARGB) headers
//! over USB HID.
//!
//! Many MSI motherboards expose their addressable-RGB ("JARGB") fan/strip
//! headers only through the proprietary Mystic Light software. This crate
//! speaks the same HID feature-report protocol directly, so you can drive each
//! LED yourself from Rust — on Linux, no vendor software, no root (with a udev
//! rule).
//!
//! # Layers
//!
//! * [`protocol`] — the pure, hardware-free wire format. Build a 761-byte HID
//!   feature report from a slice of [`Rgb`] colors. Fully unit-tested.
//! * [`transport`] — the [`Transport`](transport::Transport) trait: anything
//!   that can accept a raw feature report. Ships with a
//!   [`MockTransport`](transport::MockTransport) for tests and dry runs.
//! * [`hid`] — the real [`HidTransport`](hid::HidTransport), backed by
//!   `hidapi` (enabled by the `hidapi` feature, on by default).
//! * [`device`] — [`MysticLight`], the high-level controller you'll usually
//!   reach for.
//!
//! # Quick start
//!
//! ```no_run
//! use mystic_light::{Header, MysticLight, Rgb};
//!
//! # fn main() -> Result<(), mystic_light::Error> {
//! let mut ml = MysticLight::open()?;            // first connected device
//! ml.fill(Header::Jargb3, Rgb::new(0, 40, 80), 60)?;
//! # Ok(())
//! # }
//! ```
//!
//! # Disclaimer
//!
//! This is an unofficial, community-built library based on reverse-engineering.
//! It is not affiliated with or endorsed by Micro-Star International (MSI).
//! Writing to hardware always carries some risk; use it at your own.
#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod color;
pub mod daemon;
pub mod device;
pub mod error;
pub mod protocol;
pub mod transport;

#[cfg(feature = "hidapi")]
pub mod hid;

pub use color::{ParseColorError, Rgb};
pub use device::MysticLight;
pub use error::{Error, TransportError};
pub use protocol::{Header, MAX_LEDS, PRODUCT_ID, REPORT_ID, REPORT_LEN, VENDOR_ID};

#[cfg(feature = "hidapi")]
pub use hid::HidTransport;
