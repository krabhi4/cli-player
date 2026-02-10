"""Audio playback engine using python-mpv."""

import threading
import time
from typing import Callable, Optional

import mpv

from .subsonic import Song


class PlaybackState:
    STOPPED = "stopped"
    PLAYING = "playing"
    PAUSED = "paused"


class Player:
    """MPV-based audio player for streaming from Navidrome."""

    def __init__(self, audio_device: str = "auto"):
        self._mpv: Optional[mpv.MPV] = None
        self._state = PlaybackState.STOPPED
        self._current_song: Optional[Song] = None
        self._volume: int = 75
        self._muted: bool = False
        self._audio_device = audio_device

        # Callbacks
        self.on_track_end: Optional[Callable] = None
        self.on_position_update: Optional[Callable[[float, float], None]] = None
        self.on_state_change: Optional[Callable[[str], None]] = None
        self.on_metadata_update: Optional[Callable[[dict], None]] = None

        self._position: float = 0.0
        self._duration: float = 0.0
        self._lock = threading.Lock()

        self._init_mpv()

    def _init_mpv(self):
        """Initialize the mpv instance."""
        opts = {
            "video": False,  # Audio only
            "input_default_bindings": False,
            "input_vo_keyboard": False,
            "terminal": False,
            "volume": self._volume,
        }

        # Use ALSA directly since no PulseAudio/PipeWire
        if self._audio_device == "auto":
            opts["ao"] = "alsa"
        else:
            opts["ao"] = "alsa"
            opts["audio_device"] = f"alsa/{self._audio_device}"

        self._mpv = mpv.MPV(**opts)

        # Observe properties
        @self._mpv.property_observer("time-pos")
        def _time_pos_observer(_name, value):
            if value is not None:
                self._position = float(value)
                if self.on_position_update and self._duration > 0:
                    self.on_position_update(self._position, self._duration)

        @self._mpv.property_observer("duration")
        def _duration_observer(_name, value):
            if value is not None:
                self._duration = float(value)

        @self._mpv.property_observer("pause")
        def _pause_observer(_name, value):
            if value is True:
                self._state = PlaybackState.PAUSED
            elif value is False and self._current_song:
                self._state = PlaybackState.PLAYING
            if self.on_state_change:
                self.on_state_change(self._state)

        # End of file handler
        @self._mpv.event_callback("end-file")
        def _eof_handler(event):
            if event.get("event", {}).get("reason") == "eof" or (
                hasattr(event, "event") and hasattr(event.event, 'reason')
            ):
                try:
                    reason = event["event"]["reason"]
                except (KeyError, TypeError):
                    try:
                        reason = event.event.reason if hasattr(event, 'event') else None
                    except Exception:
                        reason = None

                if reason is not None:
                    # Convert MpvEventEndFile.REASON to check for EOF
                    reason_val = int(reason) if not isinstance(reason, int) else reason
                    if reason_val == 0:  # EOF = 0
                        self._state = PlaybackState.STOPPED
                        if self.on_track_end:
                            self.on_track_end()

    def play(self, url: str, song: Optional[Song] = None):
        """Play a song from a URL."""
        with self._lock:
            self._current_song = song
            self._position = 0.0
            self._duration = song.duration if song else 0.0
            self._state = PlaybackState.PLAYING

            if self._mpv:
                self._mpv.play(url)
                self._mpv.pause = False

            if self.on_state_change:
                self.on_state_change(self._state)

    def pause(self):
        """Pause playback."""
        if self._mpv and self._state == PlaybackState.PLAYING:
            self._mpv.pause = True

    def resume(self):
        """Resume playback."""
        if self._mpv and self._state == PlaybackState.PAUSED:
            self._mpv.pause = False

    def toggle_pause(self):
        """Toggle play/pause."""
        if self._state == PlaybackState.PLAYING:
            self.pause()
        elif self._state == PlaybackState.PAUSED:
            self.resume()

    def stop(self):
        """Stop playback."""
        if self._mpv:
            self._mpv.stop()
        self._state = PlaybackState.STOPPED
        self._current_song = None
        self._position = 0.0
        if self.on_state_change:
            self.on_state_change(self._state)

    def seek(self, offset: float):
        """Seek forward/backward by offset seconds."""
        if self._mpv and self._state in (
            PlaybackState.PLAYING,
            PlaybackState.PAUSED,
        ):
            self._mpv.seek(offset, "relative")

    def seek_to(self, position: float):
        """Seek to an absolute position in seconds."""
        if self._mpv and self._state in (
            PlaybackState.PLAYING,
            PlaybackState.PAUSED,
        ):
            self._mpv.seek(position, "absolute")

    @property
    def volume(self) -> int:
        return self._volume

    @volume.setter
    def volume(self, value: int):
        self._volume = max(0, min(100, value))
        if self._mpv:
            self._mpv.volume = self._volume

    def volume_up(self, step: int = 5):
        self.volume = self._volume + step

    def volume_down(self, step: int = 5):
        self.volume = self._volume - step

    @property
    def muted(self) -> bool:
        return self._muted

    def mute_toggle(self):
        self._muted = not self._muted
        if self._mpv:
            self._mpv.mute = self._muted

    @property
    def state(self) -> str:
        return self._state

    @property
    def current_song(self) -> Optional[Song]:
        return self._current_song

    @property
    def position(self) -> float:
        return self._position

    @property
    def duration(self) -> float:
        return self._duration

    def set_audio_filter(self, filter_str: str):
        """Set an audio filter (used by equalizer)."""
        if self._mpv:
            try:
                if filter_str:
                    self._mpv.af = filter_str
                else:
                    self._mpv.af = ""
            except Exception:
                pass  # Filter may fail on some formats

    def get_audio_filter(self) -> str:
        """Get current audio filter string."""
        if self._mpv:
            try:
                return self._mpv.af or ""
            except Exception:
                return ""
        return ""

    def cleanup(self):
        """Clean up mpv instance."""
        if self._mpv:
            try:
                self._mpv.terminate()
            except Exception:
                pass
            self._mpv = None
