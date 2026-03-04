"""Library browser widget — browse artists, albums, songs, genres, playlists."""

from textual.app import ComposeResult
from textual.message import Message
from textual.widget import Widget
from textual.widgets import (
    DataTable,
    Static,
    TabbedContent,
    TabPane,
)

from ..subsonic import Album, Artist, Genre, Playlist, Song
from ..utils import format_duration


class SongSelected(Message):
    """Message sent when a song is selected for playback."""

    def __init__(self, song: Song, all_songs: list[Song], index: int = 0):
        super().__init__()
        self.song = song
        self.all_songs = all_songs
        self.index = index


class AddToQueue(Message):
    """Message to add songs to queue."""

    def __init__(self, songs: list[Song]):
        super().__init__()
        self.songs = songs


class BrowseArtist(Message):
    def __init__(self, artist: Artist):
        super().__init__()
        self.artist = artist


class BrowseAlbum(Message):
    def __init__(self, album: Album):
        super().__init__()
        self.album = album


class BrowsePlaylist(Message):
    def __init__(self, playlist: Playlist):
        super().__init__()
        self.playlist = playlist


class BrowseGenre(Message):
    def __init__(self, genre: Genre):
        super().__init__()
        self.genre = genre


class SongTable(DataTable):
    """A DataTable specialized for displaying songs."""

    DEFAULT_CSS = """
    SongTable {
        height: 1fr;
    }
    SongTable > .datatable--header {
        background: $surface;
        color: $text-muted;
        text-style: bold;
    }
    SongTable > .datatable--cursor {
        background: $primary;
        color: $text;
    }
    """

    def __init__(self, **kwargs):
        super().__init__(**kwargs)
        self._songs: list[Song] = []
        self.cursor_type = "row"
        self.zebra_stripes = True

    def set_songs(self, songs: list[Song]):
        """Populate the table with songs."""
        self._songs = list(songs)
        self.clear(columns=True)
        self.add_columns("#", "Title", "Artist", "Album", "Duration")
        for i, song in enumerate(songs):
            self.add_row(
                str(i + 1),
                song.title,
                song.artist,
                song.album,
                format_duration(song.duration),
                key=song.id,
            )

    def get_selected_song(self) -> tuple[Song | None, int]:
        """Get the currently highlighted song."""
        if self._songs and self.cursor_row is not None:
            idx = self.cursor_row
            if 0 <= idx < len(self._songs):
                return self._songs[idx], idx
        return None, -1

    @property
    def songs(self) -> list[Song]:
        return self._songs


class AlbumList(DataTable):
    """Data table for displaying albums."""

    DEFAULT_CSS = """
    AlbumList {
        height: 1fr;
    }
    AlbumList > .datatable--cursor {
        background: $primary;
        color: $text;
    }
    """

    def __init__(self, **kwargs):
        super().__init__(**kwargs)
        self._albums: list[Album] = []
        self.cursor_type = "row"
        self.zebra_stripes = True

    def set_albums(self, albums: list[Album]):
        self._albums = list(albums)
        self.clear(columns=True)
        self.add_columns("Album", "Artist", "Year", "Tracks")
        for album in albums:
            self.add_row(
                album.name,
                album.artist,
                str(album.year) if album.year else "",
                str(album.song_count),
                key=album.id,
            )

    def get_selected_album(self) -> Album | None:
        if self._albums and self.cursor_row is not None:
            idx = self.cursor_row
            if 0 <= idx < len(self._albums):
                return self._albums[idx]
        return None


class ArtistList(DataTable):
    """Data table for displaying artists."""

    DEFAULT_CSS = """
    ArtistList {
        height: 1fr;
    }
    ArtistList > .datatable--cursor {
        background: $primary;
        color: $text;
    }
    """

    def __init__(self, **kwargs):
        super().__init__(**kwargs)
        self._artists: list[Artist] = []
        self.cursor_type = "row"
        self.zebra_stripes = True

    def set_artists(self, artists: list[Artist]):
        self._artists = list(artists)
        self.clear(columns=True)
        self.add_columns("Artist", "Albums")
        for artist in artists:
            self.add_row(
                artist.name,
                str(artist.album_count),
                key=artist.id,
            )

    def get_selected_artist(self) -> Artist | None:
        if self._artists and self.cursor_row is not None:
            idx = self.cursor_row
            if 0 <= idx < len(self._artists):
                return self._artists[idx]
        return None


