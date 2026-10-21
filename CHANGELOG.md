# Changelog

All notable changes to Tiki are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and versions follow
[Semantic Versioning](https://semver.org/). Release candidates are tagged `vX.Y.Z-rc.N`.

## [Unreleased]

## [0.2.0-rc.1] - 2026-10-21

### Added
- Edit a task by clicking it in the list: title, client, due date, priority, status and notes.
- Morning brief and end-of-day close as Windows notifications, at times you choose in
  Settings. Each arrives once a day; if the computer was off, the morning brief comes when
  it starts. The brief says what to start with and who to follow up after three days of
  waiting, and the island peeks out with the day's summary.
- Reminder settings: notifications on or off, focus mode (Tiki sleeps and stays quiet), the
  brief times, and start with Windows.
- Ask Tiki: ask about your pending work ("what is pending with Altamira?") or summarize
  your day. The AI brief also appears on the Today tab. Titles, clients and dates of open
  tasks are sent; notes never are.
- Signed automatic updates: Tiki checks GitHub every few hours, announces a new version and
  installs it from Settings › Updates. Updates are verified with Tiki's signing key.
- AI assistant settings: choose Anthropic or OpenAI, store the API key in the Windows
  credential store (never in Tiki's database or interface) and pick the model from a list.
  With a key the list comes from the provider; without one, a curated list is shown.
- Quick capture with AI: the first Enter shows what Tiki understood (task, client, due date,
  priority) and the second one saves it. Only the sentence and client names are sent.
  Without a key, for `+Name`, or if the provider fails, the sentence is saved with the
  built-in rules as before.

### Changed
- On the left and right edges the compact island shows the mood as a colored dot instead of
  a cramped "200 OK" label; the word stays on Tiki's tag and in the tooltip.

## [0.1.2-rc.3] - 2026-10-21

### Changed
- On the left and right edges the folded island shows Tiki with count bubbles (what is due
  and what is waiting) instead of rotated text, so it reads at a glance. The expanded
  island stays a narrow, tall panel; top and bottom stay horizontal.

## [0.1.2-rc.2] - 2026-10-21

### Changed
- Opening on hover now pauses on the compact notch before unfolding, and the expanded
  content appears a third of the way through the growth, so the two steps read clearly.
- On the left and right edges the island is laid out vertically: the folded notch stacks
  Tiki and a vertical summary, and the expanded island is a narrow, tall panel.

### Added
- The resting island shows "Tiki · 3 today · 2 waiting" when work is pending, and Tiki alone
  when the day is clear.

## [0.1.2-rc.1] - 2026-10-21

### Changed
- The island window keeps a fixed size and lets clicks through outside the island, so the
  notch animates with CSS only and never jumps while the window moves or resizes.
- Opening and closing use separate curves: a soft ease-out to open and a shorter one to
  close.
- Content cross-fades only when the island changes shape; tabs and panels switch instantly.

### Fixed
- Changing the language no longer closes the island; Settings stays open in the new language.
- Switching tabs no longer flashes an empty card.
- Text typed in quick capture is kept when the island refreshes.

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

[Unreleased]: https://github.com/apiservicesac/tiki/compare/v0.1.2-rc.3...HEAD
[0.1.2-rc.3]: https://github.com/apiservicesac/tiki/compare/v0.1.2-rc.2...v0.1.2-rc.3
[0.1.2-rc.2]: https://github.com/apiservicesac/tiki/compare/v0.1.2-rc.1...v0.1.2-rc.2
[0.1.2-rc.1]: https://github.com/apiservicesac/tiki/compare/v0.1.1-rc.1...v0.1.2-rc.1
[0.1.1-rc.1]: https://github.com/apiservicesac/tiki/compare/v0.1.0-rc.1...v0.1.1-rc.1
[0.1.0-rc.1]: https://github.com/apiservicesac/tiki/releases/tag/v0.1.0-rc.1
