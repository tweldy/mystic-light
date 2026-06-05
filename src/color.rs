//! The [`Rgb`] color type and helpers.

use std::fmt;

/// An 8-bit-per-channel RGB color.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Rgb {
    /// Red channel, `0..=255`.
    pub r: u8,
    /// Green channel, `0..=255`.
    pub g: u8,
    /// Blue channel, `0..=255`.
    pub b: u8,
}

impl Rgb {
    /// Black / all channels off.
    pub const BLACK: Rgb = Rgb { r: 0, g: 0, b: 0 };
    /// Alias for [`Rgb::BLACK`].
    pub const OFF: Rgb = Rgb::BLACK;
    /// Full white.
    pub const WHITE: Rgb = Rgb {
        r: 255,
        g: 255,
        b: 255,
    };
    /// Full red.
    pub const RED: Rgb = Rgb { r: 255, g: 0, b: 0 };
    /// Full green.
    pub const GREEN: Rgb = Rgb { r: 0, g: 255, b: 0 };
    /// Full blue.
    pub const BLUE: Rgb = Rgb { r: 0, g: 0, b: 255 };

    /// Construct a color from its three channels.
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Parse a hex string: `"ff3300"`, `"#ff3300"`, or the short form `"#f30"`.
    ///
    /// ```
    /// # use mystic_light::Rgb;
    /// assert_eq!(Rgb::from_hex("#f30").unwrap(), Rgb::new(0xff, 0x33, 0x00));
    /// assert_eq!(Rgb::from_hex("0a1a2a").unwrap(), Rgb::new(10, 26, 42));
    /// ```
    pub fn from_hex(s: &str) -> Result<Self, ParseColorError> {
        let h = s.strip_prefix('#').unwrap_or(s);
        let parse = |slice: &str| u8::from_str_radix(slice, 16);
        match h.len() {
            6 => Ok(Rgb {
                r: parse(&h[0..2]).map_err(|_| ParseColorError)?,
                g: parse(&h[2..4]).map_err(|_| ParseColorError)?,
                b: parse(&h[4..6]).map_err(|_| ParseColorError)?,
            }),
            3 => {
                // Expand each nibble: "f30" -> "ff3300".
                let dup = |c: &str| u8::from_str_radix(&c.repeat(2), 16);
                Ok(Rgb {
                    r: dup(&h[0..1]).map_err(|_| ParseColorError)?,
                    g: dup(&h[1..2]).map_err(|_| ParseColorError)?,
                    b: dup(&h[2..3]).map_err(|_| ParseColorError)?,
                })
            }
            _ => Err(ParseColorError),
        }
    }

    /// Lowercase six-digit hex, without a leading `#`.
    pub fn to_hex(self) -> String {
        format!("{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }

    /// Linearly interpolate toward `other`. `t` is clamped to `0.0..=1.0`.
    pub fn lerp(self, other: Rgb, t: f32) -> Rgb {
        let t = t.clamp(0.0, 1.0);
        let mix = |a: u8, b: u8| (a as f32 + (b as f32 - a as f32) * t).round() as u8;
        Rgb {
            r: mix(self.r, other.r),
            g: mix(self.g, other.g),
            b: mix(self.b, other.b),
        }
    }

    /// Scale every channel by `factor` (clamped to `0.0..=1.0`) — a simple
    /// brightness dim.
    pub fn scale(self, factor: f32) -> Rgb {
        self.lerp(Rgb::BLACK, 1.0 - factor.clamp(0.0, 1.0))
    }

    /// Build a color from HSV. `h` is in degrees (wrapped to `0..360`); `s` and
    /// `v` are clamped to `0.0..=1.0`. Handy for rainbows.
    pub fn from_hsv(h: f32, s: f32, v: f32) -> Rgb {
        let h = h.rem_euclid(360.0);
        let s = s.clamp(0.0, 1.0);
        let v = v.clamp(0.0, 1.0);
        let c = v * s;
        let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
        let m = v - c;
        let (r1, g1, b1) = match h as u32 / 60 {
            0 => (c, x, 0.0),
            1 => (x, c, 0.0),
            2 => (0.0, c, x),
            3 => (0.0, x, c),
            4 => (x, 0.0, c),
            _ => (c, 0.0, x),
        };
        let ch = |f: f32| ((f + m) * 255.0).round() as u8;
        Rgb {
            r: ch(r1),
            g: ch(g1),
            b: ch(b1),
        }
    }
}

impl fmt::Display for Rgb {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{}", self.to_hex())
    }
}

impl From<(u8, u8, u8)> for Rgb {
    fn from((r, g, b): (u8, u8, u8)) -> Self {
        Rgb { r, g, b }
    }
}

/// Returned when a string can't be parsed as a color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParseColorError;

impl fmt::Display for ParseColorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid color (expected `rrggbb`, `#rrggbb`, or `#rgb`)")
    }
}

impl std::error::Error for ParseColorError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_six_digit_hex() {
        assert_eq!(Rgb::from_hex("0a1a2a").unwrap(), Rgb::new(10, 26, 42));
        assert_eq!(Rgb::from_hex("#FF0000").unwrap(), Rgb::RED);
    }

    #[test]
    fn parses_short_hex() {
        assert_eq!(Rgb::from_hex("#f30").unwrap(), Rgb::new(0xff, 0x33, 0x00));
        assert_eq!(Rgb::from_hex("fff").unwrap(), Rgb::WHITE);
    }

    #[test]
    fn rejects_garbage() {
        assert!(Rgb::from_hex("nope").is_err());
        assert!(Rgb::from_hex("12345").is_err());
        assert!(Rgb::from_hex("#zz0011").is_err());
    }

    #[test]
    fn hex_roundtrips() {
        let c = Rgb::new(1, 2, 3);
        assert_eq!(Rgb::from_hex(&c.to_hex()).unwrap(), c);
    }

    #[test]
    fn lerp_endpoints_and_midpoint() {
        let a = Rgb::new(0, 0, 0);
        let b = Rgb::new(100, 200, 50);
        assert_eq!(a.lerp(b, 0.0), a);
        assert_eq!(a.lerp(b, 1.0), b);
        assert_eq!(a.lerp(b, 0.5), Rgb::new(50, 100, 25));
        // t is clamped.
        assert_eq!(a.lerp(b, 5.0), b);
    }

    #[test]
    fn hsv_primaries() {
        assert_eq!(Rgb::from_hsv(0.0, 1.0, 1.0), Rgb::RED);
        assert_eq!(Rgb::from_hsv(120.0, 1.0, 1.0), Rgb::GREEN);
        assert_eq!(Rgb::from_hsv(240.0, 1.0, 1.0), Rgb::BLUE);
        assert_eq!(Rgb::from_hsv(0.0, 0.0, 1.0), Rgb::WHITE);
        assert_eq!(Rgb::from_hsv(0.0, 0.0, 0.0), Rgb::BLACK);
        // Hue wraps.
        assert_eq!(Rgb::from_hsv(360.0, 1.0, 1.0), Rgb::RED);
    }
}
