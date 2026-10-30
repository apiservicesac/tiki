# Contributing to Tiki

Thanks for helping. This guide covers how to report problems, propose changes and send code.

## Report a bug or ask for a feature

Open an [issue](https://github.com/apiservicesac/tiki/issues/new/choose) with the bug report
or feature request form. For a bug, include the Tiki version (Advanced settings › General)
and, when it helps, the log at `%LOCALAPPDATA%\pe.apiservice.tiki\logs\tiki.log`.

Security problems are never reported in public issues: see [SECURITY.md](SECURITY.md).

## Set up

Every toolchain runs in Docker; nothing is installed on the host.

```bash
make images
make ui-install
make test
make lint
```

Windows-only code (the desktop shell, audio and video capture) is checked on GitHub Actions
when you open a pull request.

## Code rules

- Classes with a single responsibility (structs and traits in Rust); small files.
- Descriptive names; no one-to-three-letter identifiers.
- Constants only where needed, in their one home: `TikiSettings` (core), `DesktopConfig`
  (shell), `src/config/uiConfig.ts` and `src/styles/tokens.css` (interface).
- The simplest correct code: no unneeded parsing, indirection or dead code.
- Code and comments in English. Every visible text comes from `locales/es.json` and
  `locales/en.json`, built with templates, never by joining strings.
- Business rules live in the Rust core; the interface draws what the core decides.
- Every rule comes with a test.

## Commits and pull requests

- One logical change per commit, with a message that says what changed and why.
- Branch from `main`; keep `make test` and `make lint` green.
- Fill in the pull request template; screenshots help for anything visible.
- User-visible changes get an entry in `CHANGELOG.md` under `Unreleased`.

## License

By contributing you agree that your contribution is licensed under the [MIT License](LICENSE).

## Releases (maintainers)

- Versions follow Semantic Versioning; release candidates are tagged `vX.Y.Z-rc.N` and carry
  the suffix in the app version (`Cargo.toml`, `apps/desktop/package.json`,
  `apps/desktop/src-tauri/tauri.conf.json`), so installed copies see each one as an update.
- Releases are titled `Tiki vX.Y.Z`; their notes reuse the `CHANGELOG.md` entry.
- Pushing the tag builds and signs the installer on GitHub Actions. The release is published
  with the installer and its signature first, then `updates/latest.json` is regenerated with
  `make update-manifest` and pushed to `main`, where installed copies read it.
