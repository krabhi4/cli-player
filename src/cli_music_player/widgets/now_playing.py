"""Now Playing bar widget — shows current track info, progress, controls."""

import contextlib

from textual.app import ComposeResult
from textual.containers import Horizontal
from textual.events import Click
from textual.message import Message
from textual.reactive import reactive
from textual.widget import Widget
from textual.widgets import Static

from ..utils import format_duration


class SeekBar(Widget):
    """A clickable seek/progress bar."""

    can_focus = True

    DEFAULT_CSS = """
    SeekBar {
        width: 1fr;
        height: 1;
        background: #1f2335;
        color: #7aa2f7;
    }
    SeekBar:hover {
        color: #7dcfff;
    }
    SeekBar:focus {
        color: #9ece6a;
    }
    """

    progress: reactive[float] = reactive(0.0)
    duration: reactive[float] = reactive(0.0)

    class Seeked(Message):
        """Emitted when the user clicks the seek bar."""

        def __init__(self, position: float) -> None:
            super().__init__()
            self.position = position

    def render(self) -> str:
        width = self.size.width
        if width <= 0:
            return ""
        ratio = min(self.progress / 100.0, 1.0)
        filled = int(width * ratio)
        empty = width - filled
        return "━" * filled + "╌" * empty

    def on_click(self, event: Click) -> None:
        """Handle click to seek."""
        if self.duration > 0 and self.size.width > 0:
            ratio = event.x / self.size.width
            position = ratio * self.duration
            self.post_message(self.Seeked(position))


class ControlBtn(Widget):
    """A compact clickable control button for the player bar.

    Unlike Textual's built-in Button (which has height:3 by default and
    can be overridden by app-level CSS), this is a simple 1-line-high
    widget that reliably handles click events.
    """

    can_focus = True

    DEFAULT_CSS = """
    ControlBtn {
        width: auto;
        min-width: 5;
        height: 1;
        background: transparent;
        color: #c0caf5;
        padding: 0;
        margin: 0;
        content-align: center middle;
    }
    ControlBtn:hover {
        background: #414868;
        color: #7aa2f7;
    }
    ControlBtn:focus {
        color: #9ece6a;
    }
    ControlBtn.-active {
        color: #9ece6a;
    }
    """

    class Pressed(Message):
        """Posted when the control button is clicked."""

        def __init__(self, action: str) -> None:
            super().__init__()
            self.action = action

    def __init__(
        self,
        label: str,
        action: str,
        *,
        id: str | None = None,
        classes: str | None = None,
    ) -> None:
        super().__init__(id=id, classes=classes)
        self._label = label
        self._action = action

    def render(self) -> str:
        return f" {self._label} "

    @property
    def label(self) -> str:
        return self._label

    @label.setter
    def label(self, value: str) -> None:
        self._label = value
        self.refresh()

    def on_click(self, event: Click) -> None:
        """Handle mouse click — post a Pressed message."""
        event.stop()
        self.post_message(self.Pressed(self._action))


