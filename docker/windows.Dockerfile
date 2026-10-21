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

WORKDIR /workspace
