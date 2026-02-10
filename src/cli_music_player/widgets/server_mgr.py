"""Server management modal — add, edit, remove, switch Navidrome servers."""

from textual.app import ComposeResult
from textual.containers import Vertical, Horizontal
from textual.message import Message
from textual.screen import ModalScreen
from textual.widgets import Button, DataTable, Input, Label, Static

from ..config import AppConfig
from ..subsonic import SubsonicClient


class ServerChanged(Message):
    """Message when the active server changes."""

    def __init__(self, server_index: int):
        super().__init__()
        self.server_index = server_index


class ServerManagerModal(ModalScreen[int | None]):
    """Modal for managing Navidrome server connections."""

    DEFAULT_CSS = """
    ServerManagerModal {
        align: center middle;
    }

    ServerManagerModal > Vertical {
        width: 70%;
        max-width: 90;
        height: 80%;
        background: $surface;
        border: solid $primary;
        padding: 1;
    }

    ServerManagerModal .sm-title {
        text-style: bold;
        color: $text;
        text-align: center;
        height: 1;
        margin-bottom: 1;
    }

    ServerManagerModal DataTable {
        height: 1fr;
        max-height: 10;
        margin-bottom: 1;
    }

    ServerManagerModal .sm-form-title {
        text-style: bold;
        color: $primary;
        height: 1;
        margin-bottom: 1;
    }

    ServerManagerModal Input {
        margin-bottom: 1;
    }

    ServerManagerModal .sm-buttons {
        height: 3;
        align: center middle;
    }

    ServerManagerModal Button {
        margin: 0 1;
    }

    ServerManagerModal .sm-status {
        height: 1;
        color: $text-muted;
        text-align: center;
    }
    """

    BINDINGS = [
        ("escape", "dismiss_modal", "Close"),
    ]

    def __init__(self, config: AppConfig, **kwargs):
        super().__init__(**kwargs)
        self.config = config

    def compose(self) -> ComposeResult:
        with Vertical():
            yield Static("🖥 Server Manager", classes="sm-title")

            # Server list
            yield DataTable(id="server-list")

            # Status
            yield Static("", classes="sm-status", id="sm-status")

            # Add server form
            yield Static("Add New Server", classes="sm-form-title")
            yield Input(placeholder="Display Name (e.g., Main Library)", id="sm-name")
            yield Input(
                placeholder="URL (e.g., http://localhost:4533)",
                id="sm-url",
                value="http://localhost:",
            )
            yield Input(placeholder="Username", id="sm-username")
            yield Input(placeholder="Password", id="sm-password", password=True)

            with Horizontal(classes="sm-buttons"):
                yield Button("Test & Add", id="sm-add", variant="success")
                yield Button("Remove Selected", id="sm-remove", variant="error")
                yield Button("Switch To", id="sm-switch", variant="primary")
                yield Button("Close", id="sm-close", variant="default")

    def on_mount(self) -> None:
        table = self.query_one("#server-list", DataTable)
        table.cursor_type = "row"
        table.add_columns("", "Name", "URL", "User")
        self._refresh_server_list()

    def _refresh_server_list(self):
        table = self.query_one("#server-list", DataTable)
        table.clear()
        for i, server in enumerate(self.config.servers):
            active = "✓" if i == self.config.active_server_index else ""
            table.add_row(active, server.name, server.url, server.username, key=str(i))

    def on_button_pressed(self, event: Button.Pressed) -> None:
        if event.button.id == "sm-add":
            self._add_server()
        elif event.button.id == "sm-remove":
            self._remove_server()
        elif event.button.id == "sm-switch":
            self._switch_server()
        elif event.button.id == "sm-close":
            self.dismiss(self.config.active_server_index)

    def _add_server(self):
        name = self.query_one("#sm-name", Input).value.strip()
        url = self.query_one("#sm-url", Input).value.strip()
        username = self.query_one("#sm-username", Input).value.strip()
        password = self.query_one("#sm-password", Input).value

        status = self.query_one("#sm-status", Static)

        if not all([name, url, username, password]):
            status.update("⚠ Please fill in all fields")
            return

        status.update("Testing connection…")

        # Test connection
        try:
            client = SubsonicClient(url, username, password)
            if client.ping():
                self.config.add_server(name, url, username, password)
                status.update(f"✓ Added '{name}' successfully!")
                self._refresh_server_list()
                # Clear form
                self.query_one("#sm-name", Input).value = ""
                self.query_one("#sm-url", Input).value = "http://localhost:"
                self.query_one("#sm-username", Input).value = ""
                self.query_one("#sm-password", Input).value = ""
            else:
                status.update("✗ Connection failed — check URL and credentials")
        except Exception as e:
            status.update(f"✗ Error: {e}")

    def _remove_server(self):
        table = self.query_one("#server-list", DataTable)
        if table.cursor_row is not None and table.cursor_row < len(self.config.servers):
            idx = table.cursor_row
            name = self.config.servers[idx].name
            self.config.remove_server(idx)
            self._refresh_server_list()
            status = self.query_one("#sm-status", Static)
            status.update(f"Removed '{name}'")

    def _switch_server(self):
        table = self.query_one("#server-list", DataTable)
        if table.cursor_row is not None and table.cursor_row < len(self.config.servers):
            idx = table.cursor_row
            self.config.set_active_server(idx)
            self._refresh_server_list()
            status = self.query_one("#sm-status", Static)
            status.update(f"Switched to '{self.config.servers[idx].name}'")

    def action_dismiss_modal(self) -> None:
        self.dismiss(self.config.active_server_index)
