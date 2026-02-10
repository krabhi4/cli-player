"""Help overlay widget — keybinding reference."""

from textual.app import ComposeResult
from textual.screen import ModalScreen
from textual.containers import Vertical, VerticalScroll, Horizontal
from textual.widgets import Button, Static


HELP_TEXT = """\
[bold cyan]Keyboard Shortcuts[/bold cyan]

[bold]Playback[/bold]
  [green]Space[/green]       Play / Pause
  [green]s[/green]           Stop
  [green]n[/green]           Next track
  [green]p[/green]           Previous track
  [green]→ / ←[/green]       Seek ±5 seconds
  [green]Shift+→/←[/green]   Seek ±30 seconds

[bold]Volume[/bold]
  [green]+ / =[/green]       Volume Up
  [green]- / _[/green]       Volume Down
  [green]m[/green]           Mute toggle

[bold]Queue & Modes[/bold]
  [green]z[/green]           Toggle Shuffle
  [green]r[/green]           Cycle Repeat (Off → All → One)
  [green]a[/green]           Add highlighted song to Queue
  [green]d / Delete[/green]  Remove from Queue
  [green]Shift+↑/↓[/green]  Reorder Queue
  [green]c[/green]           Clear Queue

[bold]Playlist[/bold]
  [green]P[/green]           Save Queue as Playlist (name dialog)
                  Add songs to queue first, then press Shift+P

[bold]Navigation[/bold]
  [green]1-6[/green]         Switch tabs:
                  1=Albums  2=Artists  3=Songs
                  4=Playlists  5=Genres  6=Starred
  [green]7[/green]           Play History (songs played this session)
  [green]Esc / Bksp[/green] Go back (previous view)
  [green]Tab[/green]         Switch focus between panels

[bold]Album Sorting[/bold]
  [green]o[/green]           Cycle album sort order:
                  Newest → Random → Frequent →
                  Recent → Starred → A-Z

[bold]Starring[/bold]
  [green]f[/green]           Star / Unstar the highlighted song
                  (or the playing song if no table active)
                  View starred songs in tab 6

[bold]Equalizer[/bold]
  [green]e[/green]           Open / Close Equalizer
                  When open, first band auto-focuses:
                  [green]←/→[/green]  Switch between bands
                  [green]↑/↓[/green]  Adjust gain ±1 dB
                  [green]Click[/green] Set gain by position
                  [green]Esc[/green]  Close equalizer
                  Use preset dropdown and Reset/On-Off buttons

[bold]Features[/bold]
  [green]/[/green]           Search (songs, albums, artists)
  [green]l[/green]           Toggle Lyrics panel
  [green]R[/green]           Artist Radio (queue similar songs)
  [green]S[/green]           Server Manager (add/switch servers)
  [green]?[/green] or [green]i[/green]       This Help

[bold]Artist Drill-Down[/bold]
  Click an artist → see their Top Songs + Albums
  Press [green]Esc[/green] to go back to the full library

[bold]General[/bold]
  [green]q / Ctrl+C[/green]  Quit
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
        border: solid $primary;
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
        background: #7aa2f7;
        color: #1a1b26;
        text-style: bold;
    }

    HelpModal #help-close-btn:hover {
        background: #7dcfff;
    }
    """

    BINDINGS = [
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
