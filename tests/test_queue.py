#!/usr/bin/env python3
"""
Comprehensive tests for Queue Manager
"""

import os
import sys
import unittest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "src"))

from cli_music_player.queue import QueueManager, RepeatMode
from cli_music_player.subsonic import Song


class TestRepeatMode(unittest.TestCase):
    """Test RepeatMode enum"""

    def test_repeat_mode_cycle(self):
        """Test cycling through repeat modes"""
        mode = RepeatMode.OFF
        mode = mode.next()
        self.assertEqual(mode, RepeatMode.ALL)

        mode = mode.next()
        self.assertEqual(mode, RepeatMode.ONE)

        mode = mode.next()
        self.assertEqual(mode, RepeatMode.OFF)

    def test_repeat_mode_labels(self):
        """Test repeat mode labels"""
        self.assertEqual(RepeatMode.OFF.label, "Off")
        self.assertEqual(RepeatMode.ALL.label, "All")
        self.assertEqual(RepeatMode.ONE.label, "One")

    def test_repeat_mode_icons(self):
        """Test repeat mode icons"""
        self.assertEqual(RepeatMode.OFF.icon, "🔁")
        self.assertEqual(RepeatMode.ALL.icon, "🔁")
        self.assertEqual(RepeatMode.ONE.icon, "🔂")


