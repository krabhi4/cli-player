"""Help overlay widget — keybinding reference."""

from textual.app import ComposeResult
from textual.screen import ModalScreen
from textual.containers import Vertical
from textual.widgets import Static


HELP_TEXT = """\
[bold cyan]🎹 Keyboard Shortcuts[/bold cyan]

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
  [green]a[/green]           Add to Queue
  [green]c[/green]           Clear Queue

[bold]Navigation[/bold]
  [green]1[/green]           Albums tab
  [green]2[/green]           Artists tab
  [green]3[/green]           Songs tab
  [green]4[/green]           Playlists tab
  [green]5[/green]           Genres tab
  [green]Tab[/green]         Switch focus

[bold]Features[/bold]
  [green]/[/green]           Search
  [green]e[/green]           Toggle Equalizer
  [green]f[/green]           Star / Unstar
  [green]S[/green]           Server Manager
  [green]?[/green]           This Help

[bold]General[/bold]
  [green]q / Ctrl+C[/green]  Quit
"""


class HelpModal(ModalScreen[None]):
    """Modal showing keyboard shortcuts."""

    DEFAULT_CSS = """
    HelpModal {
        align: center middle;
    }

    HelpModal > Vertical {
        width: 60;
        height: auto;
        max-height: 80%;
        background: $surface;
        border: solid $primary;
        padding: 1 2;
    }
    """

    BINDINGS = [
        ("escape", "dismiss_help", "Close"),
        ("question_mark", "dismiss_help", "Close"),
    ]

    def compose(self) -> ComposeResult:
        with Vertical():
            yield Static(HELP_TEXT)
            yield Static(
                "\n[dim]Press [bold]Escape[/bold] or [bold]?[/bold] to close[/dim]",
            )

    def action_dismiss_help(self) -> None:
        self.dismiss(None)
