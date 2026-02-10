"""Queue view widget — displays the play queue sidebar."""

from textual.app import ComposeResult
from textual.message import Message
from textual.widget import Widget
from textual.widgets import DataTable, Label, Static

from ..subsonic import Song
from ..utils import format_duration


class QueueItemSelected(Message):
    """Message when a queue item is clicked."""

    def __init__(self, index: int, song: Song):
        super().__init__()
        self.index = index
        self.song = song


class QueueRemoveRequest(Message):
    """Message requesting removal of a queue item."""

    def __init__(self, index: int):
        super().__init__()
        self.index = index


class QueueMoveRequest(Message):
    """Message requesting a queue item be moved."""

    def __init__(self, from_index: int, to_index: int):
        super().__init__()
        self.from_index = from_index
        self.to_index = to_index


class QueueView(Widget):
    """Sidebar widget showing the play queue."""

    DEFAULT_CSS = """
    QueueView {
        width: 35;
        height: 1fr;
        border-left: solid $primary;
        background: $surface;
    }

    QueueView .queue-header {
        height: 1;
        background: $primary;
        color: $text;
        text-style: bold;
        padding: 0 1;
        text-align: center;
    }

    QueueView .queue-info {
        height: 1;
        color: $text-muted;
        padding: 0 1;
        background: $surface;
    }

    QueueView DataTable {
        height: 1fr;
    }

    QueueView DataTable > .datatable--cursor {
        background: $primary;
        color: $text;
    }
    """

    def __init__(self, **kwargs):
        super().__init__(**kwargs)
        self._songs: list[Song] = []
        self._current_index: int = -1

    def compose(self) -> ComposeResult:
        yield Static("Play Queue", classes="queue-header")
        yield Static("0 songs", classes="queue-info", id="queue-info")
        yield DataTable(id="queue-table")

    def on_mount(self) -> None:
        table = self.query_one("#queue-table", DataTable)
        table.cursor_type = "row"
        table.zebra_stripes = True
        table.add_columns("#", "Title", "Duration")

    def _title_max_len(self) -> int:
        """Calculate max title length based on widget width."""
        # Width minus columns: #(4) + Duration(6) + padding/borders(~5)
        available = self.size.width - 15
        return max(10, available)

    def update_queue(self, songs: list[Song], current_index: int = -1):
        """Update the queue display."""
        self._songs = list(songs)
        self._current_index = current_index

        table = self.query_one("#queue-table", DataTable)
        table.clear()

        max_title = self._title_max_len()

        for i, song in enumerate(songs):
            marker = ">" if i == current_index else " "
            title = song.title
            if len(title) > max_title:
                title = title[: max_title - 1] + "…"
            table.add_row(
                f"{marker}{i + 1}",
                title,
                format_duration(song.duration),
                key=str(i),
            )

        info = self.query_one("#queue-info", Static)
        total = sum(s.duration for s in songs)
        info.update(
            f"{len(songs)} songs | {format_duration(total)}"
        )

    def get_selected_index(self) -> int:
        """Get the selected queue index."""
        table = self.query_one("#queue-table", DataTable)
        if table.cursor_row is not None and table.cursor_row < len(self._songs):
            return table.cursor_row
        return -1

    def is_focused(self) -> bool:
        """Check if the queue table has focus."""
        try:
            table = self.query_one("#queue-table", DataTable)
            return table.has_focus
        except Exception:
            return False
