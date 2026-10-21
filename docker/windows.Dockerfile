# Cross-compiles the Tauri desktop app and its NSIS installer for Windows from Linux,
# following Tauri's documented cargo-xwin setup. The first build downloads the
# Microsoft C runtime and Windows SDK headers (accepting Microsoft's license) into `.cache/`.
FROM rust:1.99-bookworm

RUN apt-get update \
    && apt-get install -y --no-install-recommends clang lld llvm nsis \
    && rm -rf /var/lib/apt/lists/* \
    && rustup target add x86_64-pc-windows-msvc \
    && rustup component add clippy \
    && cargo install --locked cargo-xwin

COPY --from=node:24-bookworm-slim /usr/local/bin/node /usr/local/bin/node
COPY --from=node:24-bookworm-slim /usr/local/lib/node_modules /usr/local/lib/node_modules
RUN ln -s /usr/local/lib/node_modules/npm/bin/npm-cli.js /usr/local/bin/npm \
    && ln -s /usr/local/lib/node_modules/npm/bin/npx-cli.js /usr/local/bin/npx

# The Tauri CLI on Linux looks up the tray library with pkg-config even when the target is Windows.
RUN apt-get update \
    && apt-get install -y --no-install-recommends pkg-config libayatana-appindicator3-dev \
    && rm -rf /var/lib/apt/lists/*

# Debian ships NSIS 3.08, which lacks a header the Tauri installer script includes (added in 3.10).
ADD https://raw.githubusercontent.com/kichik/nsis/v310/Include/Win/RestartManager.nsh \
    /usr/share/nsis/Include/Win/RestartManager.nsh
RUN chmod 644 /usr/share/nsis/Include/Win/RestartManager.nsh

WORKDIR /workspace
