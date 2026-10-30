# Toolchain to build, lint and test the Rust core and the recording crate (no desktop
# dependencies). cmake, make and g++ build the bundled libopus of the recording crate.
FROM rust:1.99-slim-bookworm

RUN rustup component add clippy rustfmt \
    && apt-get update \
    && apt-get install -y --no-install-recommends pkg-config cmake make g++ \
    && rm -rf /var/lib/apt/lists/*

# whisper-rs ships bindings generated for Linux/glibc: use them here instead of needing libclang.
ENV WHISPER_DONT_GENERATE_BINDINGS=1

WORKDIR /workspace
