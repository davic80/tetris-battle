# ── Stage 1: Build WASM ────────────────────────────────────────────────────────
FROM rust:1.87-slim AS wasm-builder

RUN apt-get update && apt-get install -y \
    curl \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Install wasm-pack
RUN curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh

# Add wasm target
RUN rustup target add wasm32-unknown-unknown

WORKDIR /build
COPY game-logic/ ./game-logic/

RUN wasm-pack build game-logic --target web --out-dir /build/static/pkg

# ── Stage 2: Build Server ──────────────────────────────────────────────────────
FROM rust:1.87-slim AS server-builder

RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build
COPY server/ ./server/

RUN cargo build --release --manifest-path server/Cargo.toml

# ── Stage 3: Final Image ───────────────────────────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy server binary
COPY --from=server-builder /build/server/target/release/server ./server

# Copy static files
COPY static/ ./static/

# Copy compiled WASM into static/pkg
COPY --from=wasm-builder /build/static/pkg ./static/pkg

# Create data directory for SQLite
RUN mkdir -p /app/data

ENV PORT=8642
ENV DB_PATH=/app/data/tetris.db
ENV RUST_LOG=info

EXPOSE 8642

CMD ["./server"]
