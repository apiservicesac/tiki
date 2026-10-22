# Short commands for the Docker workflow. Nothing is installed on the host.
export HOST_UID := $(shell id -u)
export HOST_GID := $(shell id -g)

COMPOSE := docker compose run --rm

.PHONY: help images test test-core test-ui lint lint-core lint-ui format ui-install ui-build windows-check windows-installer update-manifest eval-ai clean

help:
	@echo "make eval-ai            Measure the AI on real sentences with your key"
	@echo "make update-manifest    Write updates/latest.json after building the installer"
	@echo "make images             Build the Docker images"
	@echo "make test               Run every test (Rust core + TypeScript interface)"
	@echo "make lint               Check formatting, clippy and TypeScript types"
	@echo "make format             Format the Rust code"
	@echo "make ui-install         Install the interface dependencies"
	@echo "make ui-build           Build the interface into apps/desktop/dist"
	@echo "make windows-check      Compile and lint the desktop shell for Windows"
	@echo "make windows-installer  Build the Windows installer (.exe) with cross-compilation"

images:
	docker compose build

test: test-core test-ui

test-core:
	$(COMPOSE) core cargo test -p tiki-core

test-ui:
	$(COMPOSE) ui npm test

lint: lint-core lint-ui

lint-core:
	$(COMPOSE) core sh -c "cargo fmt -p tiki-core --check && cargo clippy -p tiki-core --all-targets -- -D warnings"

lint-ui:
	$(COMPOSE) ui npm run typecheck

format:
	$(COMPOSE) core cargo fmt --all

ui-install:
	$(COMPOSE) ui npm install

ui-build:
	$(COMPOSE) ui npm run build

WINDOWS_TARGET := x86_64-pc-windows-msvc

windows-check:
	$(COMPOSE) windows sh -c "npm run build && cargo xwin clippy -p tiki-desktop --target $(WINDOWS_TARGET) -- -D warnings"

UPDATER_KEY := $(HOME)/.tauri/tiki-updater.key
# Calendar sign-in registrations (Azure and Google Cloud), kept out of the repository:
# TIKI_MICROSOFT_CLIENT_ID=…, TIKI_GOOGLE_CLIENT_ID=…, TIKI_GOOGLE_CLIENT_SECRET=…
OAUTH_ENV := $(HOME)/.tauri/tiki-oauth.env

windows-installer:
	@test -f $(UPDATER_KEY) || (echo "Missing $(UPDATER_KEY): the update signing key" && exit 1)
	@test -f $(OAUTH_ENV) || echo "Note: no $(OAUTH_ENV); this build cannot connect calendars"
	set -a; [ -f $(OAUTH_ENV) ] && . $(OAUTH_ENV); set +a; \
	TAURI_SIGNING_PRIVATE_KEY="$$(cat $(UPDATER_KEY))" TAURI_SIGNING_PRIVATE_KEY_PASSWORD="" \
		$(COMPOSE) windows npm run tauri build -- --runner cargo-xwin --target $(WINDOWS_TARGET)
	@echo "Installer: target/$(WINDOWS_TARGET)/release/bundle/nsis/"

# After `make windows-installer`: writes updates/latest.json for installed copies to find.
update-manifest:
	@test -n "$(VERSION)" || (echo "Usage: make update-manifest VERSION=0.2.0-rc.1 NOTES='...'" && exit 1)
	$(COMPOSE) -w /workspace ui node tools/update-manifest.mjs "$(VERSION)" "$(NOTES)"

# Measures the AI on real sentences with your key (costs a few cents). Example:
#   ANTHROPIC_API_KEY=... make eval-ai      or      TIKI_EVAL_PROVIDER=openai OPENAI_API_KEY=... make eval-ai
eval-ai:
	$(COMPOSE) -e TIKI_EVAL_PROVIDER -e TIKI_EVAL_MODEL -e ANTHROPIC_API_KEY -e OPENAI_API_KEY ui npm run eval

clean:
	rm -rf target apps/desktop/dist
