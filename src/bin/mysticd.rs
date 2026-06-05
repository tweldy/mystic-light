//! `mysticd` — a small daemon that owns the Mystic Light HID device and serves
//! commands over a unix socket.
//!
//! One process holds the single HID handle; any number of clients (in any
//! language) drive the LEDs by writing newline-delimited commands. See
//! [`mystic_light::daemon`] for the protocol.
//!
//! ```text
//! mysticd [--socket PATH]
//! ```
//! Default socket: `$MYSTIC_SOCK`, else `/run/mystic-light/mystic.sock`.

use std::io::{BufRead, BufReader, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

use mystic_light::daemon::{apply, parse_command};
use mystic_light::{HidTransport, MysticLight};

const DEFAULT_SOCKET: &str = "/run/mystic-light/mystic.sock";

fn socket_path() -> PathBuf {
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        if a == "--socket" || a == "-s" {
            if let Some(p) = args.next() {
                return PathBuf::from(p);
            }
        }
    }
    std::env::var_os("MYSTIC_SOCK")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_SOCKET))
}

fn main() -> Result<()> {
    let path = socket_path();

    let mut ml = MysticLight::open().context("opening Mystic Light device")?;
    eprintln!("mysticd: opened Mystic Light device");

    let listener = bind(&path).with_context(|| format!("binding socket {}", path.display()))?;
    eprintln!("mysticd: listening on {}", path.display());

    // One client at a time (the LEDs are a single shared resource anyway).
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                if let Err(e) = serve(stream, &mut ml) {
                    eprintln!("mysticd: client ended: {e:#}");
                }
            }
            Err(e) => eprintln!("mysticd: accept error: {e}"),
        }
    }
    Ok(())
}

fn bind(path: &Path) -> Result<UnixListener> {
    // Clear any stale socket from an unclean shutdown.
    if path.exists() {
        std::fs::remove_file(path).ok();
    }
    let listener = UnixListener::bind(path)?;
    // Owner-only: this drives hardware; keep it private to the running user.
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
    Ok(listener)
}

fn serve(stream: UnixStream, ml: &mut MysticLight<HidTransport>) -> Result<()> {
    let mut writer = stream.try_clone()?;
    let reader = BufReader::new(stream);
    for line in reader.lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let reply = match parse_command(&line) {
            Ok(cmd) => match apply(cmd, ml) {
                Ok(r) => r.to_string(),
                Err(e) => format!("err {e}"),
            },
            Err(e) => format!("err {e}"),
        };
        writeln!(writer, "{reply}")?;
        writer.flush()?;
    }
    Ok(())
}
