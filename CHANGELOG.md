# Changelog

All notable changes to this project are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project
adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0]

Initial release.

### Added
- Pure protocol layer (`protocol`) that builds the 761-byte ARGB feature
  report, with full unit-test coverage.
- `Transport` trait plus a hardware-free `MockTransport`.
- `HidTransport` backed by `hidapi` (behind the default `hidapi` feature).
- High-level `MysticLight` controller: `set_leds`, `fill`, `off`, `all_off`.
- `Rgb` color type with hex and HSV helpers.
- `mystic` CLI: `list`, `fill`, `off`, `rainbow`.
- `breathe` example, dual MIT/Apache-2.0 licensing, CI, and a udev rule.

[Unreleased]: https://github.com/tweldy/mystic-light/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/tweldy/mystic-light/releases/tag/v0.1.0
