//! The reverse-engineered MSI Mystic Light addressable-RGB wire format.
//!
//! Every function here is pure and hardware-free: it turns colors into the
//! exact bytes the device expects. That makes the tricky part — the framing —
//! trivially unit-testable without ever touching USB.
//!
//! # Wire format
//!
//! A single write is one 727-byte HID **feature report**. The report id is the
//! first byte (as the `hidapi` convention requires). Layout:
//!
//! | Offset | Value      | Meaning                                       |
//! |-------:|------------|-----------------------------------------------|
//! | 0      | `0x51`     | Report id                                     |
//! | 1      | `0x09`     | Fixed framing byte                            |
//! | 2      | `0x04`     | "ARGB" command type                           |
//! | 3      | `0..=2`    | Header index (`0`=JARGB1, `1`=JARGB2, `2`=JARGB3) |
//! | 4–5    | `0x00`     | Reserved                                      |
//! | 6      | `0..=240`  | LED count `n`                                 |
//! | 7..    | `n`×`RGB`  | Per-LED color, three bytes each, R,G,B order  |
//! | ..761  | `0x00`     | Zero padding                                  |
//!
//! Protocol attribution: the framing was reverse-engineered by the community;
//! see `docs/PROTOCOL.md` for references.

use crate::color::Rgb;
use crate::error::Error;

/// USB vendor id for the MSI Mystic Light HID interface.
pub const VENDOR_ID: u16 = 0x0db0;
/// USB product id for the MSI Mystic Light HID interface.
pub const PRODUCT_ID: u16 = 0x0076;

/// HID feature-report id used for ARGB writes.
pub const REPORT_ID: u8 = 0x51;
/// Total feature-report length, in bytes, including the leading report id.
///
/// Seven header bytes plus 720 bytes of color. This matches
/// `sizeof(FeaturePacket_PerLED_761)` in OpenRGB's driver for this device — the
/// "761" there names the *device family*, not this packet, which is why an
/// earlier reading of the format put this at 761.
pub const REPORT_LEN: usize = 727;
/// Maximum number of LEDs addressable on a single header.
///
/// This is what one report can physically carry: the payload region runs from
/// offset 7 to the end of the [`REPORT_LEN`]-byte report, three bytes per LED.
/// It comes out at exactly 240, independently matching `GetMaxDirectLeds()` in
/// OpenRGB's driver for this device — which is the confirmation that both this
/// constant and [`REPORT_LEN`] are now right.
///
/// The previous value of 60 was an inherited guess. Counts above the real
/// maximum are not clamped by the hardware: the board validates this field and
/// drops the whole report, so an over-large count goes *dark* rather than
/// lighting the first 240.
pub const MAX_LEDS: usize = (REPORT_LEN - OFF_RGB) / 3;

// Fixed framing constants.
const FIXED_BYTE: u8 = 0x09; // offset 1
const ARGB_TYPE: u8 = 0x04; // offset 2

// Field offsets within the report buffer.
const OFF_REPORT_ID: usize = 0;
const OFF_FIXED: usize = 1;
const OFF_TYPE: usize = 2;
const OFF_HEADER: usize = 3;
const OFF_LED_COUNT: usize = 6;
const OFF_RGB: usize = 7;

/// One of the board's addressable-RGB (JARGB) headers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Header {
    /// First header (`JARGB1`), wire index `0`.
    Jargb1 = 0,
    /// Second header (`JARGB2`), wire index `1`.
    Jargb2 = 1,
    /// Third header (`JARGB3`), wire index `2`.
    Jargb3 = 2,
}

impl Header {
    /// Every header, in wire order.
    pub const ALL: [Header; 3] = [Header::Jargb1, Header::Jargb2, Header::Jargb3];

    /// The wire index (`0`, `1`, or `2`) for this header.
    pub const fn index(self) -> u8 {
        self as u8
    }

    /// Map a wire index back to a [`Header`], or `None` if out of range.
    pub const fn from_index(i: u8) -> Option<Header> {
        match i {
            0 => Some(Header::Jargb1),
            1 => Some(Header::Jargb2),
            2 => Some(Header::Jargb3),
            _ => None,
        }
    }
}

