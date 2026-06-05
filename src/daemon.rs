//! A tiny line protocol for the `mysticd` daemon.
//!
//! The daemon owns the single HID handle and serves clients over a unix socket,
//! so other processes (e.g. an animation loop in another language) can drive the
//! LEDs without ever opening the device themselves — exactly one owner, no
//! contention.
//!
//! The protocol is newline-delimited text, one command per line. Each command
//! gets one reply line: `ok`, `pong`, or `err <message>`.
//!
//! ```text
//! ping                       -> pong
//! fill <h> <hex> [count]     -> ok      e.g. `fill 3 0000ff 60`
//! set  <h> <hex,hex,...>     -> ok      e.g. `set 3 ff0000,00ff00,0000ff`
//! off  [<h>]                 -> ok      e.g. `off` (all) or `off 3`
//! ```
//!
//! `<h>` is a header: `1`, `2`, or `3`. Colors are `rrggbb` / `#rrggbb` / `#rgb`.
//!
//! Parsing is pure and unit-tested here; the socket plumbing lives in the
//! `mysticd` binary.

use crate::color::Rgb;
use crate::device::MysticLight;
use crate::error::Error;
use crate::protocol::{Header, MAX_LEDS};
use crate::transport::Transport;

/// A parsed daemon command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    /// Liveness check; the daemon replies `pong`.
    Ping,
    /// Fill `count` LEDs of a header with one color.
    Fill {
        /// Target header.
        header: Header,
        /// Fill color.
        color: Rgb,
        /// LED count (clamped to [`MAX_LEDS`]).
        count: usize,
    },
    /// Set explicit per-LED colors on a header.
    Set {
        /// Target header.
        header: Header,
        /// One color per LED, nearest-first.
        colors: Vec<Rgb>,
    },
    /// Turn one header — or, if `None`, every header — off.
    Off {
        /// Header to clear, or `None` for all.
        header: Option<Header>,
    },
}

fn parse_header(tok: &str) -> Result<Header, String> {
    match tok {
        "1" => Ok(Header::Jargb1),
        "2" => Ok(Header::Jargb2),
        "3" => Ok(Header::Jargb3),
        other => Err(format!("bad header `{other}` (expected 1, 2, or 3)")),
    }
}

fn parse_color(tok: &str) -> Result<Rgb, String> {
    Rgb::from_hex(tok).map_err(|_| format!("bad color `{tok}`"))
}

/// Parse one protocol line into a [`Command`].
///
/// Leading/trailing whitespace is ignored. Returns a human-readable error
/// string suitable for sending straight back as `err <message>`.
pub fn parse_command(line: &str) -> Result<Command, String> {
    let mut parts = line.split_whitespace();
    let verb = parts.next().ok_or("empty command")?;
    match verb {
        "ping" => Ok(Command::Ping),
        "fill" => {
            let header = parse_header(parts.next().ok_or("fill: missing header")?)?;
            let color = parse_color(parts.next().ok_or("fill: missing color")?)?;
            let count = match parts.next() {
                Some(c) => c
                    .parse::<usize>()
                    .map_err(|_| format!("fill: bad count `{c}`"))?,
                None => MAX_LEDS,
            };
            Ok(Command::Fill {
                header,
                color,
                count,
            })
        }
        "set" => {
            let header = parse_header(parts.next().ok_or("set: missing header")?)?;
            let list = parts.next().ok_or("set: missing color list")?;
            let colors = list
                .split(',')
                .map(parse_color)
                .collect::<Result<Vec<_>, _>>()?;
            if colors.is_empty() {
                return Err("set: empty color list".into());
            }
            Ok(Command::Set { header, colors })
        }
        "off" => {
            let header = match parts.next() {
                Some(h) => Some(parse_header(h)?),
                None => None,
            };
            Ok(Command::Off { header })
        }
        other => Err(format!("unknown command `{other}`")),
    }
}

/// Apply a parsed command to a controller, returning the reply line to send.
pub fn apply<T: Transport>(cmd: Command, ml: &mut MysticLight<T>) -> Result<&'static str, Error> {
    match cmd {
        Command::Ping => Ok("pong"),
        Command::Fill {
            header,
            color,
            count,
        } => {
            ml.fill(header, color, count)?;
            Ok("ok")
        }
        Command::Set { header, colors } => {
            ml.set_leds(header, &colors)?;
            Ok("ok")
        }
        Command::Off { header } => {
            match header {
                Some(h) => ml.off(h)?,
                None => ml.all_off()?,
            }
            Ok("ok")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transport::MockTransport;

    #[test]
    fn parses_ping() {
        assert_eq!(parse_command("ping").unwrap(), Command::Ping);
        assert_eq!(parse_command("  ping \n").unwrap(), Command::Ping);
    }

    #[test]
    fn parses_fill_with_and_without_count() {
        assert_eq!(
            parse_command("fill 3 0000ff").unwrap(),
            Command::Fill {
                header: Header::Jargb3,
                color: Rgb::BLUE,
                count: MAX_LEDS
            }
        );
        assert_eq!(
            parse_command("fill 1 ff0000 24").unwrap(),
            Command::Fill {
                header: Header::Jargb1,
                color: Rgb::RED,
                count: 24
            }
        );
    }

    #[test]
    fn parses_set_list() {
        let cmd = parse_command("set 2 ff0000,00ff00,0000ff").unwrap();
        assert_eq!(
            cmd,
            Command::Set {
                header: Header::Jargb2,
                colors: vec![Rgb::RED, Rgb::GREEN, Rgb::BLUE]
            }
        );
    }

    #[test]
    fn parses_off_variants() {
        assert_eq!(parse_command("off").unwrap(), Command::Off { header: None });
        assert_eq!(
            parse_command("off 3").unwrap(),
            Command::Off {
                header: Some(Header::Jargb3)
            }
        );
    }

    #[test]
    fn rejects_bad_input() {
        assert!(parse_command("").is_err());
        assert!(parse_command("nope").is_err());
        assert!(parse_command("fill").is_err());
        assert!(parse_command("fill 9 ff0000").is_err());
        assert!(parse_command("fill 1 zzz").is_err());
        assert!(parse_command("set 1").is_err());
        assert!(parse_command("set 1 ff0000,zz").is_err());
        assert!(parse_command("off 7").is_err());
    }

    #[test]
    fn apply_runs_commands_through_a_mock() {
        let mut ml = MysticLight::new(MockTransport::new());
        assert_eq!(apply(Command::Ping, &mut ml).unwrap(), "pong");
        assert_eq!(ml.transport().count(), 0); // ping touches no hardware

        let cmd = parse_command("fill 3 0000ff 4").unwrap();
        assert_eq!(apply(cmd, &mut ml).unwrap(), "ok");
        let r = ml.transport().last().unwrap();
        assert_eq!(r[3], 2); // JARGB3
        assert_eq!(r[6], 4); // four LEDs
        assert_eq!(&r[7..10], &[0, 0, 255]);
    }
}
