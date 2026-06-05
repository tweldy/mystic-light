# mystic-light

[![CI](https://github.com/tweldy/mystic-light/actions/workflows/ci.yml/badge.svg)](https://github.com/tweldy/mystic-light/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/mystic-light.svg)](https://crates.io/crates/mystic-light)
[![docs.rs](https://img.shields.io/docsrs/mystic-light)](https://docs.rs/mystic-light)
[![license](https://img.shields.io/crates/l/mystic-light.svg)](#license)

Direct, per-LED control of **MSI Mystic Light** addressable-RGB (JARGB) headers
over USB HID — from Rust, on Linux, with no vendor software and no root.

Many MSI boards only expose their JARGB fan/strip headers through the
proprietary Mystic Light Windows app. This crate speaks the same HID
feature-report protocol directly, so every LED is yours to drive — set a static
color, or stream animation frames from the host.

> ⚠️ **Unofficial.** Built from community reverse-engineering; not affiliated
> with or endorsed by MSI. Talking to hardware carries risk — use at your own.

## Supported hardware

- **VID:PID** `0x0DB0:0x0076` (`MYSTIC LIGHT`)
- Verified on the **MSI MPG X870E** family. Other boards using the same HID
  interface very likely work — reports welcome. See
  [`docs/PROTOCOL.md`](docs/PROTOCOL.md) for the wire format.

## Install

CLI:

```sh
cargo install mystic-light
```

Library:

```toml
[dependencies]
mystic-light = "0.1"
```

Building the HID backend needs the udev headers on Linux:

```sh
sudo apt-get install libudev-dev pkg-config   # Debian/Ubuntu
```

### Permissions (Linux)

To reach `/dev/hidraw*` without `sudo`, install the bundled udev rule:

```sh
sudo cp udev/99-mystic-light.rules /etc/udev/rules.d/
sudo udevadm control --reload-rules && sudo udevadm trigger
```

(Make sure your user is in the rule's `GROUP`, e.g. `plugdev`.)

## CLI

```sh
mystic list                         # find connected devices
mystic fill 3 '#0a1a2a'             # solid color on JARGB3
mystic fill 3 ff3300 --count 24    # only the first 24 LEDs
mystic rainbow 3 --seconds 8       # animated demo
mystic off                          # all headers off
mystic off 3                        # just JARGB3
```

Headers are `1`/`2`/`3` (or `jargb1`/`jargb2`/`jargb3`). Colors accept
`rrggbb`, `#rrggbb`, or short `#rgb`.

## Library

```rust
use mystic_light::{Header, MysticLight, Rgb};

fn main() -> Result<(), mystic_light::Error> {
    let mut ml = MysticLight::open()?;          // first connected device

    // Solid fill.
    ml.fill(Header::Jargb3, Rgb::new(0, 40, 80), 60)?;

    // Per-LED frame.
    let frame: Vec<Rgb> = (0..60)
        .map(|i| Rgb::from_hsv(i as f32 * 6.0, 1.0, 1.0))
        .collect();
    ml.set_leds(Header::Jargb3, &frame)?;

    ml.off(Header::Jargb3)?;
    Ok(())
}
```

See [`examples/breathe.rs`](examples/breathe.rs) for a small animation loop.

### Design

The crate is layered so the fiddly part — the byte framing — is testable with
zero hardware:

| Layer | What it is |
|-------|------------|
| `protocol` | Pure functions building the 761-byte report. No I/O. |
| `transport` | The `Transport` trait + a `MockTransport` recorder. |
| `hid` | The real `HidTransport` (feature `hidapi`). |
| `device` | `MysticLight`, the high-level controller. |

### Feature flags

| Feature | Default | Effect |
|---------|:-------:|--------|
| `hidapi` | ✅ | The real USB-HID backend. |
| `cli` | ✅ | The `mystic` binary (implies `hidapi`). |

For a pure, dependency-light protocol library (no `hidapi`, no C deps):

```toml
mystic-light = { version = "0.1", default-features = false }
```

## Roadmap

- Hardware effect modes (breathing/rainbow computed on the board)
- Save-to-flash and global brightness/speed registers
- A long-running daemon + thin socket API for other processes to drive

## Credits

Protocol reverse-engineering builds on `nmelo/msi-mystic-light-x870e` and
OpenRGB's MSI Mystic Light controllers. Thank you.

## License

Dual-licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option. Unless you state otherwise, any contribution you submit for
inclusion shall be dual-licensed as above, without additional terms.
