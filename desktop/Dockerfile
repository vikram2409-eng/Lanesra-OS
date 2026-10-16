# Builds the Lanesra OS Team Workspace server: the same React frontend as
# the desktop app, served over HTTP by the lanesra-server binary, backed by
# a SQLite database on a mounted volume. See the help article
# https://lanesraos.com/help/set-up-team-workspace and desktop/README.md
# "Team Workspace".
#
# Build (from the desktop/ directory):
#   docker build -t lanesra-os-server .
# Run:
#   docker run -p 8080:8080 -v lanesra-data:/data lanesra-os-server
#
# Versions are pinned to the repository's own pins (WP-0020):
#   NODE_VERSION must match desktop/.nvmrc, RUST_VERSION must match
#   desktop/rust-toolchain.toml. program/tools/check_toolchain_pins.py
#   fails CI if they drift.
ARG NODE_VERSION=22
ARG RUST_VERSION=1.97.0

# ---- Frontend build ----
FROM node:${NODE_VERSION}-bookworm-slim AS frontend-builder
WORKDIR /app
COPY package.json package-lock.json .nvmrc ./
RUN npm ci
COPY tsconfig.json vite.config.ts index.html ./
COPY src ./src
RUN npm run build

# ---- Server build ----
# The whole Cargo workspace is copied (manifests, lockfile, toolchain pin and
# member sources) so that workspace settings such as [workspace.lints] apply
# and the build uses Cargo.lock exactly (--locked). Only lanesra-server is
# compiled: src-tauri is present for the workspace manifest but is never
# built, so this image needs none of the GTK/WebKit libraries it would.
FROM rust:${RUST_VERSION}-slim-bookworm AS server-builder
RUN apt-get update && apt-get install -y --no-install-recommends build-essential && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./
COPY core ./core
COPY server ./server
COPY cli ./cli
COPY src-tauri/Cargo.toml src-tauri/build.rs ./src-tauri/
COPY src-tauri/src ./src-tauri/src
RUN cargo build --release --locked -p lanesra-server

# ---- Runtime ----
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl && rm -rf /var/lib/apt/lists/* \
    && useradd --create-home --uid 10001 lanesra
WORKDIR /app
COPY --from=server-builder /app/target/release/lanesra-server ./lanesra-server
COPY --from=frontend-builder /app/dist ./dist
RUN mkdir -p /data && chown -R lanesra:lanesra /data /app
USER lanesra

ENV LANESRA_DATA_DIR=/data
ENV LANESRA_FRONTEND_DIR=/app/dist
ENV HOST=0.0.0.0
ENV PORT=8080
EXPOSE 8080
VOLUME ["/data"]

HEALTHCHECK --interval=30s --timeout=5s --start-period=20s --retries=3 \
    CMD curl -fsS http://127.0.0.1:8080/api/health || exit 1

CMD ["./lanesra-server"]
