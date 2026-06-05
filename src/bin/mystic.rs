//! `mystic` — a small command-line front end for the `mystic-light` crate.

use std::thread::sleep;
use std::time::{Duration, Instant};

use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand};

use mystic_light::hid::{self, HidTransport};
use mystic_light::{Header, MysticLight, Rgb, MAX_LEDS};

/// Control MSI Mystic Light addressable-RGB (JARGB) headers over USB HID.
#[derive(Parser)]
#[command(name = "mystic", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// List connected Mystic Light HID devices.
    List,
    /// Fill a header with one solid color.
    Fill {
        /// Header: `1`/`2`/`3` or `jargb1`/`jargb2`/`jargb3`.
        #[arg(value_parser = parse_header)]
        header: Header,
        /// Color as `rrggbb`, `#rrggbb`, or `#rgb`.
        #[arg(value_parser = parse_color)]
        color: Rgb,
        /// How many LEDs on the strip.
        #[arg(long, default_value_t = MAX_LEDS)]
        count: usize,
    },
    /// Turn a header off (or every header, if none is given).
    Off {
        /// Header to clear; omit to clear all.
        #[arg(value_parser = parse_header)]
        header: Option<Header>,
    },
    /// Sweep a rainbow across a header (a quick "is it working?" demo).
    Rainbow {
        /// Header: `1`/`2`/`3` or `jargb1`/`jargb2`/`jargb3`.
        #[arg(value_parser = parse_header)]
        header: Header,
        /// How long to run, in seconds.
        #[arg(long, default_value_t = 10.0)]
        seconds: f64,
        /// How many LEDs on the strip.
        #[arg(long, default_value_t = MAX_LEDS)]
        count: usize,
    },
}

fn parse_header(s: &str) -> Result<Header, String> {
    match s.trim().to_ascii_lowercase().as_str() {
        "1" | "jargb1" => Ok(Header::Jargb1),
        "2" | "jargb2" => Ok(Header::Jargb2),
        "3" | "jargb3" => Ok(Header::Jargb3),
        other => Err(format!("unknown header `{other}` (expected 1, 2, or 3)")),
    }
}

fn parse_color(s: &str) -> Result<Rgb, String> {
    Rgb::from_hex(s).map_err(|e| e.to_string())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Command::List => list(),
        Command::Fill {
            header,
            color,
            count,
        } => {
            let mut ml = open()?;
            ml.fill(header, color, count)?;
            println!("Filled {header:?} with {color} ({count} LEDs).");
            Ok(())
        }
        Command::Off { header } => {
            let mut ml = open()?;
            match header {
                Some(h) => {
                    ml.off(h)?;
                    println!("{h:?} off.");
                }
                None => {
                    ml.all_off()?;
                    println!("All headers off.");
                }
            }
            Ok(())
        }
        Command::Rainbow {
            header,
            seconds,
            count,
        } => rainbow(header, seconds, count),
    }
}

fn open() -> Result<MysticLight<HidTransport>> {
    MysticLight::open().context(
        "could not open a Mystic Light device — is it plugged in, and do you have permission \
         to read /dev/hidraw* ? (see the README for the udev rule)",
    )
}

fn list() -> Result<()> {
    let devices = hid::list().context("failed to enumerate HID devices")?;
    if devices.is_empty() {
        return Err(anyhow!("no Mystic Light devices found"));
    }
    for d in devices {
        let product = d.product.as_deref().unwrap_or("MYSTIC LIGHT");
        let serial = d.serial.as_deref().unwrap_or("-");
        println!("{}  product={product:?}  serial={serial}", d.path);
    }
    Ok(())
}

fn rainbow(header: Header, seconds: f64, count: usize) -> Result<()> {
    let count = count.min(MAX_LEDS);
    let mut ml = open()?;
    let start = Instant::now();
    let frame = Duration::from_millis(33); // ~30 fps
    let total = Duration::from_secs_f64(seconds.max(0.0));

    println!("Rainbow on {header:?} for {seconds:.1}s — Ctrl-C to stop early.");
    while start.elapsed() < total {
        let phase = start.elapsed().as_secs_f32() * 90.0; // degrees/sec
        let colors: Vec<Rgb> = (0..count)
            .map(|i| {
                let hue = phase + (i as f32 / count as f32) * 360.0;
                Rgb::from_hsv(hue, 1.0, 1.0)
            })
            .collect();
        ml.set_leds(header, &colors)?;
        sleep(frame);
    }
    ml.off(header)?;
    Ok(())
}
