"""Lyrics display panel widget."""

from textual.app import ComposeResult
from textual.containers import VerticalScroll
from textual.widget import Widget
from textual.widgets import Static


class LyricsPanel(Widget):
    """Panel showing lyrics for the currently playing song."""

    DEFAULT_CSS = """
    LyricsPanel {
        width: 1fr;
        height: 1fr;
        background: $surface;
        border: solid $primary;
        display: none;
    }

    LyricsPanel.visible {
        display: block;
    }

    LyricsPanel .lyrics-header {
        height: 1;
        background: $primary;
        color: $text;
        text-style: bold;
        text-align: center;
        padding: 0 1;
    }

    LyricsPanel .lyrics-text {
        padding: 1 2;
        color: $text;
    }
    """

    def __init__(self, **kwargs):
        super().__init__(**kwargs)
        self._title = ""
        self._artist = ""

    def compose(self) -> ComposeResult:
        yield Static("Lyrics", classes="lyrics-header", id="lyrics-header")
        with VerticalScroll(id="lyrics-scroll"):
            yield Static("No lyrics available", id="lyrics-text", classes="lyrics-text")

    def set_lyrics(self, title: str, artist: str, text: str):
        """Set the lyrics content."""
        self._title = title
        self._artist = artist
        try:
            header = self.query_one("#lyrics-header", Static)
            header.update(f"{title} - {artist}")
            lyrics_w = self.query_one("#lyrics-text", Static)
            lyrics_w.update(text if text else "No lyrics available")
        except Exception:
            pass

    def clear_lyrics(self):
        """Clear lyrics display."""
        self._title = ""
        self._artist = ""
        try:
            header = self.query_one("#lyrics-header", Static)
            header.update("Lyrics")
            lyrics_w = self.query_one("#lyrics-text", Static)
            lyrics_w.update("No lyrics available")
        except Exception:
            pass

    def toggle_visibility(self):
        """Toggle the lyrics panel visibility."""
        self.toggle_class("visible")