/// Build a complete feature report that sets the given per-LED `colors` on
/// `header`.
///
/// Returns [`Error::TooManyLeds`] if more than [`MAX_LEDS`] colors are given.
/// Any LEDs beyond `colors.len()` are left dark (zero-padded).
///
/// ```
/// # use mystic_light::{protocol::{build_report, REPORT_ID, REPORT_LEN}, Header, Rgb};
/// let report = build_report(Header::Jargb3, &[Rgb::RED, Rgb::GREEN]).unwrap();
/// assert_eq!(report.len(), REPORT_LEN);
/// assert_eq!(report[0], REPORT_ID);
/// assert_eq!(report[3], 2);            // JARGB3
/// assert_eq!(report[6], 2);            // two LEDs
/// assert_eq!(&report[7..13], &[255, 0, 0, 0, 255, 0]);
/// ```
pub fn build_report(header: Header, colors: &[Rgb]) -> Result<[u8; REPORT_LEN], Error> {
    if colors.len() > MAX_LEDS {
        return Err(Error::TooManyLeds {
            got: colors.len(),
            max: MAX_LEDS,
        });
    }

    let mut buf = [0u8; REPORT_LEN];
    buf[OFF_REPORT_ID] = REPORT_ID;
    buf[OFF_FIXED] = FIXED_BYTE;
    buf[OFF_TYPE] = ARGB_TYPE;
    buf[OFF_HEADER] = header.index();
    buf[OFF_LED_COUNT] = colors.len() as u8;

    for (i, c) in colors.iter().enumerate() {
        let o = OFF_RGB + i * 3;
        buf[o] = c.r;
        buf[o + 1] = c.g;
        buf[o + 2] = c.b;
    }

    Ok(buf)
}

/// Build a report that fills `count` LEDs of `header` with a single `color`.
///
/// `count` is clamped to [`MAX_LEDS`], so this can never fail.
pub fn build_fill(header: Header, color: Rgb, count: usize) -> [u8; REPORT_LEN] {
    let count = count.min(MAX_LEDS);
    let colors = [color].repeat(count);
    // `count <= MAX_LEDS`, so this is infallible.
    build_report(header, &colors).expect("count is clamped to MAX_LEDS")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_has_correct_length_and_framing() {
        let r = build_report(Header::Jargb1, &[]).unwrap();
        assert_eq!(r.len(), REPORT_LEN);
        assert_eq!(r[OFF_REPORT_ID], REPORT_ID);
        assert_eq!(r[OFF_FIXED], 0x09);
        assert_eq!(r[OFF_TYPE], 0x04);
        assert_eq!(r[4], 0x00);
        assert_eq!(r[5], 0x00);
    }

    #[test]
    fn header_index_lands_at_offset_three() {
        assert_eq!(build_report(Header::Jargb1, &[]).unwrap()[OFF_HEADER], 0);
        assert_eq!(build_report(Header::Jargb2, &[]).unwrap()[OFF_HEADER], 1);
        assert_eq!(build_report(Header::Jargb3, &[]).unwrap()[OFF_HEADER], 2);
    }

    #[test]
    fn led_count_and_rgb_payload() {
        let colors = [Rgb::new(1, 2, 3), Rgb::new(4, 5, 6), Rgb::new(7, 8, 9)];
        let r = build_report(Header::Jargb2, &colors).unwrap();
        assert_eq!(r[OFF_LED_COUNT], 3);
        assert_eq!(&r[OFF_RGB..OFF_RGB + 9], &[1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }

    #[test]
    fn unused_tail_is_zero_padded() {
        let r = build_report(Header::Jargb1, &[Rgb::WHITE]).unwrap();
        // Everything past the single LED's three bytes must be zero.
        assert!(r[OFF_RGB + 3..].iter().all(|&b| b == 0));
    }

    #[test]
    fn rejects_more_than_max_leds() {
        let too_many = [Rgb::BLACK].repeat(MAX_LEDS + 1);
        let err = build_report(Header::Jargb1, &too_many).unwrap_err();
        assert!(matches!(
            err,
            Error::TooManyLeds { got, max } if got == MAX_LEDS + 1 && max == MAX_LEDS
        ));
    }

    #[test]
    fn exactly_max_leds_is_accepted() {
        let full = [Rgb::WHITE].repeat(MAX_LEDS);
        let r = build_report(Header::Jargb3, &full).unwrap();
        assert_eq!(r[OFF_LED_COUNT], MAX_LEDS as u8);
        // Last LED's bytes sit at the end of the payload region.
        let last = OFF_RGB + (MAX_LEDS - 1) * 3;
        assert_eq!(&r[last..last + 3], &[255, 255, 255]);
    }

    #[test]
    fn fill_clamps_count() {
        let r = build_fill(Header::Jargb1, Rgb::RED, 999);
        assert_eq!(r[OFF_LED_COUNT], MAX_LEDS as u8);
        assert_eq!(&r[OFF_RGB..OFF_RGB + 3], &[255, 0, 0]);
    }

    #[test]
    fn header_index_roundtrips() {
        for h in Header::ALL {
            assert_eq!(Header::from_index(h.index()), Some(h));
        }
        assert_eq!(Header::from_index(3), None);
    }
}
