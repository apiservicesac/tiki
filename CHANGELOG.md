# Changelog

All notable changes to Tiki are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and versions follow
[Semantic Versioning](https://semver.org/). Release candidates are tagged `vX.Y.Z-rc.N`.

## [Unreleased]

## [0.1.1-rc.1] - 2026-10-21

### Changed
- The island opens on hover, without a click: the compact view first, then the whole
  island. It folds back when the pointer leaves, except while typing a task or changing
  settings.
- The notch animates its size and corners with a spring, like dooai, and its content
  fades in.

### Fixed
- Removed the square shadow that showed around the island.

## [0.1.0-rc.1] - 2026-10-21

### Added
- Island at the top of the screen with three states: hidden, compact and expanded.
- Tiki, the mascot, whose mood follows your day.
- Today tab grouped into Overdue, Due soon, Your turn and Waiting on others.
- Quick capture with `Ctrl+Alt+Space`; `+Name` creates a client.
- Check button to mark a task as delivered.
- Settings: island position (top, bottom, left, right) and language (Spanish, English).
- Tray icon and `Ctrl+Alt+N` to open or close the island.
- Windows installer cross-compiled from Linux with Docker.

[Unreleased]: https://github.com/apiservicesac/tiki/compare/v0.1.1-rc.1...HEAD
[0.1.1-rc.1]: https://github.com/apiservicesac/tiki/compare/v0.1.0-rc.1...v0.1.1-rc.1
[0.1.0-rc.1]: https://github.com/apiservicesac/tiki/releases/tag/v0.1.0-rc.1
