# Builds the Lanesra OS Team Workspace server: the same React frontend as
# the desktop app, served over HTTP by the lanesra-server binary, backed by
# a SQLite database on a mounted volume. Intended for a small team on one
# local-network host - see desktop/README.md "Team Workspace" section.
#
# Build (from the desktop/ directory):
#   docker build -t lanesra-os-server .
# Run:
#   docker run -p 8080:8080 -v lanesra-data:/data lanesra-os-server

# ---- Frontend build ----
FROM node:20-slim AS frontend-builder
WORKDIR /app
COPY package.json package-lock.json ./
RUN npm ci
COPY tsconfig.json vite.config.ts index.html ./
COPY src ./src
RUN npm run build

# ---- Server build ----
# Only core/ and server/ are copied in (not src-tauri/, which needs GTK/
# WebKit dev libraries this image has no reason to install) and no
# workspace Cargo.toml, so Cargo treats server/ as its own build root.
FROM rust:1-slim-bookworm AS server-builder
RUN apt-get update && apt-get install -y --no-install-recommends build-essential && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY core ./core
COPY server ./server
RUN cargo build --release --manifest-path server/Cargo.toml

# ---- Runtime ----
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/* \
    && useradd --create-home --uid 10001 lanesra
WORKDIR /app
COPY --from=server-builder /app/server/target/release/lanesra-server ./lanesra-server
COPY --from=frontend-builder /app/dist ./dist
RUN mkdir -p /data && chown -R lanesra:lanesra /data /app
USER lanesra

ENV LANESRA_DATA_DIR=/data
ENV LANESRA_FRONTEND_DIR=/app/dist
ENV HOST=0.0.0.0
ENV PORT=8080
EXPOSE 8080
VOLUME ["/data"]

CMD ["./lanesra-server"]
