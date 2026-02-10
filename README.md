# CLI Music Player for Navidrome

[![GitHub release](https://img.shields.io/github/v/release/krabhi4/cli-player)](https://github.com/krabhi4/cli-player/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![PRs Welcome](https://img.shields.io/badge/PRs-welcome-brightgreen.svg)](CONTRIBUTING.md)

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
- **Beautiful TUI** — Tokyo Night-themed interface with Textual

## Requirements

- Debian/Ubuntu (amd64 or arm64)
- Python 3.11+
- mpv / libmpv (`sudo apt install mpv libmpv-dev`)
- A running Navidrome instance

## Installation

### Option 1: Download the .deb from Releases (recommended)

```bash
# Download the latest release from GitHub
# https://github.com/krabhi4/cli-player/releases

# Install it
sudo dpkg -i cli-music-player_2.0.0_amd64.deb

# Run from anywhere
music-player
```

The .deb package:
- Installs the app to `/opt/cli-music-player/`
- Creates a virtualenv with all Python dependencies automatically
- Adds `music-player` command to your PATH
- Declares system dependencies (python3, mpv, libmpv-dev)

To uninstall:
```bash
sudo dpkg -r cli-music-player
```

### Option 2: Build .deb from source

```bash
git clone https://github.com/krabhi4/cli-player.git
cd cli-player
./build-deb.sh
sudo dpkg -i cli-music-player_2.0.0_amd64.deb
```

### Option 3: Run from source (development)

```bash
git clone https://github.com/krabhi4/cli-player.git
cd cli-player
python3 -m venv venv
source venv/bin/activate
pip install -e .
python3 -m cli_music_player
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

Credentials are encrypted with Fernet and stored locally.

## Configuration

Config is stored at `~/.config/cli-music-player/config.json` and includes:
- Server connections (encrypted passwords)
- Volume level
- Shuffle/repeat preferences
- Custom EQ presets
- Audio device setting

## What's New in 2.0.0

- **Starred/Favourites tab** — Browse your starred songs (tab `6`)
- **Queue management** — Remove (`d`/`Delete`) and reorder (`Shift+Up/Down`) queue items
- **Back navigation** — `Escape`/`Backspace` to navigate back through browsing history
- **Loading indicators** — Status bar shows loading feedback during library/search operations
- **Lyrics display** — Toggle lyrics panel with `l` (uses Subsonic getLyrics API)
- **Interactive equalizer** — Click or use arrow keys to adjust EQ bands, save custom presets
- **Artist Radio** — `R` to queue similar songs based on the currently playing track
- **Album sorting** — `o` to cycle through sort modes (Newest/Random/Frequent/Recent/Starred/A-Z)
- **Save queue as playlist** — `P` to save the current queue as a server-side playlist
- **Responsive queue** — Queue title truncation adapts to widget width
- **Better error feedback** — Errors surfaced to status bar instead of silently swallowed

## Contributing

Contributions are welcome! Please read the [Contributing Guide](CONTRIBUTING.md) before submitting a PR.

- Fork the repo, create a feature branch, open a PR
- PRs require 1 approval before merge
- See [open issues](https://github.com/krabhi4/cli-player/issues) for things to work on

## License

This project is licensed under the [MIT License](LICENSE).
