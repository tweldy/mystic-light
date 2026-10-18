# MSI Mystic Light ARGB HID protocol

This documents the wire format `mystic-light` speaks. It was reverse-engineered
by the community; this crate is unofficial and not affiliated with MSI.

## Device

| | |
|---|---|
| USB vendor id | `0x0DB0` (Micro-Star International) |
| USB product id | `0x0076` (`MYSTIC LIGHT`) |
| Interface | HID, `hidraw` on Linux |

## Write

A single update is one **HID feature report** of **727 bytes**. The first byte
is the report id, per the usual HID convention (the `hidapi`
`send_feature_report` call expects it there).

| Offset | Value | Meaning |
|-------:|-------|---------|
| `0` | `0x51` | Report id |
| `1` | `0x09` | Fixed framing byte |
| `2` | `0x04` | Command type: ARGB |
| `3` | `0x00..0x02` | Header index: `0` = JARGB1, `1` = JARGB2, `2` = JARGB3 |
| `4` | `0x00` | Reserved |
| `5` | `0x00` | Reserved |
| `6` | `0x00..0xF0` | LED count `n` (0..240). The board validates this field and drops the whole report if it is too large, so an over-large count goes dark rather than lighting the first 240. |
| `7 .. 7 + 3n` | `R,G,B …` | Per-LED color, three bytes each, in R, G, B order |
| `7 + 3n .. 727` | `0x00` | Zero padding |

Each header is addressed independently: to update all three, send three
reports with byte `3` set to `0`, `1`, and `2` respectively.

Per-LED direct color is the entirety of what this crate implements today. It's
the most general capability — you can reproduce any static pattern or animate
frame-by-frame from the host.

## Not yet covered

The following are known to exist in the protocol but are **not** implemented
here:

- **Hardware effect modes** (breathing, rainbow, etc. computed on the board) —
  these would let effects continue without the host streaming frames.
- **Save-to-flash** — persisting a state across reboots.
- **Global brightness / speed** registers.

Contributions documenting these are very welcome.

## References

- `nmelo/msi-mystic-light-x870e` — the X870E reverse-engineering work this
  format is drawn from.
- OpenRGB's `MSIMysticLight185Controller` — prior art for the broader MSI
  Mystic Light HID family.
