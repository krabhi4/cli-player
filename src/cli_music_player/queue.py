"""Play queue and playlist management."""

import random
from enum import Enum
from typing import Optional

from .subsonic import Song


class RepeatMode(Enum):
    OFF = "off"
    ALL = "all"
    ONE = "one"

    def next(self) -> "RepeatMode":
        """Cycle to next repeat mode."""
        modes = list(RepeatMode)
        idx = modes.index(self)
        return modes[(idx + 1) % len(modes)]

    @property
    def label(self) -> str:
        return {
            RepeatMode.OFF: "Off",
            RepeatMode.ALL: "All",
            RepeatMode.ONE: "One",
        }[self]

    @property
    def icon(self) -> str:
        return {
            RepeatMode.OFF: "🔁",
            RepeatMode.ALL: "🔁",
            RepeatMode.ONE: "🔂",
        }[self]


class QueueManager:
    """Manages the play queue with shuffle and repeat support."""

    def __init__(self):
        self._original_queue: list[Song] = []
        self._queue: list[Song] = []
        self._current_index: int = -1
        self._shuffle: bool = False
        self._repeat: RepeatMode = RepeatMode.OFF
        self._history: list[int] = []  # For tracking prev in shuffle

    @property
    def queue(self) -> list[Song]:
        return list(self._queue)

    @property
    def current_index(self) -> int:
        return self._current_index

    @property
    def current_song(self) -> Optional[Song]:
        if 0 <= self._current_index < len(self._queue):
            return self._queue[self._current_index]
        return None

    @property
    def is_empty(self) -> bool:
        return len(self._queue) == 0

    @property
    def length(self) -> int:
        return len(self._queue)

    @property
    def shuffle(self) -> bool:
        return self._shuffle

    @property
    def repeat(self) -> RepeatMode:
        return self._repeat

    @property
    def has_next(self) -> bool:
        if self._repeat in (RepeatMode.ALL, RepeatMode.ONE):
            return len(self._queue) > 0
        return self._current_index < len(self._queue) - 1

    @property
    def has_prev(self) -> bool:
        if self._shuffle:
            return len(self._history) > 0
        return self._current_index > 0

    def set_queue(self, songs: list[Song], start_index: int = 0):
        """Replace the entire queue with new songs."""
        self._original_queue = list(songs)
        self._queue = list(songs)
        self._current_index = start_index
        self._history = []
        if self._shuffle:
            self._apply_shuffle()

    def add(self, song: Song):
        """Add a song to the end of the queue."""
        self._original_queue.append(song)
        self._queue.append(song)

    def add_songs(self, songs: list[Song]):
        """Add multiple songs to the queue."""
        self._original_queue.extend(songs)
        self._queue.extend(songs)

    def add_next(self, song: Song):
        """Insert a song right after the current one."""
        insert_pos = self._current_index + 1
        self._original_queue.insert(insert_pos, song)
        self._queue.insert(insert_pos, song)

    def remove(self, index: int):
        """Remove a song by queue index."""
        if 0 <= index < len(self._queue):
            self._queue.pop(index)
            if index < self._current_index:
                self._current_index -= 1
            elif index == self._current_index:
                if self._current_index >= len(self._queue):
                    self._current_index = len(self._queue) - 1

    def clear(self):
        """Clear the entire queue."""
        self._queue.clear()
        self._original_queue.clear()
        self._current_index = -1
        self._history.clear()

    def move(self, from_idx: int, to_idx: int):
        """Move a song from one position to another."""
        if (
            0 <= from_idx < len(self._queue)
            and 0 <= to_idx < len(self._queue)
        ):
            song = self._queue.pop(from_idx)
            self._queue.insert(to_idx, song)
            # Adjust current_index
            if from_idx == self._current_index:
                self._current_index = to_idx
            elif from_idx < self._current_index <= to_idx:
                self._current_index -= 1
            elif to_idx <= self._current_index < from_idx:
                self._current_index += 1

    def next(self) -> Optional[Song]:
        """Advance to the next song and return it."""
        if self.is_empty:
            return None

        if self._repeat == RepeatMode.ONE:
            return self.current_song

        if self._shuffle:
            self._history.append(self._current_index)
            remaining = [
                i
                for i in range(len(self._queue))
                if i != self._current_index
            ]
            if remaining:
                self._current_index = random.choice(remaining)
            elif self._repeat == RepeatMode.ALL:
                self._current_index = random.randint(0, len(self._queue) - 1)
            else:
                return None
        else:
            if self._current_index < len(self._queue) - 1:
                self._current_index += 1
            elif self._repeat == RepeatMode.ALL:
                self._current_index = 0
            else:
                return None

        return self.current_song

    def previous(self) -> Optional[Song]:
        """Go to the previous song and return it."""
        if self.is_empty:
            return None

        if self._shuffle and self._history:
            self._current_index = self._history.pop()
        elif self._current_index > 0:
            self._current_index -= 1
        elif self._repeat == RepeatMode.ALL:
            self._current_index = len(self._queue) - 1
        else:
            return None

        return self.current_song

    def toggle_shuffle(self):
        """Toggle shuffle mode."""
        self._shuffle = not self._shuffle
        if self._shuffle:
            self._apply_shuffle()
        else:
            self._restore_order()

    def set_shuffle(self, enabled: bool):
        """Set shuffle mode explicitly."""
        if enabled != self._shuffle:
            self.toggle_shuffle()

    def cycle_repeat(self) -> RepeatMode:
        """Cycle through repeat modes and return the new mode."""
        self._repeat = self._repeat.next()
        return self._repeat

    def set_repeat(self, mode: RepeatMode):
        """Set repeat mode explicitly."""
        self._repeat = mode

    def _apply_shuffle(self):
        """Shuffle the queue while keeping current song in place."""
        if len(self._queue) <= 1:
            return
        current = self.current_song
        others = [s for i, s in enumerate(self._queue) if i != self._current_index]
        random.shuffle(others)
        self._queue = [current] + others if current else others
        self._current_index = 0
        self._history = []

    def _restore_order(self):
        """Restore original order after unshuffle."""
        current = self.current_song
        self._queue = list(self._original_queue)
        if current:
            try:
                self._current_index = next(
                    i for i, s in enumerate(self._queue) if s.id == current.id
                )
            except StopIteration:
                self._current_index = 0
        self._history = []

    def get_upcoming(self, count: int = 20) -> list[Song]:
        """Get the next N songs in the queue after the current one."""
        if self.is_empty or self._current_index < 0:
            return []
        start = self._current_index + 1
        return self._queue[start : start + count]

    @property
    def total_duration(self) -> int:
        """Total duration of all songs in queue (seconds)."""
        return sum(s.duration for s in self._queue)
