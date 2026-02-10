"""Now Playing bar widget — shows current track info, progress, volume."""

from textual.app import ComposeResult
from textual.containers import Horizontal, Vertical
from textual.reactive import reactive
from textual.widget import Widget
from textual.widgets import Label, ProgressBar, Static

from ..utils import format_duration


class NowPlaying(Widget):
    """Bottom bar showing the currently playing track and controls."""

    DEFAULT_CSS = """
    NowPlaying {
        dock: bottom;
        height: 5;
        background: $surface;
        border-top: solid $primary;
        padding: 0 1;
    }

    NowPlaying .np-row1 {
        height: 1;
        width: 1fr;
    }

    NowPlaying .np-row2 {
        height: 1;
        width: 1fr;
    }

    NowPlaying .np-row3 {
        height: 1;
        width: 1fr;
    }

    NowPlaying .np-title {
        width: 1fr;
        color: $text;
        text-style: bold;
    }

    NowPlaying .np-artist {
        width: auto;
        color: $text-muted;
    }

    NowPlaying .np-state {
        width: auto;
        min-width: 3;
        color: $success;
        text-style: bold;
    }

    NowPlaying .np-progress-text {
        width: auto;
        min-width: 16;
        color: $text-muted;
        text-align: right;
    }

    NowPlaying .np-volume {
        width: auto;
        min-width: 10;
        color: $accent;
    }

    NowPlaying .np-modes {
        width: auto;
        min-width: 20;
        color: $warning;
        text-align: right;
    }

    NowPlaying .np-server {
        width: auto;
        min-width: 10;
        color: $text-muted;
        text-align: right;
    }

    NowPlaying ProgressBar {
        width: 1fr;
        height: 1;
        padding: 0;
    }

    NowPlaying ProgressBar Bar {
        width: 1fr;
        height: 1;
        background: $surface-darken-2;
    }

    NowPlaying ProgressBar Bar > .bar--bar {
        color: $primary;
    }

    NowPlaying ProgressBar PercentageStatus {
        display: none;
    }

    NowPlaying ProgressBar ETAStatus {
        display: none;
    }
    """

    song_title: reactive[str] = reactive("No track playing")
    song_artist: reactive[str] = reactive("")
    song_album: reactive[str] = reactive("")
    position: reactive[float] = reactive(0.0)
    duration: reactive[float] = reactive(0.0)
    volume: reactive[int] = reactive(75)
    muted: reactive[bool] = reactive(False)
    state: reactive[str] = reactive("stopped")
    shuffle_on: reactive[bool] = reactive(False)
    repeat_mode: reactive[str] = reactive("Off")
    server_name: reactive[str] = reactive("")
    bitrate: reactive[int] = reactive(0)
    suffix: reactive[str] = reactive("")

    def compose(self) -> ComposeResult:
        with Horizontal(classes="np-row1"):
            yield Static("⏹", id="np-state", classes="np-state")
            yield Static("No track playing", id="np-title", classes="np-title")
            yield Static("", id="np-artist", classes="np-artist")
        with Horizontal(classes="np-row2"):
            yield ProgressBar(total=100, show_eta=False, show_percentage=False, id="np-progress")
            yield Static("0:00 / 0:00", id="np-time", classes="np-progress-text")
        with Horizontal(classes="np-row3"):
            yield Static("🔊 75%", id="np-volume", classes="np-volume")
            yield Static("", id="np-modes", classes="np-modes")
            yield Static("", id="np-server", classes="np-server")

    def on_mount(self) -> None:
        """Force refresh all displays after mount."""
        self.watch_volume(self.volume)
        self._update_modes()
        if self.server_name:
            self.watch_server_name(self.server_name)

    def _get_state_icon(self) -> str:
        icons = {"playing": "▶", "paused": "⏸", "stopped": "⏹"}
        return icons.get(self.state, "⏹")

    def watch_song_title(self, value: str) -> None:
        title_w = self.query_one("#np-title", Static)
        title_w.update(value)

    def watch_song_artist(self, value: str) -> None:
        artist_w = self.query_one("#np-artist", Static)
        if value:
            info = f" — {value}"
            if self.song_album:
                info += f" [{self.song_album}]"
            if self.bitrate:
                info += f"  {self.suffix.upper()} {self.bitrate}kbps"
            artist_w.update(info)
        else:
            artist_w.update("")

    def watch_position(self, value: float) -> None:
        pb = self.query_one("#np-progress", ProgressBar)
        if self.duration > 0:
            pb.progress = (value / self.duration) * 100
        else:
            pb.progress = 0
        time_w = self.query_one("#np-time", Static)
        time_w.update(
            f"{format_duration(int(value))} / {format_duration(int(self.duration))}"
        )

    def watch_state(self, value: str) -> None:
        state_w = self.query_one("#np-state", Static)
        state_w.update(self._get_state_icon())

    def watch_volume(self, value: int) -> None:
        vol_w = self.query_one("#np-volume", Static)
        if self.muted:
            vol_w.update(f"🔇 {value}%")
        else:
            icon = "🔊" if value > 50 else "🔉" if value > 0 else "🔈"
            vol_w.update(f"{icon} {value}%")

    def watch_muted(self, value: bool) -> None:
        self.watch_volume(self.volume)

    def watch_shuffle_on(self, value: bool) -> None:
        self._update_modes()

    def watch_repeat_mode(self, value: str) -> None:
        self._update_modes()

    def watch_server_name(self, value: str) -> None:
        server_w = self.query_one("#np-server", Static)
        if value:
            server_w.update(f"🖥 {value}")
        else:
            server_w.update("")

    def _update_modes(self) -> None:
        modes_w = self.query_one("#np-modes", Static)
        parts = []
        if self.shuffle_on:
            parts.append("🔀 Shuffle")
        repeat_icons = {"Off": "", "All": "🔁 Repeat", "One": "🔂 Repeat 1"}
        r = repeat_icons.get(self.repeat_mode, "")
        if r:
            parts.append(r)
        modes_w.update("  ".join(parts))

    def update_track(
        self,
        title: str,
        artist: str,
        album: str,
        duration: float,
        bitrate: int = 0,
        suffix: str = "",
    ):
        """Update the currently playing track info."""
        self.song_title = title
        self.song_artist = artist
        self.song_album = album
        self.duration = duration
        self.bitrate = bitrate
        self.suffix = suffix

    def update_position(self, pos: float, dur: float):
        """Update playback position."""
        self.position = pos
        if dur > 0:
            self.duration = dur

    def clear_track(self):
        """Clear the track display."""
        self.song_title = "No track playing"
        self.song_artist = ""
        self.song_album = ""
        self.position = 0.0
        self.duration = 0.0
        self.state = "stopped"
        self.bitrate = 0
        self.suffix = ""
