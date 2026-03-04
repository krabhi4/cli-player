# CLI Music Player for Navidrome

[![GitHub release](https://img.shields.io/github/v/release/krabhi4/cli-player)](https://github.com/krabhi4/cli-player/releases)
[![Rust](https://img.shields.io/badge/Rust-000000?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Binary Size](https://img.shields.io/badge/binary-~9MB-blue)](https://github.com/krabhi4/cli-player/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](CONTRIBUTING.md)
[![Tests](https://img.shields.io/badge/tests-293_passing-brightgreen)](https://github.com/krabhi4/cli-player/actions)

A terminal-based music player that connects to Navidrome instances and plays music locally through your server's speakers.

## Features

- **Multi-server support** — Connect to multiple Navidrome instances, switch on the fly
- **Full library browsing** — Artists, Albums, Songs, Playlists, Genres, Starred/Favourites
- **Live search** — Search across your entire library
- **Playback controls** — Play, Pause, Next, Prev, Seek, Volume
- **Queue management** — Add, remove (`d`/`Delete`), reorder (`Shift+Up/Down`), save as playlist (`P`)
- **Shuffle & Repeat** — Shuffle mode, Repeat All / Repeat One
- **18-band Equalizer** — 10 built-in presets + custom presets, interactive band adjustment (click or keyboard)
- **Lyrics display** — Toggle lyrics panel with `l`
- **Artist Radio** — Discover similar songs with `R`
- **Album sorting** — Cycle sort modes with `o` (Newest/Random/Frequent/Recent/Starred/A-Z)
- **Back navigation** — `Escape`/`Backspace` to go back when drilling into albums/artists/genres
- **Scrobbling** — Automatic play count reporting
- **Star/Favourite** — Star songs from the player
- **Beautiful TUI** — Tokyo Night-themed interface built with ratatui
- **Single binary** — No runtime dependencies, instant startup

## Requirements

- Linux (amd64) or macOS (x86_64 / Apple Silicon)
- ALSA development libraries on Linux (`sudo apt install libasound2-dev`)
- A running Navidrome instance

## Installation

### Option 1: Download from Releases (recommended)

```bash
# Download the latest release from GitHub
# https://github.com/krabhi4/cli-player/releases

# Linux — install the .deb package
sudo dpkg -i cli-music-player_3.0.0-1_amd64.deb

# Or use the binary directly (Linux/macOS)
chmod +x cli-music-player-*
./cli-music-player-x86_64-unknown-linux-gnu
```

The .deb package:
- Installs the binary to `/usr/bin/cli-music-player`
- Declares system dependency on `libasound2`

To uninstall:
```bash
sudo dpkg -r cli-music-player
```

### Option 2: Build from source

```bash
git clone https://github.com/krabhi4/cli-player.git
cd cli-player
cargo build --release
./target/release/cli-music-player
```

### Option 3: Install via cargo

```bash
cargo install --path .
cli-music-player
```

## Key Bindings

| Key | Action |
|---|---|
| `Space` | Play / Pause |
| `n` | Next track |
| `p` | Previous track |
| `→ / ←` | Seek ±5s |
| `Shift+→ / ←` | Seek ±30s |
| `+ / -` | Volume up/down |
| `m` | Mute toggle |
| `z` | Toggle Shuffle |
| `r` | Cycle Repeat (Off → All → One) |
| `a` | Add selected song to queue |
| `d / Delete` | Remove selected song from queue |
| `Shift+↑ / ↓` | Reorder queue items |
| `c` | Clear queue |
| `P` | Save queue as playlist |
| `/` | Search |
| `e` | Toggle Equalizer |
| `l` | Toggle Lyrics |
| `f` | Star/Unstar current song |
| `R` | Artist Radio (similar songs) |
| `o` | Cycle album sort mode |
| `S` | Server Manager |
| `1-6` | Switch tabs (Albums/Artists/Songs/Playlists/Genres/Starred) |
| `Esc / Backspace` | Go back (navigation history) |
| `?` | Help |
| `q` | Quit |

## First Run

On first launch, press `S` to open the Server Manager and add your Navidrome instance:
- **URL**: `http://localhost:4533` (or your Navidrome address)
- **Username/Password**: Your Navidrome credentials

Credentials are encrypted with AES-GCM and stored locally.

## Configuration

Config is stored at `~/.config/cli-music-player/config.json` and includes:
- Server connections (encrypted passwords)
- Volume level
- Shuffle/repeat preferences
- Custom EQ presets
- Audio device setting

## Testing

```bash
# Run all tests
cargo test

# Run a specific test file
cargo test --test queue_tests

# Run with output
cargo test -- --nocapture
```

### Test Coverage

- **Queue Manager**: 31 tests — Queue operations, shuffle, repeat
- **Subsonic API**: 16 tests — Data models, requests, authentication
- **Configuration**: 23 tests — Settings, encryption, persistence
- **Equalizer**: 27+ tests — 18-band EQ, dB conversion, presets, DSP
- **Player**: 27+ tests — State transitions, channel conversion, pipeline

**Total: 293 tests | Pass Rate: 100%**

## Contributing

Contributions are welcome! Please read the [Contributing Guide](CONTRIBUTING.md) before submitting a PR.

- Fork the repo, create a feature branch, open a PR
- PRs require 1 approval before merge
- All PRs must pass CI (cargo test, cargo clippy, cargo fmt)
- See [open issues](https://github.com/krabhi4/cli-player/issues) for things to work on

## License

This project is licensed under the [MIT License](LICENSE).
