"""Main Textual Application — orchestrates all components."""

import contextlib
from typing import ClassVar

from textual.app import App, ComposeResult
from textual.binding import Binding, BindingType
from textual.containers import Horizontal, Vertical
from textual.screen import ModalScreen
from textual.widgets import (
    Button,
    DataTable,
    Input,
    Select,
    Static,
    TabbedContent,
)

from .config import AppConfig
from .equalizer import Equalizer
from .player import PlaybackState, Player
from .queue import QueueManager, RepeatMode
from .subsonic import (
    Album,
    Artist,
    Genre,
    Playlist,
    Song,
    SubsonicClient,
)
from .widgets.browser import (
    AlbumList,
    ArtistList,
    GenreList,
    LibraryBrowser,
    PlaylistList,
    SongTable,
)
from .widgets.equalizer import EQBand, EqualizerWidget
from .widgets.help import HelpModal
from .widgets.lyrics import LyricsPanel
from .widgets.now_playing import NowPlaying
from .widgets.queue_view import QueueView
from .widgets.search import SearchModal
from .widgets.server_mgr import ServerManagerModal

# Album sort types supported by the Subsonic API
ALBUM_SORT_TYPES = [
    ("newest", "Newest"),
    ("random", "Random"),
    ("frequent", "Frequent"),
    ("recent", "Recent"),
    ("starred", "Starred"),
    ("alphabeticalByName", "A-Z"),
]

MAX_PLAY_HISTORY = 100


class SavePlaylistModal(ModalScreen[str]):
    """Modal for naming a playlist before saving."""

    DEFAULT_CSS = """
    SavePlaylistModal {
        align: center middle;
    }
    SavePlaylistModal > #playlist-modal-outer {
        width: 50;
        height: auto;
        background: $surface;
        border: solid $primary;
        padding: 1 2;
    }
    SavePlaylistModal .modal-title {
        text-style: bold;
        color: $primary;
        text-align: center;
        width: 1fr;
        height: 1;
        margin: 0 0 1 0;
    }
    SavePlaylistModal #playlist-name-input {
        width: 1fr;
        margin: 0 0 1 0;
    }
    SavePlaylistModal .modal-buttons {
        height: 3;
        align: center middle;
    }
    """

    BINDINGS: ClassVar[list[BindingType]] = [("escape", "cancel", "Cancel")]

    def __init__(self, default_name: str = ""):
        super().__init__()
        self._default_name = default_name

    def compose(self) -> ComposeResult:
        with Vertical(id="playlist-modal-outer"):
            yield Static("Save Queue as Playlist", classes="modal-title")
            yield Input(
                value=self._default_name,
                placeholder="Enter playlist name...",
                id="playlist-name-input",
            )
            with Horizontal(classes="modal-buttons"):
                yield Button("Save", id="playlist-save-btn", variant="success")
                yield Button("Cancel", id="playlist-cancel-btn", variant="error")

    def on_mount(self) -> None:
        self.query_one("#playlist-name-input", Input).focus()

    def on_button_pressed(self, event: Button.Pressed) -> None:
        if event.button.id == "playlist-save-btn":
            name = self.query_one("#playlist-name-input", Input).value.strip()
            if name:
                self.dismiss(name)
        elif event.button.id == "playlist-cancel-btn":
            self.dismiss(None)

    def on_input_submitted(self, event: Input.Submitted) -> None:
        name = event.value.strip()
        if name:
            self.dismiss(name)

    def action_cancel(self) -> None:
        self.dismiss(None)


