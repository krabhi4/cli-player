"""Subsonic API client for Navidrome communication."""

import hashlib
import secrets
from dataclasses import dataclass
from typing import Any

import requests

API_VERSION = "1.16.1"
CLIENT_NAME = "CLIMusicPlayer"


@dataclass
class Song:
    """Represents a song from the Subsonic API."""

    id: str
    title: str
    artist: str = "Unknown Artist"
    album: str = "Unknown Album"
    album_id: str = ""
    artist_id: str = ""
    duration: int = 0  # seconds
    track: int = 0
    year: int = 0
    genre: str = ""
    size: int = 0  # bytes
    suffix: str = ""  # file extension
    bitrate: int = 0  # kbps
    cover_art_id: str = ""

    @classmethod
    def from_api(cls, data: dict) -> "Song":
        return cls(
            id=str(data.get("id", "")),
            title=data.get("title", "Unknown"),
            artist=data.get("artist", "Unknown Artist"),
            album=data.get("album", "Unknown Album"),
            album_id=str(data.get("albumId", "")),
            artist_id=str(data.get("artistId", "")),
            duration=data.get("duration", 0),
            track=data.get("track", 0),
            year=data.get("year", 0),
            genre=data.get("genre", ""),
            size=data.get("size", 0),
            suffix=data.get("suffix", ""),
            bitrate=data.get("bitRate", 0),
            cover_art_id=str(data.get("coverArt", "")),
        )


@dataclass
class Album:
    """Represents an album from the Subsonic API."""

    id: str
    name: str
    artist: str = "Unknown Artist"
    artist_id: str = ""
    song_count: int = 0
    duration: int = 0
    year: int = 0
    genre: str = ""
    cover_art_id: str = ""

    @classmethod
    def from_api(cls, data: dict) -> "Album":
        return cls(
            id=str(data.get("id", "")),
            name=data.get("name", data.get("album", "Unknown Album")),
            artist=data.get("artist", "Unknown Artist"),
            artist_id=str(data.get("artistId", "")),
            song_count=data.get("songCount", 0),
            duration=data.get("duration", 0),
            year=data.get("year", 0),
            genre=data.get("genre", ""),
            cover_art_id=str(data.get("coverArt", "")),
        )


@dataclass
class Artist:
    """Represents an artist from the Subsonic API."""

    id: str
    name: str
    album_count: int = 0
    cover_art_id: str = ""

    @classmethod
    def from_api(cls, data: dict) -> "Artist":
        return cls(
            id=str(data.get("id", "")),
            name=data.get("name", "Unknown Artist"),
            album_count=data.get("albumCount", 0),
            cover_art_id=str(data.get("coverArt", data.get("artistImageUrl", ""))),
        )


@dataclass
class Playlist:
    """Represents a playlist from the Subsonic API."""

    id: str
    name: str
    song_count: int = 0
    duration: int = 0
    owner: str = ""
    public: bool = False

    @classmethod
    def from_api(cls, data: dict) -> "Playlist":
        return cls(
            id=str(data.get("id", "")),
            name=data.get("name", "Unknown Playlist"),
            song_count=data.get("songCount", 0),
            duration=data.get("duration", 0),
            owner=data.get("owner", ""),
            public=data.get("public", False),
        )


@dataclass
class Genre:
    """Represents a genre from the Subsonic API."""

    name: str
    song_count: int = 0
    album_count: int = 0

    @classmethod
    def from_api(cls, data: dict) -> "Genre":
        return cls(
            name=data.get("value", "Unknown"),
            song_count=data.get("songCount", 0),
            album_count=data.get("albumCount", 0),
        )


class SubsonicError(Exception):
    """Error from the Subsonic API."""

    def __init__(self, code: int, message: str):
        self.code = code
        super().__init__(f"Subsonic error {code}: {message}")


class ConnectionError(Exception):
    """Cannot connect to the Navidrome server."""


