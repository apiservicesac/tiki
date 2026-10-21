# Short commands for the Docker workflow. Nothing is installed on the host.
export HOST_UID := $(shell id -u)
export HOST_GID := $(shell id -g)

COMPOSE := docker compose run --rm

.PHONY: help images test test-core test-ui lint lint-core lint-ui format ui-install ui-build windows-check windows-installer clean

help:
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

windows-installer:
	$(COMPOSE) windows npm run tauri build -- --runner cargo-xwin --target $(WINDOWS_TARGET)
	@echo "Installer: target/$(WINDOWS_TARGET)/release/bundle/nsis/"

clean:
	rm -rf target apps/desktop/dist