class NowPlaying(Widget):
    """Bottom bar showing the currently playing track and controls."""

    DEFAULT_CSS = """
    NowPlaying {
        dock: bottom;
        height: 6;
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

    NowPlaying .np-row4 {
        height: 1;
        width: 1fr;
        align: center middle;
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
        width: 1fr;
        color: $warning;
        text-align: right;
    }

    NowPlaying .np-server {
        width: auto;
        min-width: 10;
        color: $text-muted;
        text-align: right;
    }

    NowPlaying .np-server:hover {
        color: #7aa2f7;
        text-style: underline;
    }

    NowPlaying .np-spacer {
        width: 1fr;
    }

    NowPlaying .ctrl-sep {
        width: 3;
        height: 1;
        color: #414868;
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
        # Row 1: Track info
        with Horizontal(classes="np-row1"):
            yield Static("⏹", id="np-state", classes="np-state")
            yield Static("No track playing", id="np-title", classes="np-title")
            yield Static("", id="np-artist", classes="np-artist")
        # Row 2: Seekbar + time
        with Horizontal(classes="np-row2"):
            yield SeekBar(id="np-seekbar")
            yield Static("0:00 / 0:00", id="np-time", classes="np-progress-text")
        # Row 3: Playback controls + volume
        with Horizontal(classes="np-row3"):
            yield ControlBtn("⏮", "prev_track", id="btn-prev")
            yield ControlBtn("⏯", "toggle_pause", id="btn-pause")
            yield ControlBtn("⏭", "next_track", id="btn-next")
            yield Static(" │ ", classes="ctrl-sep")
            yield ControlBtn("🔀", "toggle_shuffle", id="btn-shuffle")
            yield ControlBtn("🎛", "toggle_eq", id="btn-eq")
            yield ControlBtn("\u2139", "show_help", id="btn-info")
            yield Static("", classes="np-spacer")
            yield Static("🔊 75%", id="np-volume", classes="np-volume")
        # Row 4: Modes + server (server name is clickable)
        with Horizontal(classes="np-row4"):
            yield Static("", id="np-modes", classes="np-modes")
            yield ControlBtn("", "open_servers", id="np-server", classes="np-server")

    def on_mount(self) -> None:
        """Force refresh all displays after mount."""
        self.watch_volume(self.volume)
        self._update_modes()
        if self.server_name:
            self.watch_server_name(self.server_name)

    def _get_state_icon(self) -> str:
        icons = {"playing": "▶", "paused": "⏸", "stopped": "⏹"}
        return icons.get(self.state, "⏹")

    # ─── Watchers ────────────────────────────────────────

    def watch_song_title(self, value: str) -> None:
        with contextlib.suppress(Exception):
            self.query_one("#np-title", Static).update(value)

    def watch_song_artist(self, value: str) -> None:
        try:
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
        except Exception:
            pass

    def watch_position(self, value: float) -> None:
        try:
            seekbar = self.query_one("#np-seekbar", SeekBar)
            if self.duration > 0:
                seekbar.progress = (value / self.duration) * 100
                seekbar.duration = self.duration
            else:
                seekbar.progress = 0
            time_w = self.query_one("#np-time", Static)
            time_w.update(f"{format_duration(int(value))} / {format_duration(int(self.duration))}")
        except Exception:
            pass

    def watch_state(self, value: str) -> None:
        try:
            self.query_one("#np-state", Static).update(self._get_state_icon())
            btn = self.query_one("#btn-pause", ControlBtn)
            btn.label = "⏸" if value == "playing" else "▶"
        except Exception:
            pass

    def watch_volume(self, value: int) -> None:
        try:
            vol_w = self.query_one("#np-volume", Static)
            if self.muted:
                vol_w.update(f"🔇 {value}%")
            else:
                icon = "🔊" if value > 50 else "🔉" if value > 0 else "🔈"
                vol_w.update(f"{icon} {value}%")
        except Exception:
            pass

    def watch_muted(self, value: bool) -> None:
        self.watch_volume(self.volume)

    def watch_shuffle_on(self, value: bool) -> None:
        self._update_modes()
        try:
            btn = self.query_one("#btn-shuffle", ControlBtn)
            if value:
                btn.add_class("-active")
            else:
                btn.remove_class("-active")
        except Exception:
            pass

    def watch_repeat_mode(self, value: str) -> None:
        self._update_modes()

    def watch_server_name(self, value: str) -> None:
        try:
            server_w = self.query_one("#np-server", ControlBtn)
            server_w.label = f"🖥 {value}" if value else ""
        except Exception:
            pass

    def _update_modes(self) -> None:
        try:
            modes_w = self.query_one("#np-modes", Static)
            parts = []
            if self.shuffle_on:
                parts.append("🔀 Shuffle")
            repeat_icons = {"Off": "", "All": "🔁 Repeat", "One": "🔂 Repeat 1"}
            r = repeat_icons.get(self.repeat_mode, "")
            if r:
                parts.append(r)
            modes_w.update("  ".join(parts))
        except Exception:
            pass

    # ─── Button Press Handler ────────────────────────────

    def on_control_btn_pressed(self, event: ControlBtn.Pressed) -> None:
        """Handle control button presses — call app action directly."""
        action_method = getattr(self.app, f"action_{event.action}", None)
        if action_method:
            action_method()

    def on_seek_bar_seeked(self, event: SeekBar.Seeked) -> None:
        """Handle seek bar click."""
        self.app.action_seek_to(event.position)  # type: ignore[attr-defined]

    # ─── Public API ──────────────────────────────────────

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
