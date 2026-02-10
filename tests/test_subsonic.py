#!/usr/bin/env python3
"""
Comprehensive tests for Subsonic API client
"""

import os
import sys
import unittest
from unittest.mock import MagicMock, Mock, patch

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "src"))

from cli_music_player.subsonic import (
    Album,
    Artist,
    ConnectionError,
    Genre,
    Playlist,
    Song,
    SubsonicClient,
    SubsonicError,
)


class TestSubsonicDataModels(unittest.TestCase):
    """Test Subsonic data model classes"""

    def test_song_from_api(self):
        """Test Song creation from API data"""
        data = {
            "id": "123",
            "title": "Test Song",
            "artist": "Test Artist",
            "album": "Test Album",
            "duration": 180,
            "track": 1,
            "year": 2023,
            "genre": "Rock",
            "coverArt": "cover123",
        }
        song = Song.from_api(data)

        self.assertEqual(song.id, "123")
        self.assertEqual(song.title, "Test Song")
        self.assertEqual(song.artist, "Test Artist")
        self.assertEqual(song.duration, 180)
        self.assertEqual(song.track, 1)
        self.assertEqual(song.year, 2023)

    def test_song_from_api_missing_fields(self):
        """Test Song handles missing optional fields"""
        data = {"id": "123"}
        song = Song.from_api(data)

        self.assertEqual(song.title, "Unknown")
        self.assertEqual(song.artist, "Unknown Artist")
        self.assertEqual(song.album, "Unknown Album")
        self.assertEqual(song.duration, 0)

    def test_album_from_api(self):
        """Test Album creation from API data"""
        data = {
            "id": "456",
            "name": "Test Album",
            "artist": "Test Artist",
            "songCount": 12,
            "duration": 2400,
            "year": 2023,
        }
        album = Album.from_api(data)

        self.assertEqual(album.id, "456")
        self.assertEqual(album.name, "Test Album")
        self.assertEqual(album.artist, "Test Artist")
        self.assertEqual(album.song_count, 12)

    def test_artist_from_api(self):
        """Test Artist creation from API data"""
        data = {"id": "789", "name": "Test Artist", "albumCount": 5}
        artist = Artist.from_api(data)

        self.assertEqual(artist.id, "789")
        self.assertEqual(artist.name, "Test Artist")
        self.assertEqual(artist.album_count, 5)

    def test_playlist_from_api(self):
        """Test Playlist creation from API data"""
        data = {
            "id": "pl1",
            "name": "My Playlist",
            "songCount": 20,
            "duration": 4800,
            "owner": "testuser",
            "public": True,
        }
        playlist = Playlist.from_api(data)

        self.assertEqual(playlist.id, "pl1")
        self.assertEqual(playlist.name, "My Playlist")
        self.assertEqual(playlist.song_count, 20)
        self.assertTrue(playlist.public)

    def test_genre_from_api(self):
        """Test Genre creation from API data"""
        data = {"value": "Rock", "songCount": 150, "albumCount": 25}
        genre = Genre.from_api(data)

        self.assertEqual(genre.name, "Rock")
        self.assertEqual(genre.song_count, 150)
        self.assertEqual(genre.album_count, 25)