class SubsonicClient:
    """Client for the Subsonic REST API used by Navidrome."""

    def __init__(self, base_url: str, username: str, password: str):
        self.base_url = base_url.rstrip("/")
        self.username = username
        self.password = password
        self.session = requests.Session()
        self._timeout = 15

    def close(self):
        """Close the HTTP session and release resources."""
        if self.session:
            self.session.close()

    def __del__(self):
        """Cleanup when the client is destroyed."""
        self.close()

    def _auth_params(self) -> dict:
        """Generate authentication parameters using token+salt method."""
        salt = secrets.token_hex(16)
        token = hashlib.md5((self.password + salt).encode()).hexdigest()
        return {
            "u": self.username,
            "t": token,
            "s": salt,
            "v": API_VERSION,
            "c": CLIENT_NAME,
            "f": "json",
        }

    def _request(self, endpoint: str, **params) -> dict:
        """Make an API request and return the parsed response."""
        url = f"{self.base_url}/rest/{endpoint}"
        all_params = self._auth_params()
        all_params.update(params)

        try:
            # Explicitly pass timeout to enforce it on this request
            resp = self.session.get(url, params=all_params, timeout=self._timeout)
            resp.raise_for_status()
        except requests.exceptions.Timeout as e:
            raise ConnectionError(f"Request timed out after {self._timeout}s: {e}") from e
        except requests.exceptions.ConnectionError as e:
            raise ConnectionError(f"Cannot connect to {self.base_url}: {e}") from e
        except requests.exceptions.RequestException as e:
            raise ConnectionError(f"Request failed: {e}") from e

        data = resp.json()
        sub_response = data.get("subsonic-response", {})

        if sub_response.get("status") == "failed":
            error = sub_response.get("error", {})
            raise SubsonicError(error.get("code", -1), error.get("message", "Unknown error"))

        return dict(sub_response)  # Type narrowing

    def _stream_url(self, song_id: str, fmt: str = "raw") -> str:
        """Build a streaming URL for a song (for mpv to consume)."""
        params = self._auth_params()
        params["id"] = song_id
        params["format"] = fmt
        query = "&".join(f"{k}={v}" for k, v in params.items())
        return f"{self.base_url}/rest/stream.view?{query}"

    def _cover_art_url(self, cover_id: str, size: int = 256) -> str:
        """Build a cover art URL."""
        if not cover_id:
            return ""
        params = self._auth_params()
        params["id"] = cover_id
        params["size"] = str(size)
        query = "&".join(f"{k}={v}" for k, v in params.items())
        return f"{self.base_url}/rest/getCoverArt.view?{query}"

    # ─── Connection ──────────────────────────────────────────────────

    def ping(self) -> bool:
        """Test the connection to the server."""
        try:
            resp = self._request("ping.view")
            return resp.get("status") == "ok"
        except Exception:
            return False

    # ─── Browsing ────────────────────────────────────────────────────

    def get_artists(self) -> list[Artist]:
        """Get all artists (indexed)."""
        resp = self._request("getArtists.view")
        artists_data = resp.get("artists", {})
        result: list[Artist] = []
        for index in artists_data.get("index", []):
            for artist_data in index.get("artist", []):
                result.append(Artist.from_api(artist_data))
        return result

    def get_artist(self, artist_id: str) -> tuple[Artist, list[Album]]:
        """Get artist details and their albums."""
        resp = self._request("getArtist.view", id=artist_id)
        artist_data = resp.get("artist", {})
        artist = Artist.from_api(artist_data)
        albums = [Album.from_api(a) for a in artist_data.get("album", [])]
        return artist, albums

    def get_album(self, album_id: str) -> tuple[Album, list[Song]]:
        """Get album details and its songs."""
        resp = self._request("getAlbum.view", id=album_id)
        album_data = resp.get("album", {})
        album = Album.from_api(album_data)
        songs = [Song.from_api(s) for s in album_data.get("song", [])]
        return album, songs

    def get_album_list(
        self,
        list_type: str = "newest",
        size: int = 50,
        offset: int = 0,
        **extra_params,
    ) -> list[Album]:
        """Get a list of albums.

        list_type: newest, random, frequent, recent, starred,
                   alphabeticalByName, alphabeticalByArtist, byYear, byGenre
        """
        params = {"type": list_type, "size": size, "offset": offset}
        params.update(extra_params)
        resp = self._request("getAlbumList2.view", **params)
        album_list = resp.get("albumList2", {})
        return [Album.from_api(a) for a in album_list.get("album", [])]

    def get_song(self, song_id: str) -> Song:
        """Get details for a single song."""
        resp = self._request("getSong.view", id=song_id)
        return Song.from_api(resp.get("song", {}))

    def get_random_songs(self, size: int = 50, genre: str = "") -> list[Song]:
        """Get random songs."""
        params: dict[str, Any] = {"size": size}
        if genre:
            params["genre"] = genre
        resp = self._request("getRandomSongs.view", **params)
        songs_data = resp.get("randomSongs", {})
        return [Song.from_api(s) for s in songs_data.get("song", [])]

    # ─── Search ──────────────────────────────────────────────────────

    def search(
        self,
        query: str,
        artist_count: int = 10,
        album_count: int = 10,
        song_count: int = 20,
    ) -> tuple[list[Artist], list[Album], list[Song]]:
        """Search for artists, albums, and songs."""
        resp = self._request(
            "search3.view",
            query=query,
            artistCount=artist_count,
            albumCount=album_count,
            songCount=song_count,
        )
        results = resp.get("searchResult3", {})
        artists = [Artist.from_api(a) for a in results.get("artist", [])]
        albums = [Album.from_api(a) for a in results.get("album", [])]
        songs = [Song.from_api(s) for s in results.get("song", [])]
        return artists, albums, songs

    # ─── Genres ──────────────────────────────────────────────────────

    def get_genres(self) -> list[Genre]:
        """Get all genres."""
        resp = self._request("getGenres.view")
        genres_data = resp.get("genres", {})
        return [Genre.from_api(g) for g in genres_data.get("genre", [])]

    def get_songs_by_genre(self, genre: str, count: int = 50, offset: int = 0) -> list[Song]:
        """Get songs by genre."""
        resp = self._request(
            "getSongsByGenre.view",
            genre=genre,
            count=count,
            offset=offset,
        )
        songs_data = resp.get("songsByGenre", {})
        return [Song.from_api(s) for s in songs_data.get("song", [])]

    # ─── Playlists ───────────────────────────────────────────────────

    def get_playlists(self) -> list[Playlist]:
        """Get all playlists."""
        resp = self._request("getPlaylists.view")
        playlist_data = resp.get("playlists", {})
        return [Playlist.from_api(p) for p in playlist_data.get("playlist", [])]

    def get_playlist(self, playlist_id: str) -> tuple[Playlist, list[Song]]:
        """Get a playlist and its songs."""
        resp = self._request("getPlaylist.view", id=playlist_id)
        pl_data = resp.get("playlist", {})
        playlist = Playlist.from_api(pl_data)
        songs = [Song.from_api(s) for s in pl_data.get("entry", [])]
        return playlist, songs

    def create_playlist(
        self,
        name: str,
        song_ids: list[str] | None = None,
    ) -> str:
        """Create a new playlist. Returns playlist ID."""
        params: dict[str, Any] = {"name": name}
        if song_ids:
            params["songId"] = song_ids
        resp = self._request("createPlaylist.view", **params)
        pl = resp.get("playlist", {})
        return str(pl.get("id", ""))

    # ─── Starred / Favourites ────────────────────────────────────────

    def get_starred(self) -> tuple[list[Artist], list[Album], list[Song]]:
        """Get all starred items."""
        resp = self._request("getStarred2.view")
        starred = resp.get("starred2", {})
        artists = [Artist.from_api(a) for a in starred.get("artist", [])]
        albums = [Album.from_api(a) for a in starred.get("album", [])]
        songs = [Song.from_api(s) for s in starred.get("song", [])]
        return artists, albums, songs

    def star(self, song_id: str = "", album_id: str = "", artist_id: str = ""):
        """Star an item."""
        params: dict[str, str] = {}
        if song_id:
            params["id"] = song_id
        if album_id:
            params["albumId"] = album_id
        if artist_id:
            params["artistId"] = artist_id
        self._request("star.view", **params)

    def unstar(self, song_id: str = "", album_id: str = "", artist_id: str = ""):
        """Unstar an item."""
        params: dict[str, str] = {}
        if song_id:
            params["id"] = song_id
        if album_id:
            params["albumId"] = album_id
        if artist_id:
            params["artistId"] = artist_id
        self._request("unstar.view", **params)

    # ─── Scrobbling ──────────────────────────────────────────────────

    def scrobble(self, song_id: str, submission: bool = True):
        """Scrobble a song (report it as played)."""
        self._request(
            "scrobble.view",
            id=song_id,
            submission=str(submission).lower(),
        )

    def now_playing(self, song_id: str):
        """Report a song as currently playing (non-submission scrobble)."""
        self.scrobble(song_id, submission=False)

    # ─── Lyrics ────────────────────────────────────────────────────────

    def get_lyrics(self, artist: str = "", title: str = "") -> str:
        """Get lyrics for a song."""
        params: dict[str, str] = {}
        if artist:
            params["artist"] = artist
        if title:
            params["title"] = title
        resp = self._request("getLyrics.view", **params)
        lyrics_data = resp.get("lyrics", {})
        return str(lyrics_data.get("value", ""))

    # ─── Similar / Discovery ─────────────────────────────────────────

    def get_similar_songs(self, song_id: str, count: int = 50) -> list[Song]:
        """Get songs similar to a given song."""
        resp = self._request("getSimilarSongs2.view", id=song_id, count=count)
        similar = resp.get("similarSongs2", {})
        return [Song.from_api(s) for s in similar.get("song", [])]

    def get_top_songs(self, artist_name: str, count: int = 50) -> list[Song]:
        """Get top songs for an artist (via Last.fm data)."""
        resp = self._request("getTopSongs.view", artist=artist_name, count=count)
        top = resp.get("topSongs", {})
        return [Song.from_api(s) for s in top.get("song", [])]

    # ─── URL Builders (for external use) ─────────────────────────────

    def stream_url(self, song_id: str) -> str:
        """Get the streaming URL for a song."""
        return self._stream_url(song_id)

    def cover_art_url(self, cover_id: str, size: int = 256) -> str:
        """Get the cover art URL."""
        return self._cover_art_url(cover_id, size)
