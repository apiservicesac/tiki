# Changelog

All notable changes to Tiki are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and versions follow
[Semantic Versioning](https://semver.org/). Release candidates are tagged `vX.Y.Z-rc.N`.

## [Unreleased]

### Fixed
- On the left and right edges the folded notch is wide enough for Tiki's face, which was
  cut against the screen edge.

## [0.6.0-rc.1] - 2026-10-30

### Added
- Recorded sessions. Record audio only, or audio and video of a screen or a window, from
  the header or Ctrl+Alt+R; when a call or a calendar meeting starts, the notch offers to
  record it, never without a click. While recording the notch shows a red dot and the
  time, and the island stays out of the video.
- Local transcription with Whisper: what you said and what the others said are told
  apart. The model downloads the first time you record.
- AI notes for each session: summary, key points, decisions and proposed tasks you create
  with one click; written automatically when an AI key is set, or on demand.
- A Sessions tab: a player synced to the transcript, search over what was said, export to
  Downloads, delete, retry a failed transcript and the disk space used. Media is kept 90
  days by default; transcripts and notes stay.
- A session that was recording when Tiki closed is recovered at the next start.
- With nothing pending, the resting notch shows CPU, memory, temperature and download
  speed, high figures in amber (Settings › Notch).
- Joining a Google Meet call opens it with the account the meeting comes from.
- Linked accounts show who they are: picture, name and email with the Google or Microsoft
  icon in Settings, and each meeting shows its account and platform (Google Meet, Teams,
  Zoom) in the Agenda, the notch alert and Windows notifications. Google accounts linked
  before this sign in again once to show their name and picture.
- Tiki for Windows is open source under the MIT License, with a new website at
  tiki.apiservicesac.com.

### Changed
- New app icons with Tiki's current face.
- Tiki's rounded body has a soft edge in the mood color, so it reads as a face and not
  only two eyes on a black notch.
- Switching between Today, Tasks and Agenda fades the new tab in instead of swapping it
  at once.
- The page after connecting a calendar shows Tiki's face, says whether it worked and tries
  to close itself.

### Fixed
- The end of long panels was cut at the bottom.
- An open project picker was cut when the card was short.
- When Google or Microsoft refuse a request, the log keeps their reason.
- A calendar sign-in waiting for the browser can be cancelled, and starting another one
  replaces it, so closing the browser no longer leaves Tiki waiting for five minutes.

## [0.5.1-rc.1] - 2026-10-30

### Added
- A trash: send an item there from its edit form, restore it from Tasks › Trash, delete
  it forever or empty the trash. Items left there are deleted after 30 days, and
  removing a project sends its items to the trash.
- Delivered tasks have a Reopen button.
- The notch alert has its buttons on the right: done, snooze 10 minutes or join.

### Changed
- Tiki is bigger: type, the notch, the island and the windows grow together, so it reads
  at a glance on a large screen; on small screens the window fits the free area.
- Every panel stays inside the window and scrolls when it does not fit, with Tiki's face
  in the header of every panel.
- The settings explain each option in one line and group them as alerts, quiet and
  briefs; the night hours are one row.
- The notch and Today say "3 to handle" instead of "3 today", waiting badges say
  "waiting 3 days", and the item shown as Now is not listed again below it.
- Projects are searched and created from one box.
- Quick capture and the edit form choose task or reminder with the same toggle.
- The plugins page shows the next plugin and names the later ones in one line.

### Fixed
- The edit form lost its buttons at the bottom, and Today with an alert lost its last rows.
- The deliver check only showed on hover; it now stays visible.
- On the side edges, the project picker covered the Tasks view toggle.
- Singular and plural in project counts and the brief totals.
- The alert's ripples ran into the notch edge and the text; badges and the buttons beside
  them had different heights.

## [0.5.0-rc.1] - 2026-10-29

### Added
- A new face for Tiki: a dark visor with two eyes whose shape and slow motion tell the mood
  at a glance, the same in the notch, the card and Settings.
- The notch glows softly in the mood's color; mood colors are the same everywhere.
- Alerts stand out: a wide aura in the alert's color, ripples from Tiki's face, the kind of
  alert on top (reminder, task, follow-up, meeting, morning, end of day) and a line that runs
  out until the notch folds back.
- Quiet at night (22:00–07:00 by default, set in the reminder settings): Tiki sleeps, and
  alerts still show in the notch but without sound or a Windows notification.
- Quiet during calls: when another app uses the microphone, the notch dims and alerts come
  without sound or a Windows notification.
- An activity history of every item (created, edited, moved, snoozed, rang, delivered,
  reopened), the base for the coming reports.
- Saving an item that breaks a rule (a reminder without a time) shows a message and keeps
  the form filled.

### Changed
- Tasks with an alarm appear in the Agenda next to reminders and meetings, and their alarm
  can be snoozed.
- Follow-up nudges have their own schedule and come back every 2 days while the item waits.
- Reminders no longer count as delivered work.
- Saved reminders without a time become tasks, so they show again in the Tasks tab.

### Fixed
- Reopening or snoozing a delivered item erased its delivery.
- An item moved out of "Waiting" could still ring its automatic follow-up.
- An alert's countdown restarted when the island redrew while it showed.
- Light theme colors for the "good" green and the urgency badges.

## [0.4.0-rc.1] - 2026-10-22

### Added
- Calendars: connect Microsoft 365 / Outlook and Google accounts from Settings › Accounts.
  Tiki signs in in the browser, only reads the calendar and keeps the sign-in in Windows
  Credential Manager. Meetings sync every 5 minutes.
- Meetings in the Agenda, next to reminders and ordered by time, with their place and a Join
  button for video calls.
- A notch alert 15 minutes before each meeting, with Join.
- "Now" counts the free time before the next meeting, and the morning brief lists the day's
  meetings.

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
- Ask Tiki: ask about your pending work ("what is pending with Acme?") or summarize
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