class TestSubsonicClient(unittest.TestCase):
    """Test SubsonicClient functionality"""

    @patch("cli_music_player.subsonic.requests.Session")
    def test_client_initialization(self, mock_session_class):
        """Test client initialization"""
        mock_session = Mock()
        mock_session_class.return_value = mock_session

        client = SubsonicClient("http://test.com", "user", "pass")

        self.assertEqual(client.base_url, "http://test.com")
        self.assertEqual(client.username, "user")
        self.assertEqual(client.password, "pass")
        mock_session_class.assert_called_once()

    @patch("cli_music_player.subsonic.requests.Session")
    def test_auth_params_generation(self, mock_session_class):
        """Test authentication parameter generation"""
        mock_session = Mock()
        mock_session_class.return_value = mock_session

        client = SubsonicClient("http://test.com", "user", "pass")
        params = client._auth_params()

        self.assertIn("u", params)
        self.assertIn("t", params)
        self.assertIn("s", params)
        self.assertIn("v", params)
        self.assertIn("c", params)
        self.assertIn("f", params)

        self.assertEqual(params["u"], "user")
        self.assertEqual(params["f"], "json")

    @patch("cli_music_player.subsonic.requests.Session")
    def test_request_success(self, mock_session_class):
        """Test successful API request"""
        mock_session = Mock()
        mock_response = Mock()
        mock_response.json.return_value = {
            "subsonic-response": {"status": "ok", "version": "1.16.0"}
        }
        mock_session.get.return_value = mock_response
        mock_session_class.return_value = mock_session

        client = SubsonicClient("http://test.com", "user", "pass")
        result = client._request("ping")

        self.assertEqual(result["status"], "ok")
        mock_session.get.assert_called_once()

    @patch("cli_music_player.subsonic.requests.Session")
    def test_request_api_error(self, mock_session_class):
        """Test API error response"""
        mock_session = Mock()
        mock_response = Mock()
        mock_response.json.return_value = {
            "subsonic-response": {
                "status": "failed",
                "error": {"code": 40, "message": "Wrong username or password"},
            }
        }
        mock_session.get.return_value = mock_response
        mock_session_class.return_value = mock_session

        client = SubsonicClient("http://test.com", "user", "pass")

        with self.assertRaises(SubsonicError) as context:
            client._request("ping")

        self.assertIn("Wrong username or password", str(context.exception))

    @patch("cli_music_player.subsonic.requests.Session")
    def test_request_timeout(self, mock_session_class):
        """Test request timeout handling"""
        import requests

        mock_session = Mock()
        mock_session.get.side_effect = requests.exceptions.Timeout("Timeout")
        mock_session_class.return_value = mock_session

        client = SubsonicClient("http://test.com", "user", "pass")

        with self.assertRaises(ConnectionError) as context:
            client._request("ping")

        self.assertIn("timed out", str(context.exception))

    @patch("cli_music_player.subsonic.requests.Session")
    def test_ping(self, mock_session_class):
        """Test ping endpoint"""
        mock_session = Mock()
        mock_response = Mock()
        mock_response.json.return_value = {"subsonic-response": {"status": "ok"}}
        mock_session.get.return_value = mock_response
        mock_session_class.return_value = mock_session

        client = SubsonicClient("http://test.com", "user", "pass")
        result = client.ping()

        self.assertTrue(result)

    @patch("cli_music_player.subsonic.requests.Session")
    def test_stream_url(self, mock_session_class):
        """Test stream URL generation"""
        mock_session = Mock()
        mock_session_class.return_value = mock_session

        client = SubsonicClient("http://test.com", "user", "pass")
        url = client.stream_url("song123")

        self.assertIn("http://test.com/rest/stream", url)
        self.assertIn("id=song123", url)


class TestSubsonicAPIMethods(unittest.TestCase):
    """Test Subsonic API method calls"""

    def setUp(self):
        self.patcher = patch("cli_music_player.subsonic.requests.Session")
        self.mock_session_class = self.patcher.start()
        self.mock_session = Mock()
        self.mock_session_class.return_value = self.mock_session
        self.client = SubsonicClient("http://test.com", "user", "pass")

    def tearDown(self):
        self.patcher.stop()

    def _mock_response(self, data):
        """Helper to create mock response"""
        mock_response = Mock()
        mock_response.json.return_value = {
            "subsonic-response": {"status": "ok", **data}
        }
        self.mock_session.get.return_value = mock_response

    def test_get_album_list(self):
        """Test getting albums"""
        self._mock_response(
            {
                "albumList2": {
                    "album": [
                        {"id": "1", "name": "Album 1", "artist": "Artist 1"},
                        {"id": "2", "name": "Album 2", "artist": "Artist 2"},
                    ]
                }
            }
        )

        albums = self.client.get_album_list()

        self.assertEqual(len(albums), 2)
        self.assertEqual(albums[0].name, "Album 1")
        self.assertEqual(albums[1].name, "Album 2")

    def test_get_artists(self):
        """Test getting artists"""
        self._mock_response(
            {
                "artists": {
                    "index": [
                        {
                            "artist": [
                                {"id": "1", "name": "Artist A"},
                                {"id": "2", "name": "Artist B"},
                            ]
                        }
                    ]
                }
            }
        )

        artists = self.client.get_artists()

        self.assertEqual(len(artists), 2)
        self.assertEqual(artists[0].name, "Artist A")

    def test_search(self):
        """Test search functionality"""
        self._mock_response(
            {
                "searchResult3": {
                    "artist": [{"id": "1", "name": "Test Artist"}],
                    "album": [
                        {"id": "2", "name": "Test Album", "artist": "Test Artist"}
                    ],
                    "song": [
                        {
                            "id": "3",
                            "title": "Test Song",
                            "artist": "Test Artist",
                            "album": "Test Album",
                        }
                    ],
                }
            }
        )

        artists, albums, songs = self.client.search("test")

        self.assertEqual(len(artists), 1)
        self.assertEqual(len(albums), 1)
        self.assertEqual(len(songs), 1)
        self.assertEqual(artists[0].name, "Test Artist")


if __name__ == "__main__":
    unittest.main()
