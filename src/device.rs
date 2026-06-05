//! [`MysticLight`] — the high-level controller.

use crate::color::Rgb;
use crate::error::Error;
use crate::protocol::{self, Header, MAX_LEDS};
use crate::transport::Transport;

/// A high-level handle for driving Mystic Light ARGB headers.
///
/// Generic over its [`Transport`], so the same logic runs against real hardware
/// ([`crate::hid::HidTransport`]) or a [`crate::transport::MockTransport`] in
/// tests.
pub struct MysticLight<T: Transport> {
    transport: T,
}

impl<T: Transport> MysticLight<T> {
    /// Wrap an existing transport.
    pub fn new(transport: T) -> Self {
        Self { transport }
    }

    /// Set explicit per-LED colors on one header.
    ///
    /// `colors[0]` is the LED closest to the header. At most [`MAX_LEDS`]
    /// colors may be given; LEDs past `colors.len()` are turned off.
    pub fn set_leds(&mut self, header: Header, colors: &[Rgb]) -> Result<(), Error> {
        let report = protocol::build_report(header, colors)?;
        self.transport.send_feature_report(&report)?;
        Ok(())
    }

    /// Fill `count` LEDs of a header with a single color (`count` is clamped to
    /// [`MAX_LEDS`]).
    pub fn fill(&mut self, header: Header, color: Rgb, count: usize) -> Result<(), Error> {
        let report = protocol::build_fill(header, color, count);
        self.transport.send_feature_report(&report)?;
        Ok(())
    }

    /// Turn a single header off.
    pub fn off(&mut self, header: Header) -> Result<(), Error> {
        self.fill(header, Rgb::OFF, MAX_LEDS)
    }

    /// Turn every header off.
    pub fn all_off(&mut self) -> Result<(), Error> {
        for header in Header::ALL {
            self.off(header)?;
        }
        Ok(())
    }

    /// Borrow the underlying transport (e.g. to inspect a mock in tests).
    pub fn transport(&self) -> &T {
        &self.transport
    }

    /// Consume the controller and return its transport.
    pub fn into_transport(self) -> T {
        self.transport
    }
}

#[cfg(feature = "hidapi")]
impl MysticLight<crate::hid::HidTransport> {
    /// Open the first connected Mystic Light device.
    pub fn open() -> Result<Self, Error> {
        Ok(Self::new(crate::hid::HidTransport::open()?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::MockTransport;

    fn controller() -> MysticLight<MockTransport> {
        MysticLight::new(MockTransport::new())
    }

    #[test]
    fn fill_emits_one_report_with_expected_bytes() {
        let mut ml = controller();
        ml.fill(Header::Jargb3, Rgb::new(10, 20, 30), 2).unwrap();

        let t = ml.transport();
        assert_eq!(t.count(), 1);
        let r = t.last().unwrap();
        assert_eq!(r[0], protocol::REPORT_ID);
        assert_eq!(r[3], 2); // JARGB3
        assert_eq!(r[6], 2); // count clamped to requested 2
        assert_eq!(&r[7..13], &[10, 20, 30, 10, 20, 30]);
    }

    #[test]
    fn off_blacks_out_max_leds() {
        let mut ml = controller();
        ml.off(Header::Jargb1).unwrap();
        let r = ml.transport().last().unwrap().to_vec();
        assert_eq!(r[6], MAX_LEDS as u8);
        assert!(r[7..7 + MAX_LEDS * 3].iter().all(|&b| b == 0));
    }

    #[test]
    fn all_off_hits_every_header() {
        let mut ml = controller();
        ml.all_off().unwrap();
        let sent = &ml.transport().sent;
        assert_eq!(sent.len(), 3);
        let headers: Vec<u8> = sent.iter().map(|r| r[3]).collect();
        assert_eq!(headers, vec![0, 1, 2]);
    }

    #[test]
    fn set_leds_rejects_overflow() {
        let mut ml = controller();
        let too_many = [Rgb::WHITE].repeat(MAX_LEDS + 1);
        assert!(matches!(
            ml.set_leds(Header::Jargb1, &too_many),
            Err(Error::TooManyLeds { .. })
        ));
        // Nothing should have been sent on the error path.
        assert_eq!(ml.transport().count(), 0);
    }
}
