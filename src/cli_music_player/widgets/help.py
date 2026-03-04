"""Help overlay widget — keybinding reference."""

from typing import ClassVar

from textual.app import ComposeResult
from textual.binding import BindingType
from textual.containers import Horizontal, Vertical, VerticalScroll
from textual.screen import ModalScreen
from textual.widgets import Button, Static

HELP_TEXT = """\
[bold #e6edf3]Keyboard Shortcuts[/bold #e6edf3]

[bold]Playback[/bold]
  [#58a6ff]Space[/#58a6ff]       Play / Pause
  [#58a6ff]s[/#58a6ff]           Stop
  [#58a6ff]n[/#58a6ff]           Next track
  [#58a6ff]p[/#58a6ff]           Previous track
  [#58a6ff]→ / ←[/#58a6ff]       Seek ±5 seconds
  [#58a6ff]Shift+→/←[/#58a6ff]   Seek ±30 seconds

[bold]Volume[/bold]
  [#58a6ff]+ / =[/#58a6ff]       Volume Up
  [#58a6ff]- / _[/#58a6ff]       Volume Down
  [#58a6ff]m[/#58a6ff]           Mute toggle

[bold]Queue & Modes[/bold]
  [#58a6ff]z[/#58a6ff]           Toggle Shuffle
  [#58a6ff]r[/#58a6ff]           Cycle Repeat (Off → All → One)
  [#58a6ff]a[/#58a6ff]           Add highlighted song to Queue
  [#58a6ff]d / Delete[/#58a6ff]  Remove from Queue
  [#58a6ff]Shift+↑/↓[/#58a6ff]  Reorder Queue
  [#58a6ff]c[/#58a6ff]           Clear Queue

[bold]Playlist[/bold]
  [#58a6ff]P[/#58a6ff]           Save Queue as Playlist (name dialog)
                  Add songs to queue first, then press Shift+P

[bold]Navigation[/bold]
  [#58a6ff]1-6[/#58a6ff]         Switch tabs:
                  1=Albums  2=Artists  3=Songs
                  4=Playlists  5=Genres  6=Starred
  [#58a6ff]7[/#58a6ff]           Play History (songs played this session)
  [#58a6ff]Esc / Bksp[/#58a6ff] Go back (previous view)
  [#58a6ff]Tab[/#58a6ff]         Switch focus between panels

[bold]Album Sorting[/bold]
  [#58a6ff]o[/#58a6ff]           Cycle album sort order:
                  Newest → Random → Frequent →
                  Recent → Starred → A-Z

[bold]Starring[/bold]
  [#58a6ff]f[/#58a6ff]           Star / Unstar the highlighted song
                  (or the playing song if no table active)
                  View starred songs in tab 6

[bold]Equalizer[/bold]
  [#58a6ff]e[/#58a6ff]           Open / Close Equalizer
                  When open, first band auto-focuses:
                  [#58a6ff]←/→[/#58a6ff]  Switch between bands
                  [#58a6ff]↑/↓[/#58a6ff]  Adjust gain ±1 dB
                  [#58a6ff]Click[/#58a6ff] Set gain by position
                  [#58a6ff]Esc[/#58a6ff]  Close equalizer
                  Use preset dropdown and Reset/On-Off buttons

[bold]Features[/bold]
  [#58a6ff]/[/#58a6ff]           Search (songs, albums, artists)
  [#58a6ff]l[/#58a6ff]           Toggle Lyrics panel
  [#58a6ff]R[/#58a6ff]           Artist Radio (queue similar songs)
  [#58a6ff]S[/#58a6ff]           Server Manager (add/switch servers)
  [#58a6ff]?[/#58a6ff] or [#58a6ff]i[/#58a6ff]       This Help

[bold]Artist Drill-Down[/bold]
  Click an artist → see their Top Songs + Albums
  Press [#58a6ff]Esc[/#58a6ff] to go back to the full library

[bold]General[/bold]
  [#58a6ff]q / Ctrl+C[/#58a6ff]  Quit
"""


class HelpModal(ModalScreen[None]):
    """Modal showing keyboard shortcuts."""

    DEFAULT_CSS = """
    HelpModal {
        align: center middle;
    }

    HelpModal > #help-outer {
        width: 60;
        height: 80%;
        background: $surface;
        border: round #30363d;
        padding: 1 2;
    }

    HelpModal #help-scroll {
        height: 1fr;
    }

    HelpModal #help-close-row {
        width: 1fr;
        height: 5;
        align: center middle;
    }

    HelpModal #help-close-btn {
        min-width: 20;
        background: #21262d;
        color: #e6edf3;
        text-style: bold;
    }

    HelpModal #help-close-btn:hover {
        background: #30363d;
    }
    """

    BINDINGS: ClassVar[list[BindingType]] = [
        ("escape", "dismiss_help", "Close"),
        ("question_mark", "dismiss_help", "Close"),
    ]

    def compose(self) -> ComposeResult:
        with Vertical(id="help-outer"):
            with VerticalScroll(id="help-scroll"):
                yield Static(HELP_TEXT)
            with Horizontal(id="help-close-row"):
                yield Button("  Close  [Esc]  ", id="help-close-btn")

    def on_button_pressed(self, event: Button.Pressed) -> None:
        if event.button.id == "help-close-btn":
            self.dismiss(None)

    def action_dismiss_help(self) -> None:
        self.dismiss(None)