class MusicPlayerApp(App):
    """CLI Music Player for Navidrome."""

    TITLE = "CLI Music Player"
    SUB_TITLE = "Navidrome Terminal Client"

    CSS_PATH = "styles/app.tcss"

    BINDINGS: ClassVar[list[BindingType]] = [  # type: ignore[assignment]
        Binding("space", "toggle_pause", "Play/Pause", show=True, priority=True),
        Binding("n", "next_track", "Next", show=True),
        Binding("p", "prev_track", "Prev", show=True),
        Binding("s", "stop_playback", "Stop", show=False),
        Binding("right", "seek_forward", "Seek +5s", show=False),
        Binding("left", "seek_backward", "Seek -5s", show=False),
        Binding("shift+right", "seek_forward_long", "Seek +30s", show=False),
        Binding("shift+left", "seek_backward_long", "Seek -30s", show=False),
        Binding("plus", "volume_up", "Vol+", show=True, priority=True),
        Binding("equal", "volume_up", "Vol+", show=False, priority=True),
        Binding("minus", "volume_down", "Vol-", show=True, priority=True),
        Binding("underscore", "volume_down", "Vol-", show=False, priority=True),
        Binding("m", "mute_toggle", "Mute", show=False),
        Binding("z", "toggle_shuffle", "Shuffle", show=True),
        Binding("r", "cycle_repeat", "Repeat", show=True),
        Binding("a", "add_to_queue", "Add Queue", show=False),
        Binding("c", "clear_queue", "Clear Q", show=False),
        Binding("d", "remove_from_queue", "Remove", show=False),
        Binding("delete", "remove_from_queue", "Remove", show=False),
        Binding("shift+up", "queue_move_up", "Move Up", show=False),
        Binding("shift+down", "queue_move_down", "Move Down", show=False),
        Binding("slash", "open_search", "Search", show=True),
        Binding("e", "toggle_eq", "EQ", show=True),
        Binding("l", "toggle_lyrics", "Lyrics", show=False),
        Binding("f", "toggle_star", "Star", show=False),
        Binding("R", "artist_radio", "Radio", show=False),
        Binding("P", "save_playlist", "Save Playlist", show=False),
        Binding("o", "cycle_album_sort", "Sort", show=False),
        Binding("S", "open_servers", "Servers", show=True),
        Binding("question_mark", "show_help", "Help", show=True),
        Binding("i", "show_help", "Info", show=False),
        Binding("1", "tab_albums", "Albums", show=False),
        Binding("2", "tab_artists", "Artists", show=False),
        Binding("3", "tab_songs", "Songs", show=False),
        Binding("4", "tab_playlists", "Playlists", show=False),
        Binding("5", "tab_genres", "Genres", show=False),
        Binding("6", "tab_starred", "Starred", show=False),
        Binding("7", "tab_history", "History", show=False),
        Binding("escape", "go_back", "Back", show=False, priority=True),
        Binding("backspace", "go_back", "Back", show=False, priority=True),
        Binding("q", "quit_app", "Quit", show=True),
    ]

    def __init__(self):
        super().__init__()
        self.config = AppConfig()
        self.player = Player(audio_device=self.config.audio_device)
        self.equalizer = Equalizer(self.config)
        self.queue_mgr = QueueManager()
        self.client: SubsonicClient | None = None

        # Restore saved state
        self.player.volume = self.config.volume
        self.queue_mgr.set_shuffle(self.config.shuffle)
        if self.config.repeat_mode == "all":
            self.queue_mgr.set_repeat(RepeatMode.ALL)
        elif self.config.repeat_mode == "one":
            self.queue_mgr.set_repeat(RepeatMode.ONE)

        # Wire up player callbacks
        self.player.on_track_end = self._on_track_end
        self.player.on_position_update = self._on_position_update

        # Connect equalizer to player
        self.equalizer.set_player(self.player)

        # Current browsing state
        self._current_album_songs: list[Song] = []
        self._scrobble_reported = False

        # Navigation history for back navigation (limit to prevent memory bloat)
        self._nav_history: list[dict] = []
        self._max_nav_history = 50  # Maximum navigation history entries

        # Album sort state
        self._album_sort_index: int = 0

        # Starred song IDs — for star/unstar toggle
        self._starred_ids: set[str] = set()

        # Play history — most recent first
        self._play_history: list[Song] = []

    def compose(self) -> ComposeResult:
        yield Static("CLI Music Player", id="app-header")
        with Horizontal(id="main-area"):
            with Vertical(id="content-area"):
                if self.config.active_server is None:
                    # Welcome screen — no server configured
                    with Vertical(id="welcome"), Vertical(id="welcome-box"):
                        yield Static("CLI Music Player", classes="welcome-title")
                        yield Static(
                            "\nNo servers configured yet.\n"
                            "Press [bold]S[/bold] to add a Navidrome server.\n",
                            classes="welcome-text",
                        )
                else:
                    yield LibraryBrowser(id="library-browser")
                yield EqualizerWidget(self.equalizer, id="eq-widget")
                yield LyricsPanel(id="lyrics-panel")
            yield QueueView(id="queue-view")
        yield NowPlaying(id="now-playing")
        yield Static(
            "v2.0.2 | ? Help | Space Play/Pause | n Next | p Prev | / Search | o Sort | P Save Playlist | S Servers | q Quit",
            id="status-bar",
        )

    async def on_mount(self) -> None:
        """Initialize after mount."""
        # Update now-playing with saved state
        np = self.query_one("#now-playing", NowPlaying)
        np.volume = self.player.volume
        np.shuffle_on = self.queue_mgr.shuffle
        np.repeat_mode = self.queue_mgr.repeat.label

        if self.config.active_server:
            np.server_name = self.config.active_server.name
            self._connect_to_server()
            # Load initial data
            self.call_later(self._load_library)

    def _connect_to_server(self):
        """Connect to the active Navidrome server."""
        server = self.config.active_server
        if server:
            password = self.config.get_password(server)
            self.client = SubsonicClient(server.url, server.username, password)

    async def _load_library(self, restore_tab: str = ""):  # noqa: PLR0915
        """Load initial library data. Optionally restore a specific tab."""
        if not self.client:
            return

        self._show_status("Loading library...")

        try:
            # Load albums
            sort_type = ALBUM_SORT_TYPES[self._album_sort_index][0]
            albums = self.client.get_album_list(sort_type, size=50)
            try:
                album_list = self.query_one("#album-list", AlbumList)
                album_list.set_albums(albums)
            except Exception as e:
                self._show_status(f"Albums widget error: {e}")

            # Load artists
            artists = self.client.get_artists()
            try:
                artist_list = self.query_one("#artist-list", ArtistList)
                artist_list.set_artists(artists)
            except Exception as e:
                self._show_status(f"Artists widget error: {e}")

            # Load random songs for the Songs tab
            songs = self.client.get_random_songs(size=50)
            try:
                song_table = self.query_one("#song-table-main", SongTable)
                song_table.set_songs(songs)
            except Exception as e:
                self._show_status(f"Songs widget error: {e}")

            # Load playlists
            playlists = self.client.get_playlists()
            try:
                playlist_list = self.query_one("#playlist-list", PlaylistList)
                playlist_list.set_playlists(playlists)
            except Exception as e:
                self._show_status(f"Playlists widget error: {e}")

            # Load genres
            genres = self.client.get_genres()
            try:
                genre_list = self.query_one("#genre-list", GenreList)
                genre_list.set_genres(genres)
            except Exception as e:
                self._show_status(f"Genres widget error: {e}")

            # Load starred songs and populate starred IDs
            starred_count = 0
            try:
                _, _, starred_songs = self.client.get_starred()
                starred_count = len(starred_songs)
                self._starred_ids = {s.id for s in starred_songs}
                starred_table = self.query_one("#starred-songs", SongTable)
                starred_table.set_songs(starred_songs)
            except Exception as e:
                self._show_status(f"Starred load error: {e}")

            # Update browser header
            try:
                browser = self.query_one("#library-browser", LibraryBrowser)
                server = self.config.active_server
                browser.set_header(f"{server.name} - Library" if server else "Library")
                browser.set_breadcrumb("Library")
            except Exception:
                pass

            # Restore tab if specified
            if restore_tab:
                try:
                    tabs = self.query_one("#browser-tabs", TabbedContent)
                    tabs.active = restore_tab
                except Exception:
                    pass

            sort_label = ALBUM_SORT_TYPES[self._album_sort_index][1]
            self._show_status(
                f"Library loaded | {starred_count} starred | Albums: {sort_label} | "
                "o Sort | P Save Playlist | ? Help | q Quit"
            )

        except Exception as e:
            self._show_status(f"Error loading library: {e}")

    # ─── check_action — conditionally disable go_back ─────────

    def check_action(self, action: str, parameters: tuple) -> bool | None:
        """Disable certain app bindings when specific widgets are focused."""
        focused = self.focused

        if action == "go_back" and focused and isinstance(focused, (Input, Select, EQBand)):
            return None

        # Disable seek bindings when EQ band is focused (left/right navigate bands)
        if (
            action
            in (
                "seek_forward",
                "seek_backward",
                "seek_forward_long",
                "seek_backward_long",
            )
            and focused
            and isinstance(focused, EQBand)
        ):
            return None

        return True

    # ─── Playback Actions ────────────────────────────────────────

    def _play_song(self, song: Song, queue_songs: list[Song] | None = None, start_index: int = 0):
        """Play a song and optionally set up the queue."""
        if not self.client:
            self._show_status("No server connected")
            return

        if queue_songs:
            self.queue_mgr.set_queue(queue_songs, start_index)

        url = self.client.stream_url(song.id)
        self.player.play(url, song)
        self._scrobble_reported = False

        # Add to play history (most recent first, avoid duplicates at top)
        if not self._play_history or self._play_history[0].id != song.id:
            self._play_history.insert(0, song)
            if len(self._play_history) > MAX_PLAY_HISTORY:
                self._play_history = self._play_history[:MAX_PLAY_HISTORY]

        # Update now-playing
        np = self.query_one("#now-playing", NowPlaying)
        np.update_track(
            song.title,
            song.artist,
            song.album,
            song.duration,
            song.bitrate,
            song.suffix,
        )
        np.state = "playing"

        # Report now playing
        with contextlib.suppress(Exception):
            self.client.now_playing(song.id)

        # Update queue view
        self._update_queue_display()

        # Fetch lyrics for the new song
        self._fetch_lyrics(song)

    def _fetch_lyrics(self, song: Song):
        """Fetch and display lyrics for a song."""
        if not self.client:
            return
        try:
            lyrics_text = self.client.get_lyrics(song.artist, song.title)
            lyrics = self.query_one("#lyrics-panel", LyricsPanel)
            lyrics.set_lyrics(song.title, song.artist, lyrics_text)
        except Exception:
            try:
                lyrics = self.query_one("#lyrics-panel", LyricsPanel)
                lyrics.set_lyrics(song.title, song.artist, "")
            except Exception:
                pass

    def _on_track_end(self):
        """Called when the current track finishes."""
        # Scrobble (only if not already scrobbled at 50%)
        if self.client and self.player.current_song and not self._scrobble_reported:
            try:
                self.client.scrobble(self.player.current_song.id)
                self._scrobble_reported = True
            except Exception:
                pass

        # Auto-advance
        next_song = self.queue_mgr.next()
        if next_song:
            self.call_from_thread(self._play_song, next_song)
        else:
            self.call_from_thread(self._stop_playback)

    def _on_position_update(self, position: float, duration: float):
        """Called periodically with playback position."""
        try:
            np = self.query_one("#now-playing", NowPlaying)
            self.call_from_thread(np.update_position, position, duration)

            # Scrobble at 50% or 4 minutes
            if (
                not self._scrobble_reported
                and self.player.current_song
                and self.client
                and ((duration > 0 and position / duration > 0.5) or position > 240)
            ):
                self._scrobble_reported = True
                with contextlib.suppress(Exception):
                    self.client.scrobble(self.player.current_song.id)
        except Exception:
            pass

    def _stop_playback(self):
        """Stop playback and clear display."""
        self.player.stop()
        np = self.query_one("#now-playing", NowPlaying)
        np.clear_track()
        try:
            lyrics = self.query_one("#lyrics-panel", LyricsPanel)
            lyrics.clear_lyrics()
        except Exception:
            pass

    def _update_queue_display(self):
        """Refresh the queue view widget."""
        try:
            qv = self.query_one("#queue-view", QueueView)
            qv.update_queue(self.queue_mgr.queue, self.queue_mgr.current_index)
        except Exception:
            pass

    def _show_status(self, text: str):
        """Update the status bar."""
        try:
            status = self.query_one("#status-bar", Static)
            status.update(text)
        except Exception:
            pass

    # ─── Navigation History ──────────────────────────────────────

    def _push_nav(self, view_type: str, **kwargs):
        """Push the current view state onto the navigation stack."""
        # Save the currently active tab so we can restore it on back
        active_tab = ""
        try:
            tabs = self.query_one("#browser-tabs", TabbedContent)
            active_tab = tabs.active
        except Exception:
            pass
        self._nav_history.append({"type": view_type, "tab": active_tab, **kwargs})

        # Limit history size to prevent unbounded memory growth
        if len(self._nav_history) > self._max_nav_history:
            self._nav_history.pop(0)  # Remove oldest entry

    def _pop_nav(self) -> dict | None:
        """Pop the last view from the navigation stack."""
        if self._nav_history:
            return self._nav_history.pop()
        return None

    # ─── DataTable Row Selection ─────────────────────────────────

    def on_data_table_row_selected(self, event: DataTable.RowSelected) -> None:  # noqa: PLR0912
        """Handle row selection in data tables."""
        table = event.data_table

        if table.id == "album-list":
            album_list = self.query_one("#album-list", AlbumList)
            album = album_list.get_selected_album()
            if album and self.client:
                self._browse_album(album)

        elif table.id == "artist-list":
            artist_list = self.query_one("#artist-list", ArtistList)
            artist = artist_list.get_selected_artist()
            if artist and self.client:
                self._browse_artist(artist)

        elif table.id in (
            "song-table-main",
            "album-songs",
            "starred-songs",
            "history-songs",
        ):
            if table.id == "song-table-main":
                song_table = self.query_one("#song-table-main", SongTable)
            elif table.id == "starred-songs":
                song_table = self.query_one("#starred-songs", SongTable)
            elif table.id == "history-songs":
                song_table = self.query_one("#history-songs", SongTable)
            else:
                song_table = self.query_one("#album-songs", SongTable)

            song, idx = song_table.get_selected_song()
            if song:
                self._play_song(song, song_table.songs, idx)

        elif table.id == "playlist-list":
            playlist_list = self.query_one("#playlist-list", PlaylistList)
            pl = playlist_list.get_selected_playlist()
            if pl and self.client:
                self._browse_playlist(pl)

        elif table.id == "genre-list":
            genre_list = self.query_one("#genre-list", GenreList)
            genre = genre_list.get_selected_genre()
            if genre and self.client:
                self._browse_genre(genre)

        elif table.id == "queue-table":
            qv = self.query_one("#queue-view", QueueView)
            idx = qv.get_selected_index()
            song = self.queue_mgr.jump_to(idx)
            if song:
                self._play_song(song)

    # ─── Browse Drill-Down ───────────────────────────────────────

    def _browse_album(self, album: Album):
        """Drill into an album to show its songs."""
        if not self.client:
            return
        try:
            # Push current state for back navigation
            self._push_nav("library")

            self._show_status(f"Loading album: {album.name}...")
            _, songs = self.client.get_album(album.id)
            self._current_album_songs = songs

            browser = self.query_one("#library-browser", LibraryBrowser)
            browser.set_breadcrumb("Library", album.artist, album.name)
            browser.set_header(f"{album.name} - {album.artist}")

            try:
                song_table = self.query_one("#song-table-main", SongTable)
                song_table.set_songs(songs)
                tabs = self.query_one("#browser-tabs", TabbedContent)
                tabs.active = "tab-songs"
            except Exception:
                pass

            self._show_status(f"{album.name} - {len(songs)} songs")

        except Exception as e:
            self._show_status(f"Error loading album: {e}")

    def _browse_artist(self, artist: Artist):
        """Drill into an artist to show their top songs and albums."""
        if not self.client:
            return
        try:
            self._push_nav("library")

            self._show_status(f"Loading artist: {artist.name}...")
            _, albums = self.client.get_artist(artist.id)
            browser = self.query_one("#library-browser", LibraryBrowser)
            browser.set_breadcrumb("Library", artist.name)
            browser.set_header(f"{artist.name}")

            album_list = self.query_one("#album-list", AlbumList)
            album_list.set_albums(albums)

            # Load top songs for the artist into the Songs tab
            top_songs = []
            try:
                top_songs = self.client.get_top_songs(artist.name, count=50)
            except Exception as e:
                self._show_status(f"Top songs error: {e}")

            if top_songs:
                song_table = self.query_one("#song-table-main", SongTable)
                song_table.set_songs(top_songs)
                # Switch to Songs tab to show top songs
                tabs = self.query_one("#browser-tabs", TabbedContent)
                tabs.active = "tab-songs"
                browser.set_header(f"{artist.name} — Top Songs")
                self._show_status(
                    f"{artist.name} - {len(top_songs)} top songs | Press 1 for {len(albums)} albums"
                )
            else:
                # No top songs — show albums
                tabs = self.query_one("#browser-tabs", TabbedContent)
                tabs.active = "tab-albums"
                self._show_status(f"{artist.name} - {len(albums)} albums")

        except Exception as e:
            self._show_status(f"Error loading artist: {e}")

    def _browse_playlist(self, playlist: Playlist):
        """Drill into a playlist to show its songs."""
        if not self.client:
            return
        try:
            self._push_nav("library")

            self._show_status(f"Loading playlist: {playlist.name}...")
            _, songs = self.client.get_playlist(playlist.id)
            browser = self.query_one("#library-browser", LibraryBrowser)
            browser.set_breadcrumb("Library", "Playlists", playlist.name)
            browser.set_header(f"{playlist.name}")

            song_table = self.query_one("#song-table-main", SongTable)
            song_table.set_songs(songs)
            tabs = self.query_one("#browser-tabs", TabbedContent)
            tabs.active = "tab-songs"

            self._show_status(f"{playlist.name} - {len(songs)} songs")

        except Exception as e:
            self._show_status(f"Error loading playlist: {e}")

    def _browse_genre(self, genre: Genre):
        """Browse songs by genre."""
        if not self.client:
            return
        try:
            self._push_nav("library")

            self._show_status(f"Loading genre: {genre.name}...")
            songs = self.client.get_songs_by_genre(genre.name, count=50)
            browser = self.query_one("#library-browser", LibraryBrowser)
            browser.set_breadcrumb("Library", "Genres", genre.name)
            browser.set_header(f"{genre.name}")

            song_table = self.query_one("#song-table-main", SongTable)
            song_table.set_songs(songs)
            tabs = self.query_one("#browser-tabs", TabbedContent)
            tabs.active = "tab-songs"

            self._show_status(f"{genre.name} - {len(songs)} songs")

        except Exception as e:
            self._show_status(f"Error loading genre: {e}")

    # ─── Key Binding Actions ─────────────────────────────────────

    def action_toggle_pause(self) -> None:
        if self.player.state == PlaybackState.STOPPED:
            song = self.queue_mgr.current_song
            if song:
                self._play_song(song)
        else:
            self.player.toggle_pause()
            np = self.query_one("#now-playing", NowPlaying)
            np.state = self.player.state

    def action_next_track(self) -> None:
        next_song = self.queue_mgr.next()
        if next_song:
            self._play_song(next_song)
        else:
            self._show_status("End of queue")

    def action_prev_track(self) -> None:
        if self.player.position > 3.0 and self.player.current_song:
            self.player.seek_to(0)
            return
        prev_song = self.queue_mgr.previous()
        if prev_song:
            self._play_song(prev_song)

    def action_stop_playback(self) -> None:
        self._stop_playback()

    def action_seek_forward(self) -> None:
        self.player.seek(5)

    def action_seek_backward(self) -> None:
        self.player.seek(-5)

    def action_seek_forward_long(self) -> None:
        self.player.seek(30)

    def action_seek_backward_long(self) -> None:
        self.player.seek(-30)

    def action_seek_to(self, position: float) -> None:
        """Seek to an absolute position (from clickable seekbar)."""
        self.player.seek_to(position)

    def action_volume_up(self) -> None:
        self.player.volume_up()
        np = self.query_one("#now-playing", NowPlaying)
        np.volume = self.player.volume
        self.config.volume = self.player.volume
        self.config.save()
        self._show_status(f"Volume: {self.player.volume}%")

    def action_volume_down(self) -> None:
        self.player.volume_down()
        np = self.query_one("#now-playing", NowPlaying)
        np.volume = self.player.volume
        self.config.volume = self.player.volume
        self.config.save()
        self._show_status(f"Volume: {self.player.volume}%")

    def action_mute_toggle(self) -> None:
        self.player.mute_toggle()
        np = self.query_one("#now-playing", NowPlaying)
        np.muted = self.player.muted

    def action_toggle_shuffle(self) -> None:
        self.queue_mgr.toggle_shuffle()
        np = self.query_one("#now-playing", NowPlaying)
        np.shuffle_on = self.queue_mgr.shuffle
        self.config.shuffle = self.queue_mgr.shuffle
        self.config.save()
        self._update_queue_display()
        self._show_status(f"Shuffle {'ON' if self.queue_mgr.shuffle else 'OFF'}")

    def action_cycle_repeat(self) -> None:
        mode = self.queue_mgr.cycle_repeat()
        np = self.query_one("#now-playing", NowPlaying)
        np.repeat_mode = mode.label
        self.config.repeat_mode = mode.value
        self.config.save()
        self._show_status(f"Repeat: {mode.label}")

    def action_add_to_queue(self) -> None:
        """Add currently highlighted song(s) to queue."""
        try:
            song_table = self.query_one("#song-table-main", SongTable)
            song, _ = song_table.get_selected_song()
            if song:
                self.queue_mgr.add(song)
                self._update_queue_display()
                self._show_status(f"Added '{song.title}' to queue")
                return
        except Exception:
            pass
        # Try starred songs table too
        try:
            starred_table = self.query_one("#starred-songs", SongTable)
            song, _ = starred_table.get_selected_song()
            if song:
                self.queue_mgr.add(song)
                self._update_queue_display()
                self._show_status(f"Added '{song.title}' to queue")
                return
        except Exception:
            pass
        # Try history
        try:
            history_table = self.query_one("#history-songs", SongTable)
            song, _ = history_table.get_selected_song()
            if song:
                self.queue_mgr.add(song)
                self._update_queue_display()
                self._show_status(f"Added '{song.title}' to queue")
                return
        except Exception:
            pass
        self._show_status("No song selected")

    def action_clear_queue(self) -> None:
        self.queue_mgr.clear()
        self._update_queue_display()
        self._show_status("Queue cleared")

    def action_remove_from_queue(self) -> None:
        """Remove the highlighted song from the queue."""
        try:
            qv = self.query_one("#queue-view", QueueView)
            idx = qv.get_selected_index()
            if idx >= 0 and idx < self.queue_mgr.length:
                song = self.queue_mgr.queue[idx]
                self.queue_mgr.remove(idx)
                self._update_queue_display()
                self._show_status(f"Removed '{song.title}' from queue")
            else:
                self._show_status("No queue item selected")
        except Exception:
            self._show_status("No queue item selected")

    def action_queue_move_up(self) -> None:
        """Move the highlighted queue item up."""
        try:
            qv = self.query_one("#queue-view", QueueView)
            idx = qv.get_selected_index()
            if idx > 0:
                self.queue_mgr.move(idx, idx - 1)
                self._update_queue_display()
        except Exception:
            pass

    def action_queue_move_down(self) -> None:
        """Move the highlighted queue item down."""
        try:
            qv = self.query_one("#queue-view", QueueView)
            idx = qv.get_selected_index()
            if idx >= 0 and idx < self.queue_mgr.length - 1:
                self.queue_mgr.move(idx, idx + 1)
                self._update_queue_display()
        except Exception:
            pass

    def action_open_search(self) -> None:
        if not self.client:
            self._show_status("No server connected")
            return

        def on_search_result(result):
            if result is None:
                return
            if isinstance(result, Song):
                self._play_song(result, [result], 0)
            elif isinstance(result, Album):
                self._browse_album(result)
            elif isinstance(result, Artist):
                self._browse_artist(result)

        self.push_screen(SearchModal(self.client), on_search_result)

    def action_toggle_eq(self) -> None:
        try:
            eq = self.query_one("#eq-widget", EqualizerWidget)
            eq.toggle_visibility()
        except Exception:
            pass

    def action_toggle_lyrics(self) -> None:
        """Toggle the lyrics panel."""
        try:
            lyrics = self.query_one("#lyrics-panel", LyricsPanel)
            lyrics.toggle_visibility()
        except Exception:
            pass

    def _get_selected_song_from_table(self) -> Song | None:
        """Get the highlighted song from the currently active tab's table."""
        try:
            tabs = self.query_one("#browser-tabs", TabbedContent)
            active = tabs.active
            table_map = {
                "tab-songs": "song-table-main",
                "tab-starred": "starred-songs",
                "tab-history": "history-songs",
            }
            table_id = table_map.get(active)
            if table_id:
                table = self.query_one(f"#{table_id}", SongTable)
                song, _ = table.get_selected_song()
                if song:
                    return song
        except Exception:
            pass
        return None

    def action_toggle_star(self) -> None:
        """Star/unstar the selected song (from table) or currently playing song."""
        if not self.client:
            self._show_status("No server connected")
            return

        # Prefer the highlighted song in the active table, fall back to playing song
        song = self._get_selected_song_from_table() or self.player.current_song

        if not song:
            self._show_status("No song selected — highlight or play a song first")
            return

        try:
            if song.id in self._starred_ids:
                self.client.unstar(song_id=song.id)
                self._starred_ids.discard(song.id)
                self._show_status(f"Unstarred '{song.title}'")
            else:
                self.client.star(song_id=song.id)
                self._starred_ids.add(song.id)
                self._show_status(f"Starred '{song.title}'")
        except Exception as e:
            self._show_status(f"Star/unstar failed: {e}")
        # Refresh starred tab
        self._refresh_starred()

    def _refresh_starred(self):
        """Reload the starred songs list and update starred IDs."""
        if not self.client:
            return
        try:
            _, _, starred_songs = self.client.get_starred()
            self._starred_ids = {s.id for s in starred_songs}
            starred_table = self.query_one("#starred-songs", SongTable)
            starred_table.set_songs(starred_songs)
            self._show_status(f"Starred: {len(starred_songs)} songs")
        except Exception as e:
            self._show_status(f"Starred refresh error: {e}")

    def action_artist_radio(self) -> None:
        """Start artist radio using similar songs."""
        if not self.client or not self.player.current_song:
            self._show_status("Play a song first to start radio")
            return

        song = self.player.current_song
        self._show_status(f"Loading radio for '{song.artist}'...")

        try:
            similar = self.client.get_similar_songs(song.id, count=50)
            if similar:
                self.queue_mgr.set_queue(similar, 0)
                self._play_song(similar[0])
                self._show_status(f"Radio: {len(similar)} similar songs queued")
            else:
                self._show_status("No similar songs found")
        except Exception as e:
            self._show_status(f"Radio failed: {e}")

    def action_save_playlist(self) -> None:
        """Save the current queue as a playlist (with name prompt)."""
        if not self.client:
            self._show_status("No server connected")
            return
        if self.queue_mgr.is_empty:
            self._show_status("Queue is empty — add songs first")
            return

        current = self.player.current_song
        if current:
            default_name = f"Queue - {current.artist}"
        else:
            default_name = f"Queue ({self.queue_mgr.length} songs)"

        def on_result(name):
            if name is None or not self.client:
                return
            song_ids = [s.id for s in self.queue_mgr.queue]
            try:
                self.client.create_playlist(name, song_ids)
                self._show_status(f"Saved playlist '{name}' ({len(song_ids)} songs)")
                # Refresh playlists
                try:
                    playlists = self.client.get_playlists()
                    playlist_list = self.query_one("#playlist-list", PlaylistList)
                    playlist_list.set_playlists(playlists)
                except Exception:
                    pass
            except Exception as e:
                self._show_status(f"Failed to save playlist: {e}")

        self.push_screen(SavePlaylistModal(default_name), on_result)

    def action_cycle_album_sort(self) -> None:
        """Cycle through album sort types."""
        self._album_sort_index = (self._album_sort_index + 1) % len(ALBUM_SORT_TYPES)
        sort_type, sort_label = ALBUM_SORT_TYPES[self._album_sort_index]

        if not self.client:
            return

        self._show_status(f"Loading albums sorted by: {sort_label}...")

        try:
            albums = self.client.get_album_list(sort_type, size=50)
            album_list = self.query_one("#album-list", AlbumList)
            album_list.set_albums(albums)
            tabs = self.query_one("#browser-tabs", TabbedContent)
            tabs.active = "tab-albums"
            self._show_status(f"Albums sorted by: {sort_label} | Press o to cycle sort")
        except Exception as e:
            self._show_status(f"Error loading albums: {e}")

    def action_go_back(self) -> None:
        """Navigate back in browsing history."""
        nav = self._pop_nav()
        if nav and nav.get("type") == "library":
            restore_tab = nav.get("tab", "")
            self.call_later(self._load_library, restore_tab)
            self._show_status("Back to library")
        elif not nav:
            self._show_status("No navigation history | Press q to quit")

    def action_open_servers(self) -> None:
        def on_server_result(result):
            if result is not None:
                self._connect_to_server()
                np = self.query_one("#now-playing", NowPlaying)
                if self.config.active_server:
                    np.server_name = self.config.active_server.name
                self.call_later(self._reload_app)

        self.push_screen(ServerManagerModal(self.config), on_server_result)

    async def _reload_app(self):
        """Reload the app after server change."""
        if self.config.active_server:
            # Check if welcome screen needs replacing
            try:
                welcome = self.query_one("#welcome")
                await welcome.remove()
                content = self.query_one("#content-area")
                browser = LibraryBrowser(id="library-browser")
                await content.mount(browser, before=self.query_one("#eq-widget"))
            except Exception:
                pass

            self._connect_to_server()
            await self._load_library()

    def action_show_help(self) -> None:
        self.push_screen(HelpModal())

    def action_tab_albums(self) -> None:
        try:
            tabs = self.query_one("#browser-tabs", TabbedContent)
            tabs.active = "tab-albums"
        except Exception:
            pass

    def action_tab_artists(self) -> None:
        try:
            tabs = self.query_one("#browser-tabs", TabbedContent)
            tabs.active = "tab-artists"
        except Exception:
            pass

    def action_tab_songs(self) -> None:
        try:
            tabs = self.query_one("#browser-tabs", TabbedContent)
            tabs.active = "tab-songs"
        except Exception:
            pass

    def action_tab_playlists(self) -> None:
        try:
            tabs = self.query_one("#browser-tabs", TabbedContent)
            tabs.active = "tab-playlists"
        except Exception:
            pass

    def action_tab_genres(self) -> None:
        try:
            tabs = self.query_one("#browser-tabs", TabbedContent)
            tabs.active = "tab-genres"
        except Exception:
            pass

    def action_tab_starred(self) -> None:
        """Switch to the Starred tab and refresh."""
        try:
            tabs = self.query_one("#browser-tabs", TabbedContent)
            tabs.active = "tab-starred"
        except Exception:
            pass
        # Refresh starred songs
        self._refresh_starred()

    def action_tab_history(self) -> None:
        """Switch to History tab and show play history."""
        try:
            history_table = self.query_one("#history-songs", SongTable)
            history_table.set_songs(self._play_history)
            tabs = self.query_one("#browser-tabs", TabbedContent)
            tabs.active = "tab-history"
            if self._play_history:
                self._show_status(f"Play history: {len(self._play_history)} songs")
            else:
                self._show_status("No play history yet — play some songs first")
        except Exception as e:
            self._show_status(f"History error: {e}")

    def action_quit_app(self) -> None:
        # Save state
        self.config.volume = self.player.volume
        self.config.shuffle = self.queue_mgr.shuffle
        self.config.repeat_mode = self.queue_mgr.repeat.value
        self.config.custom_eq_gains = list(self.equalizer.gains)
        self.config.save()

        # Cleanup
        self.player.cleanup()
        self.exit()
