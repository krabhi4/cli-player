# 🎵 CLI Music Player for Navidrome

A terminal-based music player that connects to Navidrome instances and plays music locally through your server's speakers.

## Features

- **Multi-server support** — Connect to multiple Navidrome instances, switch on the fly
- **Full library browsing** — Artists, Albums, Songs, Playlists, Genres
- **Live search** — Search across your entire library
- **Playback controls** — Play, Pause, Next, Prev, Seek, Volume
- **Queue management** — Add, remove, reorder, save as playlist
- **Shuffle & Repeat** — Shuffle mode, Repeat All / Repeat One
- **18-band Equalizer** — 10 built-in presets + custom presets
- **Scrobbling** — Automatic play count reporting
- **Star/Favourite** — Star songs from the player
- **Beautiful TUI** — Tokyo Night-themed interface with Textual

## Requirements

- Debian/Ubuntu (amd64 or arm64)
- Python 3.11+
- mpv / libmpv (`sudo apt install mpv libmpv-dev`)
- A running Navidrome instance

## Installation

### Option 1: Install via .deb package (recommended)

```bash
# Build the .deb
./build-deb.sh

# Install it
sudo dpkg -i cli-music-player_1.0.0_amd64.deb

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

### Option 2: Run from source

```bash
# Clone and enter directory
git clone <repo-url>
cd cli-music-player

# Create venv and install
python3 -m venv venv
source venv/bin/activate
pip install -e .

# Run
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
| `c` | Clear queue |
| `/` | Search |
| `e` | Toggle Equalizer |
| `f` | Star/Unstar current song |
| `S` | Server Manager |
| `1-5` | Switch tabs (Albums/Artists/Songs/Playlists/Genres) |
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
