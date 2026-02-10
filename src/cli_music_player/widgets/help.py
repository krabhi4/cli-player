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
  [green]a[/green]           Add to Queue
  [green]c[/green]           Clear Queue

[bold]Navigation[/bold]
  [green]1-5[/green]         Switch tabs
  [green]Tab[/green]         Switch focus

[bold]Features[/bold]
  [green]/[/green]           Search
  [green]e[/green]           Toggle Equalizer
  [green]f[/green]           Star / Unstar
  [green]S[/green]           Server Manager
  [green]?[/green] or [green]i[/green]       This Help

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
