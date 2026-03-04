# Contributing to CLI Music Player

Thank you for your interest in contributing! This guide will help you get started.

## Getting Started

### Prerequisites

- Rust toolchain (install via [rustup](https://rustup.rs/))
- ALSA development libraries on Linux (`sudo apt install libasound2-dev`)
- A running Navidrome instance (for testing)

### Development Setup

```bash
# Fork and clone the repo
git clone git@github.com:<your-username>/cli-player.git
cd cli-player

# Build
cargo build

# Run
cargo run

# Run tests
cargo test
```

## How to Contribute

### Reporting Bugs

- Use the [Bug Report](https://github.com/krabhi4/cli-player/issues/new?template=bug_report.md) issue template
- Include your OS, Rust version, and terminal emulator
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
4. **Test your changes**:
   ```bash
   cargo test
   cargo clippy
   cargo fmt --check
   ```
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
src/
├── main.rs                  # CLI entry point (clap)
├── lib.rs                   # Module declarations
├── utils.rs                 # Formatting helpers
├── audio/
│   ├── decoder.rs           # symphonia-based audio decoder
│   ├── equalizer_dsp.rs     # 18-band biquad EQ DSP
│   ├── output.rs            # cpal audio output
│   ├── pipeline.rs          # Audio thread orchestrator
│   └── resampler.rs         # Sample rate conversion (rubato)
├── player/
│   ├── engine.rs            # High-level playback engine
│   └── state.rs             # Playback state types
├── queue/
│   └── mod.rs               # Queue, shuffle, repeat logic
├── subsonic/
│   ├── client.rs            # Navidrome/Subsonic API client
│   ├── models.rs            # Song, Album, Artist, etc.
│   ├── auth.rs              # Token+salt authentication
│   └── error.rs             # API error types
├── config/
│   ├── mod.rs               # AppConfig load/save
│   ├── models.rs            # ServerConfig, EQPreset
│   ├── crypto.rs            # AES-GCM password encryption
│   └── presets.rs           # Default EQ presets
├── equalizer/
│   └── mod.rs               # High-level EQ preset management
└── tui/
    ├── app.rs               # Main ratatui app — orchestrates everything
    ├── event.rs             # Terminal event handling
    ├── theme.rs             # Tokyo Night color palette
    └── widgets/
        ├── browser.rs       # Library browser (albums, artists, songs, etc.)
        ├── equalizer.rs     # EQ widget
        ├── help.rs          # Help/keybindings modal
        ├── lyrics.rs        # Lyrics panel
        ├── now_playing.rs   # Now Playing bar + controls + seekbar
        ├── queue_view.rs    # Queue panel
        ├── search.rs        # Search modal
        └── server_mgr.rs   # Server manager modal
```

## Code Style

- Follow standard Rust conventions (`cargo fmt`, `cargo clippy`)
- Use `thiserror` for error types, `anyhow` for application errors
- TUI widgets go in `src/tui/widgets/`
- Keep audio processing on the dedicated audio thread (see `src/audio/pipeline.rs`)

## Testing

```bash
# Run all 293 tests
cargo test

# Run a specific test module
cargo test --test queue_tests
cargo test --test equalizer_tests

# Run with output
cargo test -- --nocapture
```

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
