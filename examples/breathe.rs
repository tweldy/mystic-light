//! A gentle "breathing" pulse on JARGB3 — the library equivalent of
//! `mystic rainbow`, but showing the high-level API.
//!
//! Run with: `cargo run --example breathe`

use std::thread::sleep;
use std::time::{Duration, Instant};

use mystic_light::{Header, MysticLight, Rgb};

fn main() -> Result<(), mystic_light::Error> {
    let mut ml = MysticLight::open()?;
    let base = Rgb::new(0, 60, 120); // a calm blue
    let start = Instant::now();

    while start.elapsed() < Duration::from_secs(12) {
        // Sine-eased brightness in [0.1, 1.0].
        let t = start.elapsed().as_secs_f32();
        let wave = (t * std::f32::consts::PI / 2.0).sin() * 0.5 + 0.5;
        let level = 0.1 + 0.9 * wave;
        ml.fill(Header::Jargb3, base.scale(level), 60)?;
        sleep(Duration::from_millis(33));
    }

    ml.off(Header::Jargb3)?;
    Ok(())
}
