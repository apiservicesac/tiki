# Changelog

All notable changes to Tiki are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and versions follow
[Semantic Versioning](https://semver.org/). Release candidates are tagged `vX.Y.Z-rc.N`.

## [Unreleased]

## [0.3.1-rc.4] - 2026-10-22

### Changed
- Reminders live in the Agenda: every reminder by day (Today, Tomorrow, then dates), with its
  time and how often it repeats; calendar meetings will join it in 0.4. The Tasks tab keeps
  only tasks: Pending and Delivered.

## [0.3.1-rc.3] - 2026-10-22

### Fixed
- The island cut its content when it grew without a redraw, for example when the AI filled
  the fields of the box or a reminder was chosen. The notch now follows its content's size,
  and the click area with it.

## [0.3.1-rc.2] - 2026-10-22

### Fixed
- Reminders had no place of their own: they were mixed with tasks in the Tasks tab. Tasks now
  has Pending, Reminders and Delivered; Reminders lists every reminder by time with how often
  it repeats ("Fri 23 · 11:00 · Every day"), and Pending shows only tasks.
- The receipt counts only tasks as pending, not reminders.

## [0.3.1-rc.1] - 2026-10-22

### Changed
- One box for everything (the + button and Ctrl+Alt+Space): write a task, a reminder or a
  question. With AI, Enter reads it and either fills the item's fields to review, or answers
  and proposes changes you apply with a click. Ask Tiki is no longer a separate panel.
- Tasks and reminders are different things. A reminder is only a moment and how often it
  repeats (every day, weekdays, weekly, monthly); when it rings you mark it done or snooze
  it (+10 min, +1 h, tomorrow). A task keeps its work fields.
- Four general statuses: To do, In progress, Waiting, Done ("Testing" becomes In progress).
- Waiting tasks keep who or what they wait for as free text, and Tiki reminds you to follow
  up after a few days.
- No lists to maintain: a project you name for the first time is created when saving (the
  picker offers "Create «name»"), and the AI may name one from your sentence.
- Today shows only today: overdue, due soon, today's reminders and follow-ups, with "Now: do
  X" on top and one summary card (the latest reminder with its buttons, or the AI brief).
  Tasks shows everything.
- Priority counts: urgent and important items come first and show "!!" or "!".
- The Tickets tab is hidden until a ticket connector arrives; the mascot says "TODO BIEN"
  instead of "200 OK".

## [0.3.0-rc.1] - 2026-10-22

### Added
- Standalone reminders: an item that is only a heads-up at a moment ("call mom at 5"). Quick
  capture and the edit form choose Task or Reminder; the AI tells them apart. Reminders list
  under "Reminders" in Today with their time, ring in the notch and are done once they ring.
- Tags across projects (#billing, #urgent): a field in quick capture and the edit form, read
  by the AI from "#" words, shown under each title and searchable in Tasks with "#".
- Project notes: context for Tiki (who is who, how they work) in the project panel. The AI
  reads them for briefs, answers and proposed changes.
- Ask Tiki can act: it proposes changes (create a task or reminder, set a reminder or due
  date, move a task) as cards; each one applies only with your click, or "Apply all".
- Tiki learns from your corrections: when you change what the AI filled in before saving,
  it remembers the correction and follows it for similar sentences.
- AI prompts built from harness pieces (persona, guardrails, memory, action guide and one
  skill per job), and an eval of real sentences (`make eval-ai`) to measure the AI before
  each release.

### Fixed
- The arrow of the project picker sits at the far right; the Tasks tab, project panel and
  receipt keep the same margin on both sides; the project panel kept its styles after the
  rename.

## [0.2.2-rc.2] - 2026-10-22

### Changed
- Tiki groups work by **project** instead of client: a project can be a client, a product,
  a team or personal. Existing clients become projects, with their tasks and colors.
- Quick capture is a sentence with fields under it: project (picker with search), due date,
  reminder and priority. With an AI key, Enter fills the fields from the sentence, written
  any way you like, and you review them before saving with a second Enter. Without a key,
  the sentence is the title and you pick the fields.
- The AI capture understands any way of saying a day ("the 25th", "in two days", "next
  week") and reminders with their time.

### Removed
- The word-rules reading of quick capture (fixed words for days, times, "avísame" and
  "+Name"): it misread sentences such as "avisa a Juan…" or "a las 9 personas". Projects are
  created from the project picker or the project panel.

## [0.2.2-rc.1] - 2026-10-21

### Added
- Tasks tab in the island: every task grouped by client, the most pressing first, with a
  search box (task or client), a client picker with its own search, and Pending or Delivered.
  It works the same with three clients or fifty.
- Delivered history: delivered tasks with their day ("today", "yesterday", a date); opening
  one lets you reopen it. Today ends with "Delivered today (n)", which leads there.
- Client panel in the island (from the client picker): create clients, rename them, pick one
  of sixteen colors, or remove them; their tasks stay, without client.
- Task reminders: a "Remind me" day and time when editing a task, and quick capture
  understands "avísame el viernes a las 9" or "remind me at 5pm" (by rules or with AI). At
  that time the notch grows with the reminder and chimes; clicking it opens the task.
- The brief as the receipt of the design: clicking the morning brief or the end-of-day close
  shows each section with its lines, Tiki's suggestion and the day's totals.

### Changed
- The client field when editing a task is a picker with search instead of a plain list.
- New clients get one of sixteen colors instead of six.

## [0.2.1-rc.6] - 2026-10-21

### Fixed
- The notch showing a reminder had square corners; it is rounded like the other states.

## [0.2.1-rc.5] - 2026-10-21

### Added
- Reminders appear in the notch: it grows a little, Tiki hops, and the title shows with a
  one-line preview. Resting on it or clicking opens the island with the full reminder above
  the Today list. Untouched, it folds after a few seconds and leaves a dot until opened.
- The island plays its own soft chime with each reminder, so alerts work even when Windows
  hides notification banners ("Do not disturb", full-screen apps).
- Reminder settings: Sound, and Also in Windows to keep sending Windows notifications.

### Removed
- The "Test notification" button: the notch alert replaces it.

## [0.2.1-rc.4] - 2026-10-21

### Fixed
- Moving the morning brief or the end-of-day close to a time still ahead today now sends it
  at that time, even if today's was already sent (for example, when Tiki started after 08:30).

### Added
- "Test notification" in the reminder settings shows the brief right away, without using up
  today's, to check that Windows lets Tiki notify.
- A diagnostics log (`%LOCALAPPDATA%\pe.apiservice.tiki\logs\tiki.log`) records when each
  brief is due and shown, saved reminder times and errors, to explain problems afterwards.

## [0.2.1-rc.3] - 2026-10-21

### Fixed
- Settings no longer scroll on the left and right edges: the narrow island shows them
  without the big mascot, which the notch already shows.

## [0.2.1-rc.2] - 2026-10-21

### Fixed
- The advanced settings window opened blank on Windows. It now loads, with the island's
  dark background from the first frame.

### Changed
- Opening the advanced settings folds the island back to its notch, so it never covers them.

## [0.2.1-rc.1] - 2026-10-21

### Added
- Advanced settings window, opened from Settings in the island and drawn in the island's
  style, with its own title bar and a side menu: General (start with Windows, version and
  updates), AI assistant, Plugins and sources, Mail and calendar, and Shortcuts. Plugins and
  accounts show what each brings and when it arrives.

### Changed
- Settings in the island keep only what belongs to it: position, language and reminders
  (notifications, focus mode, morning brief and end-of-day close).
- On/off settings are switches, and plugin kinds are colored, as in the design.
- Ask Tiki opens the AI section of the advanced settings when no key is set.
- Both windows stay in sync: a change in one applies to the other right away.

### Fixed
- The model list opened white with unreadable options; lists and time pickers now follow
  the dark theme, and selects use Tiki's own arrow instead of the system one.
- Time fields no longer show the system clock icon; clicking the field opens the picker.

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
