FROM node:22-bookworm-slim AS frontend
WORKDIR /web
COPY web/package*.json ./
RUN npm ci --no-audit --no-fund
COPY web/ ./
RUN npm run build

FROM rust:1.93-bookworm AS backend
RUN apt-get update && apt-get install -y --no-install-recommends cmake clang libclang-dev libgl-dev libglu1-mesa-dev && rm -rf /var/lib/apt/lists/*
WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY .cargo .cargo
COPY crates crates
ENV CARGO_BUILD_JOBS=12
RUN --mount=type=cache,target=/usr/local/cargo/registry --mount=type=cache,target=/usr/local/cargo/git --mount=type=cache,target=/build/target cargo build --locked --release -p thetta-web && cp target/release/thetta-web /usr/local/bin/thetta-web

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates libstdc++6 libgl1 libglu1-mesa curl && rm -rf /var/lib/apt/lists/* && useradd --uid 1000 --create-home thetta && mkdir /data && chown thetta:thetta /data
COPY --from=backend /usr/local/bin/thetta-web /usr/local/bin/thetta-web
COPY --from=frontend --chown=thetta:thetta /web/dist /app/static
USER thetta
ENV THETTA_DATA_DIR=/data THETTA_STATIC_DIR=/app/static THETTA_BIND=0.0.0.0:3007
EXPOSE 3007
HEALTHCHECK --interval=30s --timeout=5s --start-period=15s CMD curl -fsS http://127.0.0.1:3007/api/health || exit 1
CMD ["thetta-web"]