class TestQueueManager(unittest.TestCase):
    """Test QueueManager functionality"""

    def setUp(self):
        """Create fresh queue manager for each test"""
        self.queue = QueueManager()
        self.songs = [
            Song(
                id=f"song{i}",
                title=f"Song {i}",
                artist="Test",
                album="Test",
                duration=180,
            )
            for i in range(10)
        ]

    def test_initial_state(self):
        """Test initial queue state"""
        self.assertTrue(self.queue.is_empty)
        self.assertEqual(self.queue.length, 0)
        self.assertEqual(self.queue.current_index, -1)
        self.assertIsNone(self.queue.current_song)
        self.assertFalse(self.queue.shuffle)
        self.assertEqual(self.queue.repeat, RepeatMode.OFF)

    def test_set_queue(self):
        """Test setting queue with songs"""
        self.queue.set_queue(self.songs, 2)

        self.assertFalse(self.queue.is_empty)
        self.assertEqual(self.queue.length, 10)
        self.assertEqual(self.queue.current_index, 2)
        assert self.queue.current_song is not None
        self.assertEqual(self.queue.current_song.id, "song2")

    def test_add_song(self):
        """Test adding single song"""
        self.queue.set_queue(self.songs[:5], 0)
        new_song = Song(id="new", title="New Song", artist="Test", album="Test", duration=200)

        self.queue.add(new_song)

        self.assertEqual(self.queue.length, 6)
        self.assertEqual(self.queue.queue[-1].id, "new")

    def test_add_songs(self):
        """Test adding multiple songs"""
        self.queue.set_queue(self.songs[:3], 0)
        new_songs = self.songs[3:6]

        self.queue.add_songs(new_songs)

        self.assertEqual(self.queue.length, 6)

    def test_add_next(self):
        """Test adding song next in queue"""
        self.queue.set_queue(self.songs[:5], 2)
        new_song = Song(id="next", title="Next Song", artist="Test", album="Test", duration=150)

        self.queue.add_next(new_song)

        self.assertEqual(self.queue.length, 6)
        self.assertEqual(self.queue.queue[3].id, "next")

    def test_remove_song(self):
        """Test removing song from queue"""
        self.queue.set_queue(self.songs, 5)

        self.queue.remove(3)

        self.assertEqual(self.queue.length, 9)
        self.assertEqual(self.queue.current_index, 4)  # Adjusted

    def test_remove_current_song(self):
        """Test removing currently playing song"""
        self.queue.set_queue(self.songs, 5)

        self.queue.remove(5)

        self.assertEqual(self.queue.length, 9)
        # Current index should point to next song or be adjusted
        self.assertGreaterEqual(self.queue.current_index, 0)

    def test_remove_last_song_makes_empty(self):
        """Test removing all songs makes queue empty"""
        self.queue.set_queue(self.songs[:1], 0)

        self.queue.remove(0)

        self.assertTrue(self.queue.is_empty)
        self.assertEqual(self.queue.current_index, -1)

    def test_clear_queue(self):
        """Test clearing entire queue"""
        self.queue.set_queue(self.songs, 3)

        self.queue.clear()

        self.assertTrue(self.queue.is_empty)
        self.assertEqual(self.queue.length, 0)
        self.assertEqual(self.queue.current_index, -1)

    def test_move_song(self):
        """Test moving song in queue"""
        self.queue.set_queue(self.songs[:5], 0)
        song_at_1 = self.queue.queue[1]

        self.queue.move(1, 3)

        self.assertEqual(self.queue.queue[3].id, song_at_1.id)

    def test_move_current_song_updates_index(self):
        """Test moving current song updates current_index"""
        self.queue.set_queue(self.songs[:5], 2)

        self.queue.move(2, 4)

        self.assertEqual(self.queue.current_index, 4)

    def test_next_without_repeat(self):
        """Test next song without repeat"""
        self.queue.set_queue(self.songs[:5], 0)

        song = self.queue.next()

        self.assertIsNotNone(song)
        assert song is not None  # Type narrowing
        self.assertEqual(song.id, "song1")
        self.assertEqual(self.queue.current_index, 1)

    def test_next_at_end_without_repeat(self):
        """Test next at end of queue without repeat"""
        self.queue.set_queue(self.songs[:5], 4)

        song = self.queue.next()

        self.assertIsNone(song)

    def test_next_with_repeat_all(self):
        """Test next with repeat all"""
        self.queue.set_queue(self.songs[:5], 4)
        self.queue.set_repeat(RepeatMode.ALL)

        song = self.queue.next()

        self.assertIsNotNone(song)
        assert song is not None  # Type narrowing
        self.assertEqual(song.id, "song0")
        self.assertEqual(self.queue.current_index, 0)

    def test_next_with_repeat_one(self):
        """Test next with repeat one"""
        self.queue.set_queue(self.songs[:5], 2)
        self.queue.set_repeat(RepeatMode.ONE)

        song = self.queue.next()

        self.assertIsNotNone(song)
        assert song is not None  # Type narrowing
        self.assertEqual(song.id, "song2")
        self.assertEqual(self.queue.current_index, 2)

    def test_previous(self):
        """Test previous song"""
        self.queue.set_queue(self.songs[:5], 3)

        song = self.queue.previous()

        self.assertIsNotNone(song)
        assert song is not None  # Type narrowing
        self.assertEqual(song.id, "song2")
        self.assertEqual(self.queue.current_index, 2)

    def test_previous_at_start(self):
        """Test previous at start of queue"""
        self.queue.set_queue(self.songs[:5], 0)

        song = self.queue.previous()

        self.assertIsNone(song)

    def test_previous_with_repeat_all(self):
        """Test previous with repeat all wraps around"""
        self.queue.set_queue(self.songs[:5], 0)
        self.queue.set_repeat(RepeatMode.ALL)

        song = self.queue.previous()

        self.assertIsNotNone(song)
        assert song is not None  # Type narrowing
        self.assertEqual(song.id, "song4")
        self.assertEqual(self.queue.current_index, 4)

    def test_shuffle_toggle(self):
        """Test shuffle toggle"""
        self.queue.set_queue(self.songs, 0)

        self.assertFalse(self.queue.shuffle)
        self.queue.toggle_shuffle()
        self.assertTrue(self.queue.shuffle)
        self.queue.toggle_shuffle()
        self.assertFalse(self.queue.shuffle)

    def test_shuffle_keeps_current_song_first(self):
        """Test shuffle keeps current song at index 0"""
        self.queue.set_queue(self.songs, 5)
        current_song = self.queue.current_song
        assert current_song is not None  # Type narrowing

        self.queue.set_shuffle(True)

        self.assertEqual(self.queue.current_index, 0)
        assert self.queue.current_song is not None  # Type narrowing
        self.assertEqual(self.queue.current_song.id, current_song.id)

    def test_unshuffle_restores_order(self):
        """Test unshuffle restores original order"""
        self.queue.set_queue(self.songs, 3)
        original_song = self.queue.current_song
        assert original_song is not None  # Type narrowing

        self.queue.set_shuffle(True)
        self.queue.set_shuffle(False)

        assert self.queue.current_song is not None  # Type narrowing
        self.assertEqual(self.queue.current_song.id, original_song.id)
        # Queue should be back in original order
        for i, song in enumerate(self.queue.queue):
            self.assertEqual(song.id, f"song{i}")

    def test_has_next(self):
        """Test has_next property"""
        self.queue.set_queue(self.songs[:5], 3)
        self.assertTrue(self.queue.has_next)

        self.queue.set_queue(self.songs[:5], 4)
        self.assertFalse(self.queue.has_next)

        # With repeat, always has next
        self.queue.set_repeat(RepeatMode.ALL)
        self.assertTrue(self.queue.has_next)

    def test_has_prev(self):
        """Test has_prev property"""
        self.queue.set_queue(self.songs[:5], 2)
        self.assertTrue(self.queue.has_prev)

        self.queue.set_queue(self.songs[:5], 0)
        self.assertFalse(self.queue.has_prev)

    def test_jump_to_valid_index(self):
        """Test jump_to with valid index"""
        self.queue.set_queue(self.songs, 0)

        song = self.queue.jump_to(5)

        self.assertIsNotNone(song)
        assert song is not None  # Type narrowing
        self.assertEqual(song.id, "song5")
        self.assertEqual(self.queue.current_index, 5)

    def test_jump_to_invalid_index(self):
        """Test jump_to with invalid index"""
        self.queue.set_queue(self.songs, 0)

        song = self.queue.jump_to(-1)
        self.assertIsNone(song)

        song = self.queue.jump_to(100)
        self.assertIsNone(song)

    def test_jump_to_clears_shuffle_history(self):
        """Test jump_to clears shuffle history"""
        self.queue.set_queue(self.songs, 0)
        self.queue.set_shuffle(True)
        self.queue.next()  # Adds to history

        self.queue.jump_to(5)

        self.assertEqual(len(self.queue._history), 0)

    def test_get_upcoming(self):
        """Test getting upcoming songs"""
        self.queue.set_queue(self.songs, 0)

        upcoming = self.queue.get_upcoming(3)

        self.assertEqual(len(upcoming), 3)
        self.assertEqual(upcoming[0].id, "song1")
        self.assertEqual(upcoming[1].id, "song2")
        self.assertEqual(upcoming[2].id, "song3")

    def test_total_duration(self):
        """Test total duration calculation"""
        self.queue.set_queue(self.songs[:5], 0)

        # Each song is 180 seconds, 5 songs = 900
        self.assertEqual(self.queue.total_duration, 900)


if __name__ == "__main__":
    unittest.main()
