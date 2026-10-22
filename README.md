# Tiki

A desktop assistant for support work: your tickets, tasks, reminders and daily brief in an
island at the top of the screen, with Tiki the mascot showing how your day is going.

Platforms: Windows first; Linux, macOS, Android and iOS later. There is no web version.

## Layout

```text
crates/tiki-core/        Rust core, no UI: domain, SQLite storage, daily brief, mood,
                         projects, reminders, plugin contracts, i18n, workspace facade
apps/desktop/            Desktop app
  src/                   TypeScript interface (island, mascot, quick capture)
  src-tauri/             Tauri 2 shell: window, tray, global shortcuts, commands
  tests/                 Interface tests (Vitest)
locales/                 es.json and en.json, shared by Rust and TypeScript
docker/                  Toolchains: core (Rust), ui (Node), windows (cross-compile)
```

### Core modules (`crates/tiki-core/src`)

| Module | Responsibility |
| --- | --- |
| `domain` | Task, Project, statuses, priorities, origin of a task |
| `storage` | SQLite database, migrations and one repository per entity |
| `briefing` | Sorts open tasks into overdue / due soon / your turn / waiting, and narrates them |
| `mood` | Picks the mascot's mood from the day |
| `capture` | A confirmed quick capture: fields the AI filled from a sentence or the user picked (no word rules) |
| `projects` | Projects (a client, a product, a team or personal): workload, create, change, remove |
| `editing` | Task details and edits |
| `reminders` | When the briefs and task reminders are due, and the alerts they produce |
| `plugins` | Contracts for the two connection kinds: `TicketSource` and `ClientInstance` |
| `i18n` | Locale catalogs, translator, date formatting |
| `workspace` | `TikiWorkspace`, the single entry point every client uses |
| `settings` | `TikiSettings`, the only place with tunable values |

## AI

The AI library (`apps/desktop/src/ai`) wraps the official Anthropic and OpenAI SDKs; the
shell adds the key, which never reaches the interface. Prompts are built from harness pieces
(`ai/harness`): persona, guardrails, memory (project notes and the user's capture
corrections), the guide of actions Tiki may propose, and one skill per job (the single box,
which captures items or answers and proposes changes, and the brief). Tiki never changes
anything by itself: proposed actions wait for the user's click. There are no word rules:
without an AI key, the text is the title and the user picks the fields.

Tiki keeps no lists to maintain: projects are created the first time they are named, tags
are free, and who a task waits for is free text.

## Plugins

Every external system is a plugin offering one or both connection kinds:

- **Ticket source** (type 1): where tickets and project tasks come from.
- **Client instance** (type 2): a client's system that Tiki inspects read-only while
  investigating a ticket. The trait has no write methods.

Odoo will be the first plugin and implements both; Slack, Jira and GitHub can follow
without touching the rest of the app.

## Working with Docker

Nothing is installed on the host. Containers run as your user, caches live in `.cache/`.

```bash
make images            # build the toolchain images
make ui-install        # install the interface dependencies
make test              # Rust core + interface tests
make lint              # rustfmt, clippy (pedantic, warnings as errors), TypeScript types
make windows-check     # compile and lint the desktop shell for Windows
make windows-installer # build the Windows installer (.exe)
```

`make windows-installer` cross-compiles with `cargo-xwin`; the first run downloads the
Microsoft C runtime and Windows SDK headers (about 1.5 GB) and accepts Microsoft's license.
The `Windows installer` GitHub workflow builds the same installer natively on Windows.

## Releases

- Versions follow Semantic Versioning. The app version lives in `Cargo.toml`,
  `apps/desktop/package.json` and `apps/desktop/src-tauri/tauri.conf.json`.
- Release candidates are tagged `vX.Y.Z-rc.N` and published as GitHub pre-releases titled
  `Tiki vX.Y.Z-rc.N`; the final release is tagged `vX.Y.Z` and titled `Tiki vX.Y.Z`.
- Release candidates carry the suffix in the app version too (`0.2.0-rc.1`), so installed
  copies see each new candidate as an update.
- Every release adds its entry to `CHANGELOG.md`; release notes reuse that entry.
- Updates are signed. The private key lives outside the repository in
  `~/.tauri/tiki-updater.key` (back it up: without it, installed copies cannot be updated);
  the public key is in `tauri.conf.json`. Release steps:
  1. `ANTHROPIC_API_KEY=… make eval-ai` measures the AI on real sentences
     (`apps/desktop/tests/evals`); it should stay at 85% or more.
  2. `make windows-installer` builds and signs the installer.
  3. `make update-manifest VERSION=X.Y.Z-rc.N NOTES='…'` writes `updates/latest.json`.
  4. Publish the GitHub release with the installer, then commit and push `updates/latest.json`
     (installed copies read it from `main`).
- Everything in the repository is written in English: code, comments, commits, changelog
  and release notes. Only the interface texts in `locales/` are translated.

## Code rules

- Classes (structs + traits in Rust) with a single responsibility; small files.
- Descriptive names; no one-to-three-letter identifiers.
- Constants only where needed and centralized (`TikiSettings`, `DesktopConfig`, `uiConfig.ts`,
  `mascotAppearance.ts`, `tokens.css`).
- Code and comments in English; every visible text comes from `locales/`.
