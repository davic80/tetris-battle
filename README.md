# Tetris Battle

2-player online Tetris. One player creates a room and shares a 6-character code; the other player enters it. Both boards are visible in real time. Cleared lines send garbage to your opponent.

## Stack

- **Game logic**: Rust compiled to WebAssembly (SRS rotations, Tetris Guideline scoring/garbage)
- **Server**: Rust + Axum (WebSocket relay, REST API, SQLite match history)
- **Frontend**: Vanilla JS + Canvas (no framework)
- **Deployment**: Docker multi-arch (`linux/amd64` + `linux/arm64/v8`) + Cloudflare Tunnel on Raspberry Pi

## Running locally

```bash
# 1. Build WASM
CARGO_HOME=/tmp/cargo-home \
  RUSTUP_HOME=~/.rustup \
  PATH=~/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH \
  wasm-pack build game-logic --target web --out-dir ../static/pkg

# 2. Build and run server
cd server
CARGO_HOME=/tmp/cargo-home \
  RUSTUP_HOME=~/.rustup \
  PATH=~/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH \
  cargo run --release

# 3. Open http://localhost:8642 in two browser tabs
```

## Deployment (Raspberry Pi)

1. Copy `.env.example` to `.env` and set `CLOUDFLARE_TUNNEL_TOKEN`
2. Run `docker compose up -d`

Images are built and pushed automatically to `ghcr.io/davic80/tetris-battle` on push to `main` or version tags.

## Controls

| Action | Desktop | Mobile |
|--------|---------|--------|
| Move left/right | Arrow keys | ◀ ▶ buttons |
| Soft drop | Arrow down | ▼ button |
| Hard drop | Space | ⬇⬇ button |
| Rotate CW | Arrow up / X | ↻ button |
| Rotate CCW | Z / Ctrl | ↺ button |

## Settings

- **Next pieces preview**: 0, 1, or 4 (dropdown in game)
- **Language**: ES / EN (flag buttons top-right)
