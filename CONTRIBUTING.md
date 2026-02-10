# Contributing to CLI Music Player

Thank you for your interest in contributing! This guide will help you get started.

## Getting Started

### Prerequisites

- Debian/Ubuntu (amd64 or arm64)
- Python 3.11+
- mpv / libmpv (`sudo apt install mpv libmpv-dev`)
- A running Navidrome instance (for testing)

### Development Setup

```bash
# Fork and clone the repo
git clone git@github.com:<your-username>/cli-player.git
cd cli-player

# Create a virtual environment
python3 -m venv venv
source venv/bin/activate

# Install in development mode
pip install -e .

# Run the app
python3 -m cli_music_player
```

## How to Contribute

### Reporting Bugs

- Use the [Bug Report](https://github.com/krabhi4/cli-player/issues/new?template=bug_report.md) issue template
- Include your OS, Python version, mpv version, and terminal emulator
- Describe what you expected vs. what happened
- Include screenshots if it's a UI issue

### Suggesting Features

- Use the [Feature Request](https://github.com/krabhi4/cli-player/issues/new?template=feature_request.md) issue template
- Explain the use case and why it would be useful

### Submitting Code

1. **Fork** the repository
2. **Create a branch** from `main`:
   ```bash
   git checkout -b feature/your-feature-name
   ```
3. **Make your changes** — keep commits focused and descriptive
4. **Test your changes** — make sure the app starts and your feature works
5. **Push** to your fork:
   ```bash
   git push origin feature/your-feature-name
   ```
6. **Open a Pull Request** against `main`

### Pull Request Guidelines

- PRs require **1 approval** before merging
- Keep PRs focused — one feature or fix per PR
- Update the README if you add new features or keybindings
- Don't bump the version number — maintainers will handle that
- Stale reviews are automatically dismissed when new commits are pushed

## Project Structure

```
src/cli_music_player/
├── __init__.py          # Version
├── __main__.py          # CLI entry point
├── app.py               # Main Textual app — orchestrates everything
├── config.py            # Config management, encrypted credentials
├── equalizer.py         # 18-band EQ with presets
├── player.py            # mpv-based audio playback engine
├── queue.py             # Queue, shuffle, repeat logic
├── subsonic.py          # Navidrome/Subsonic API client
├── utils.py             # Formatting helpers
├── styles/
│   └── app.tcss         # Textual CSS theme (Tokyo Night)
└── widgets/
    ├── browser.py       # Library browser (albums, artists, songs, etc.)
    ├── equalizer.py     # EQ widget
    ├── help.py          # Help/keybindings modal
    ├── now_playing.py   # Now Playing bar + controls + seekbar
    ├── queue_view.py    # Queue panel
    ├── search.py        # Search modal
    └── server_mgr.py   # Server manager modal
```

## Code Style

- Python code follows standard PEP 8
- Use type hints where practical
- Textual widgets go in `src/cli_music_player/widgets/`
- CSS goes in `src/cli_music_player/styles/app.tcss` for app-level styles, or in widget `DEFAULT_CSS` for widget-specific styles
- **Important:** App-level CSS (`app.tcss`) has higher priority than widget `DEFAULT_CSS`. If you add a new widget type, make sure global styles in `app.tcss` don't conflict

## Testing

Currently the project uses manual testing and Textual's headless `run_test()` for basic smoke tests:

```bash
source venv/bin/activate
python3 -c "
import asyncio
from cli_music_player.app import MusicPlayerApp

async def test():
    app = MusicPlayerApp()
    async with app.run_test(size=(120, 40)) as pilot:
        print('App renders OK')

asyncio.run(test())
"
```

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