class PlaylistList(DataTable):
    """Data table for displaying playlists."""

    DEFAULT_CSS = """
    PlaylistList {
        height: 1fr;
    }
    PlaylistList > .datatable--cursor {
        background: $primary;
        color: $text;
    }
    """

    def __init__(self, **kwargs):
        super().__init__(**kwargs)
        self._playlists: list[Playlist] = []
        self.cursor_type = "row"
        self.zebra_stripes = True

    def set_playlists(self, playlists: list[Playlist]):
        self._playlists = list(playlists)
        self.clear(columns=True)
        self.add_columns("Name", "Songs", "Duration")
        for pl in playlists:
            self.add_row(
                pl.name,
                str(pl.song_count),
                format_duration(pl.duration),
                key=pl.id,
            )

    def get_selected_playlist(self) -> Playlist | None:
        if self._playlists and self.cursor_row is not None:
            idx = self.cursor_row
            if 0 <= idx < len(self._playlists):
                return self._playlists[idx]
        return None


class GenreList(DataTable):
    """Data table for displaying genres."""

    DEFAULT_CSS = """
    GenreList {
        height: 1fr;
    }
    GenreList > .datatable--cursor {
        background: $primary;
        color: $text;
    }
    """

    def __init__(self, **kwargs):
        super().__init__(**kwargs)
        self._genres: list[Genre] = []
        self.cursor_type = "row"
        self.zebra_stripes = True

    def set_genres(self, genres: list[Genre]):
        self._genres = list(genres)
        self.clear(columns=True)
        self.add_columns("Genre", "Songs", "Albums")
        for genre in genres:
            self.add_row(
                genre.name,
                str(genre.song_count),
                str(genre.album_count),
                key=genre.name,
            )

    def get_selected_genre(self) -> Genre | None:
        if self._genres and self.cursor_row is not None:
            idx = self.cursor_row
            if 0 <= idx < len(self._genres):
                return self._genres[idx]
        return None


class LibraryBrowser(Widget):
    """Main library browsing widget with tabs for different views."""

    DEFAULT_CSS = """
    LibraryBrowser {
        width: 1fr;
        height: 1fr;
    }

    LibraryBrowser .browser-header {
        height: 1;
        background: $surface;
        color: $text;
        padding: 0 1;
        text-style: bold;
    }

    LibraryBrowser .browser-nav {
        height: 1;
        background: $surface-darken-1;
        padding: 0 1;
        color: $text-muted;
    }
    """

    def __init__(self, **kwargs):
        super().__init__(**kwargs)
        self._breadcrumb: list[str] = ["Library"]
        self._current_view: str = "albums"  # Default view
        self._current_songs: list[Song] = []

    def compose(self) -> ComposeResult:
        yield Static("Library", classes="browser-header", id="browser-header")
        yield Static("", classes="browser-nav", id="browser-nav")
        with TabbedContent(id="browser-tabs"):
            with TabPane("Albums", id="tab-albums"):
                yield AlbumList(id="album-list")
            with TabPane("Artists", id="tab-artists"):
                yield ArtistList(id="artist-list")
            with TabPane("Songs", id="tab-songs"):
                yield SongTable(id="song-table-main")
            with TabPane("Playlists", id="tab-playlists"):
                yield PlaylistList(id="playlist-list")
            with TabPane("Genres", id="tab-genres"):
                yield GenreList(id="genre-list")
            with TabPane("Starred", id="tab-starred"):
                yield SongTable(id="starred-songs")
            with TabPane("History", id="tab-history"):
                yield SongTable(id="history-songs")

    def set_breadcrumb(self, *parts: str):
        self._breadcrumb = list(parts)
        nav = self.query_one("#browser-nav", Static)
        nav.update(" > ".join(self._breadcrumb))

    def set_header(self, text: str):
        header = self.query_one("#browser-header", Static)
        header.update(text)
