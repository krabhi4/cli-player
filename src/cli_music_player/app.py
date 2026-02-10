"""Main Textual Application — orchestrates all components."""

import threading
from pathlib import Path
from typing import Optional

from textual.app import App, ComposeResult
from textual.binding import Binding
from textual.containers import Horizontal, Vertical
from textual.widgets import DataTable, Footer, Header, Static, Label, TabbedContent

from .config import AppConfig
from .equalizer import Equalizer
from .player import Player, PlaybackState
from .queue import QueueManager, RepeatMode
from .subsonic import (
    SubsonicClient,
    Song,
    Album,
    Artist,
    Playlist,
    Genre,
    SubsonicError,
)
from .widgets.browser import (
    LibraryBrowser,
    SongTable,
    AlbumList,
    ArtistList,
    PlaylistList,
    GenreList,
    SongSelected,
    AddToQueue,
)
from .widgets.equalizer import EqualizerWidget
from .widgets.help import HelpModal
from .widgets.now_playing import NowPlaying, SeekBar, ControlBtn
from .widgets.queue_view import QueueView
from .widgets.search import SearchModal
from .widgets.server_mgr import ServerManagerModal


class MusicPlayerApp(App):
    """CLI Music Player for Navidrome."""

    TITLE = "🎵 CLI Music Player"
    SUB_TITLE = "Navidrome Terminal Client"

    CSS_PATH = "styles/app.tcss"

    BINDINGS = [
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
        Binding("slash", "open_search", "Search", show=True),
        Binding("e", "toggle_eq", "EQ", show=True),
        Binding("f", "toggle_star", "Star", show=False),
        Binding("S", "open_servers", "Servers", show=True),
        Binding("question_mark", "show_help", "Help", show=True),
        Binding("i", "show_help", "Info", show=False),
        Binding("1", "tab_albums", "Albums", show=False),
        Binding("2", "tab_artists", "Artists", show=False),
        Binding("3", "tab_songs", "Songs", show=False),
        Binding("4", "tab_playlists", "Playlists", show=False),
        Binding("5", "tab_genres", "Genres", show=False),
        Binding("q", "quit_app", "Quit", show=True),
    ]

    def __init__(self):
        super().__init__()
        self.config = AppConfig()
        self.player = Player(audio_device=self.config.audio_device)
        self.equalizer = Equalizer(self.config)
        self.queue_mgr = QueueManager()
        self.client: Optional[SubsonicClient] = None

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

    def compose(self) -> ComposeResult:
        yield Static("🎵 CLI Music Player", id="app-header")
        with Horizontal(id="main-area"):
            with Vertical(id="content-area"):
                if self.config.active_server is None:
                    # Welcome screen — no server configured
                    with Vertical(id="welcome"):
                        with Vertical(id="welcome-box"):
                            yield Static(
                                "🎵 CLI Music Player", classes="welcome-title"
                            )
                            yield Static(
                                "\nNo servers configured yet.\n"
                                "Press [bold]S[/bold] to add a Navidrome server.\n",
                                classes="welcome-text",
                            )
                else:
                    yield LibraryBrowser(id="library-browser")
                yield EqualizerWidget(self.equalizer, id="eq-widget")
            yield QueueView(id="queue-view")
        yield NowPlaying(id="now-playing")
        yield Static(
            "? Help | Space Play/Pause | n Next | p Prev | / Search | S Servers | q Quit",
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

    async def _load_library(self):
        """Load initial library data."""
        if not self.client:
            return

        try:
            # Load albums (newest first)
            albums = self.client.get_album_list("newest", size=50)
            try:
                album_list = self.query_one("#album-list", AlbumList)
                album_list.set_albums(albums)
            except Exception:
                pass

            # Load artists
            artists = self.client.get_artists()
            try:
                artist_list = self.query_one("#artist-list", ArtistList)
                artist_list.set_artists(artists)
            except Exception:
                pass

            # Load random songs for the Songs tab
            songs = self.client.get_random_songs(size=50)
            try:
                song_table = self.query_one("#song-table-main", SongTable)
                song_table.set_songs(songs)
            except Exception:
                pass

            # Load playlists
            playlists = self.client.get_playlists()
            try:
                playlist_list = self.query_one("#playlist-list", PlaylistList)
                playlist_list.set_playlists(playlists)
            except Exception:
                pass

            # Load genres
            genres = self.client.get_genres()
            try:
                genre_list = self.query_one("#genre-list", GenreList)
                genre_list.set_genres(genres)
            except Exception:
                pass

            # Update browser header
            try:
                browser = self.query_one("#library-browser", LibraryBrowser)
                browser.set_header(
                    f"📚 {self.config.active_server.name} — Library"
                )
                browser.set_breadcrumb("Library")
            except Exception:
                pass

        except Exception as e:
            self._show_status(f"Error loading library: {e}")

    # ─── Playback Actions ────────────────────────────────────────

    def _play_song(self, song: Song, queue_songs: Optional[list[Song]] = None, start_index: int = 0):
        """Play a song and optionally set up the queue."""
        if not self.client:
            self._show_status("No server connected")
            return

        if queue_songs:
            self.queue_mgr.set_queue(queue_songs, start_index)

        url = self.client.stream_url(song.id)
        self.player.play(url, song)
        self._scrobble_reported = False

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
        try:
            self.client.now_playing(song.id)
        except Exception:
            pass

        # Update queue view
        self._update_queue_display()

    def _on_track_end(self):
        """Called when the current track finishes."""
        # Scrobble
        if self.client and self.player.current_song:
            try:
                self.client.scrobble(self.player.current_song.id)
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
            ):
                if (duration > 0 and position / duration > 0.5) or position > 240:
                    self._scrobble_reported = True
                    try:
                        self.client.scrobble(self.player.current_song.id)
                    except Exception:
                        pass
        except Exception:
            pass

    def _stop_playback(self):
        """Stop playback and clear display."""
        self.player.stop()
        np = self.query_one("#now-playing", NowPlaying)
        np.clear_track()

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

    # ─── DataTable Row Selection ─────────────────────────────────

    def on_data_table_row_selected(self, event: DataTable.RowSelected) -> None:
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

        elif table.id in ("song-table-main", "album-songs"):
            # Get the song table
            if table.id == "song-table-main":
                song_table = self.query_one("#song-table-main", SongTable)
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
            if idx >= 0 and idx < self.queue_mgr.length:
                self.queue_mgr._current_index = idx
                song = self.queue_mgr.current_song
                if song:
                    self._play_song(song)

    # ─── Browse Drill-Down ───────────────────────────────────────

    def _browse_album(self, album: Album):
        """Drill into an album to show its songs."""
        if not self.client:
            return
        try:
            _, songs = self.client.get_album(album.id)
            self._current_album_songs = songs

            # Replace the album tab content with a song table
            browser = self.query_one("#library-browser", LibraryBrowser)
            browser.set_breadcrumb("Library", album.artist, album.name)
            browser.set_header(f"💿 {album.name} — {album.artist}")

            # Switch to songs tab and show album songs
            try:
                song_table = self.query_one("#song-table-main", SongTable)
                song_table.set_songs(songs)
                tabs = self.query_one("#browser-tabs", TabbedContent)
                tabs.active = "tab-songs"
            except Exception:
                pass

        except Exception as e:
            self._show_status(f"Error: {e}")

    def _browse_artist(self, artist: Artist):
        """Drill into an artist to show their albums."""
        if not self.client:
            return
        try:
            _, albums = self.client.get_artist(artist.id)
            browser = self.query_one("#library-browser", LibraryBrowser)
            browser.set_breadcrumb("Library", artist.name)
            browser.set_header(f"🎤 {artist.name}")

            album_list = self.query_one("#album-list", AlbumList)
            album_list.set_albums(albums)
            tabs = self.query_one("#browser-tabs", TabbedContent)
            tabs.active = "tab-albums"

        except Exception as e:
            self._show_status(f"Error: {e}")

    def _browse_playlist(self, playlist: Playlist):
        """Drill into a playlist to show its songs."""
        if not self.client:
            return
        try:
            _, songs = self.client.get_playlist(playlist.id)
            browser = self.query_one("#library-browser", LibraryBrowser)
            browser.set_breadcrumb("Library", "Playlists", playlist.name)
            browser.set_header(f"📋 {playlist.name}")

            song_table = self.query_one("#song-table-main", SongTable)
            song_table.set_songs(songs)
            tabs = self.query_one("#browser-tabs", TabbedContent)
            tabs.active = "tab-songs"

        except Exception as e:
            self._show_status(f"Error: {e}")

    def _browse_genre(self, genre: Genre):
        """Browse songs by genre."""
        if not self.client:
            return
        try:
            songs = self.client.get_songs_by_genre(genre.name, count=50)
            browser = self.query_one("#library-browser", LibraryBrowser)
            browser.set_breadcrumb("Library", "Genres", genre.name)
            browser.set_header(f"🎸 {genre.name}")

            song_table = self.query_one("#song-table-main", SongTable)
            song_table.set_songs(songs)
            tabs = self.query_one("#browser-tabs", TabbedContent)
            tabs.active = "tab-songs"

        except Exception as e:
            self._show_status(f"Error: {e}")

    # ─── Key Binding Actions ─────────────────────────────────────

    def action_toggle_pause(self) -> None:
        if self.player.state == PlaybackState.STOPPED:
            # If stopped but queue has items, play current
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
        # If more than 3 seconds in, restart current song
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
        self._show_status(f"🔊 Volume: {self.player.volume}%")

    def action_volume_down(self) -> None:
        self.player.volume_down()
        np = self.query_one("#now-playing", NowPlaying)
        np.volume = self.player.volume
        self.config.volume = self.player.volume
        self.config.save()
        self._show_status(f"🔉 Volume: {self.player.volume}%")

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
        self._show_status(
            f"Shuffle {'ON' if self.queue_mgr.shuffle else 'OFF'}"
        )

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
        except Exception:
            self._show_status("No song selected")

    def action_clear_queue(self) -> None:
        self.queue_mgr.clear()
        self._update_queue_display()
        self._show_status("Queue cleared")

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

    def action_toggle_star(self) -> None:
        """Star/unstar the currently playing song."""
        if not self.client or not self.player.current_song:
            return
        song = self.player.current_song
        try:
            self.client.star(song_id=song.id)
            self._show_status(f"★ Starred '{song.title}'")
        except Exception:
            try:
                self.client.unstar(song_id=song.id)
                self._show_status(f"☆ Unstarred '{song.title}'")
            except Exception as e:
                self._show_status(f"Error: {e}")

    def action_open_servers(self) -> None:
        def on_server_result(result):
            if result is not None:
                self._connect_to_server()
                np = self.query_one("#now-playing", NowPlaying)
                if self.config.active_server:
                    np.server_name = self.config.active_server.name
                self.call_later(self._reload_app)

        self.push_screen(
            ServerManagerModal(self.config), on_server_result
        )

    async def _reload_app(self):
        """Reload the app after server change."""
        if self.config.active_server:
            # Check if welcome screen needs replacing
            try:
                welcome = self.query_one("#welcome")
                # Remove welcome and add browser
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
