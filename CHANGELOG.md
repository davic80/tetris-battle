# Changelog

All notable changes to this project will be documented in this file.
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-03-22

### Added
- Game options moved to the lobby (creator card): next-pieces count (0/1/4) and ghost piece toggle are now configured before creating a room and passed to the game via URL params
- Copy-code button: copies the 6-char room code to the clipboard with a success toast
- Share button: triggers the native Web Share API on mobile (share sheet); falls back to clipboard copy on desktop
- Ghost piece toggle: when disabled the drop-preview ghost is hidden during play

### Changed
- Removed the in-game settings bar (`<select>` for next pieces) from `game.html`; all settings are now pre-configured in the lobby by the room creator

## [0.1.3] - 2026-03-22

### Fixed
- WebSocket reconnection from `game.html`: when the browser redirects from the lobby to `/game`, the new WS connection is now recognised as a reconnect and receives `game_start` directly instead of being rejected as "room full"; this was the root cause of the "Esperando al rival" overlay never dismissing

## [0.1.2] - 2026-03-22

### Fixed
- Player slot assignment bug: `create_room` was pre-populating `player1`, causing the first WebSocket `join` to be assigned slot 2 and the second to be rejected as "room full"

## [0.1.1] - 2026-03-22

### Fixed
- Repository root path: files were nested under an extra `tetris-battle/` subfolder in the initial commit; repo structure is now correct

## [0.1.0] - 2026-03-22

### Added
- 2-player online Tetris with 6-character room code system
- Rust game logic compiled to WebAssembly (SRS wall kicks, 7-bag randomizer)
- Tetris Guideline scoring: single/double/triple/Tetris, back-to-back bonus, combo bonus
- Garbage attack table: 2-player line-sending with cancellation mechanic
- Axum WebSocket server: room creation, player slot assignment, broadcast relay
- REST API: `POST /api/rooms`, `GET /api/rooms/:code`
- SQLite match history persistence (room code, players, winner, timestamps)
- ES / EN language selector with flag buttons
- Mobile on-screen touch controls (8 buttons: move, rotate CW/CCW, soft drop, hard drop)
- Desktop keyboard controls: arrow keys, Z (rotate CCW), Space (hard drop), DAS/ARR
- Next-pieces preview panel (configurable: 0 / 1 / 4)
- Ghost piece indicator
- Garbage incoming bar with flash animation
- Combo / Tetris text display
- 3-2-1-GO countdown before match start
- Game over + winner overlay with play-again / home buttons
- Docker multi-arch image (`linux/amd64` + `linux/arm64/v8`) via GitHub Actions
- Cloudflare Tunnel deployment support (Raspberry Pi)
- `docker-compose.yml` with named SQLite volume

[0.1.3]: https://github.com/davic80/tetris-battle/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/davic80/tetris-battle/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/davic80/tetris-battle/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/davic80/tetris-battle/releases/tag/v0.1.0
