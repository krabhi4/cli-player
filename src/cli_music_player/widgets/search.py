"""Search modal widget."""

from textual.app import ComposeResult
from textual.containers import Vertical
from textual.screen import ModalScreen
from textual.widgets import Input, Label, DataTable, Static, TabbedContent, TabPane

from ..subsonic import SubsonicClient, Artist, Album, Song
from ..utils import format_duration


class SearchResults:
    """Container for search results."""

    def __init__(self):
        self.artists: list[Artist] = []
        self.albums: list[Album] = []
        self.songs: list[Song] = []


class SearchModal(ModalScreen[Song | Album | Artist | None]):
    """Modal screen for searching the library."""

    DEFAULT_CSS = """
    SearchModal {
        align: center middle;
    }

    SearchModal > Vertical {
        width: 80%;
        max-width: 100;
        height: 80%;
        background: $surface;
        border: solid $primary;
        padding: 1;
    }

    SearchModal .search-title {
        text-style: bold;
        color: $text;
        text-align: center;
        height: 1;
        margin-bottom: 1;
    }

    SearchModal Input {
        margin-bottom: 1;
    }

    SearchModal DataTable {
        height: 1fr;
    }

    SearchModal .search-status {
        height: 1;
        color: $text-muted;
        text-align: center;
    }
    """

    BINDINGS = [
        ("escape", "dismiss_search", "Close"),
    ]

    def __init__(self, client: SubsonicClient, **kwargs):
        super().__init__(**kwargs)
        self.client = client
        self.results = SearchResults()

    def compose(self) -> ComposeResult:
        with Vertical():
            yield Static("🔍 Search Library", classes="search-title")
            yield Input(placeholder="Type to search…", id="search-input")
            yield Static("Type at least 2 characters to search", classes="search-status", id="search-status")
            with TabbedContent(id="search-tabs"):
                with TabPane(f"Songs (0)", id="search-tab-songs"):
                    yield DataTable(id="search-songs")
                with TabPane(f"Albums (0)", id="search-tab-albums"):
                    yield DataTable(id="search-albums")
                with TabPane(f"Artists (0)", id="search-tab-artists"):
                    yield DataTable(id="search-artists")

    def on_mount(self) -> None:
        songs_table = self.query_one("#search-songs", DataTable)
        songs_table.cursor_type = "row"
        songs_table.add_columns("#", "Title", "Artist", "Album", "Duration")

        albums_table = self.query_one("#search-albums", DataTable)
        albums_table.cursor_type = "row"
        albums_table.add_columns("Album", "Artist", "Year")

        artists_table = self.query_one("#search-artists", DataTable)
        artists_table.cursor_type = "row"
        artists_table.add_columns("Artist", "Albums")

        self.query_one("#search-input", Input).focus()

    async def on_input_changed(self, event: Input.Changed) -> None:
        query = event.value.strip()
        if len(query) < 2:
            status = self.query_one("#search-status", Static)
            status.update("Type at least 2 characters to search")
            return

        status = self.query_one("#search-status", Static)
        status.update("Searching…")

        try:
            artists, albums, songs = self.client.search(query)
            self.results.artists = artists
            self.results.albums = albums
            self.results.songs = songs

            # Update songs table
            songs_table = self.query_one("#search-songs", DataTable)
            songs_table.clear()
            for i, song in enumerate(songs):
                songs_table.add_row(
                    str(i + 1), song.title, song.artist,
                    song.album, format_duration(song.duration),
                    key=song.id,
                )

            # Update albums table
            albums_table = self.query_one("#search-albums", DataTable)
            albums_table.clear()
            for album in albums:
                albums_table.add_row(
                    album.name, album.artist,
                    str(album.year) if album.year else "",
                    key=album.id,
                )

            # Update artists table
            artists_table = self.query_one("#search-artists", DataTable)
            artists_table.clear()
            for artist in artists:
                artists_table.add_row(
                    artist.name, str(artist.album_count),
                    key=artist.id,
                )

            total = len(artists) + len(albums) + len(songs)
            status.update(
                f"Found {len(songs)} songs, {len(albums)} albums, {len(artists)} artists"
            )

        except Exception as e:
            status.update(f"Error: {e}")

    def on_data_table_row_selected(self, event: DataTable.RowSelected) -> None:
        table_id = event.data_table.id
        if table_id == "search-songs" and event.cursor_row < len(self.results.songs):
            self.dismiss(self.results.songs[event.cursor_row])
        elif table_id == "search-albums" and event.cursor_row < len(self.results.albums):
            self.dismiss(self.results.albums[event.cursor_row])
        elif table_id == "search-artists" and event.cursor_row < len(self.results.artists):
            self.dismiss(self.results.artists[event.cursor_row])

    def action_dismiss_search(self) -> None:
        self.dismiss(None)
