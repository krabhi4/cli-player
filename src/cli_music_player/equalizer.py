"""Equalizer engine using mpv's superequalizer audio filter."""

from typing import Optional
from .config import AppConfig, EQPreset


# 18-band superequalizer frequencies (Hz)
EQ_BANDS = [
    65, 92, 131, 185, 262, 370, 523, 740, 1047,
    1480, 2093, 2960, 4186, 5920, 8372, 11840, 16744, 20000,
]

# Band labels for display
EQ_BAND_LABELS = [
    "65", "92", "131", "185", "262", "370", "523", "740", "1K",
    "1.5K", "2.1K", "3K", "4.2K", "5.9K", "8.4K", "12K", "17K", "20K",
]

# Gain range in dB
GAIN_MIN = -12.0
GAIN_MAX = 12.0


class Equalizer:
    """18-band equalizer using mpv's superequalizer filter."""

    def __init__(self, config: AppConfig):
        self.config = config
        self.gains: list[float] = [0.0] * 18
        self.enabled: bool = True
        self._player = None  # Set by app after player init

        # Load last used preset
        preset = config.get_eq_preset(config.active_eq_preset)
        if preset:
            self.gains = list(preset.gains)
        else:
            self.gains = list(config.custom_eq_gains)

    def set_player(self, player):
        """Attach the player instance for filter application."""
        self._player = player
        self.apply()

    def get_filter_string(self) -> str:
        """Build the mpv superequalizer filter string from current gains."""
        if not self.enabled or all(g == 0.0 for g in self.gains):
            return ""

        # superequalizer uses bands 1b through 18b
        parts = []
        for i, gain in enumerate(self.gains):
            band_num = i + 1
            # superequalizer gain is in dB, range roughly -20 to +20
            clamped = max(GAIN_MIN, min(GAIN_MAX, gain))
            # mpv superequalizer expects absolute level, where 0dB = 1.0
            # Convert dB gain to linear multiplier then to superequalizer value
            # Actually superequalizer takes dB values directly: 0 = no change
            # Values > 0 boost, values < 0 cut; range is roughly 0-20
            # The parameter is actually a positive multiplier where ~2 = +6dB
            # Let's use the simpler approach: map -12..+12 dB to 0..24
            level = clamped + 12  # Shift to 0..24 range
            parts.append(f"{band_num}b={level:.1f}")

        return f"superequalizer={':'.join(parts)}"

    def apply(self):
        """Apply current EQ settings to the player."""
        if self._player:
            filter_str = self.get_filter_string()
            self._player.set_audio_filter(filter_str)

    def set_band(self, band_index: int, gain: float):
        """Set gain for a specific band (0-17)."""
        if 0 <= band_index < 18:
            self.gains[band_index] = max(GAIN_MIN, min(GAIN_MAX, gain))
            self.apply()

    def set_all_bands(self, gains: list[float]):
        """Set all 18 bands at once."""
        for i, g in enumerate(gains[:18]):
            self.gains[i] = max(GAIN_MIN, min(GAIN_MAX, g))
        self.apply()

    def reset(self):
        """Reset all bands to flat (0 dB)."""
        self.gains = [0.0] * 18
        self.apply()

    def toggle(self):
        """Toggle equalizer on/off."""
        self.enabled = not self.enabled
        self.apply()

    def load_preset(self, preset_name: str):
        """Load a named preset."""
        preset = self.config.get_eq_preset(preset_name)
        if preset:
            self.gains = list(preset.gains)
            self.config.active_eq_preset = preset_name
            self.config.save()
            self.apply()

    def save_as_preset(self, name: str):
        """Save current gains as a custom preset."""
        self.config.save_custom_eq_preset(name, self.gains)

    def get_presets(self) -> list[EQPreset]:
        """Get all available presets."""
        return list(self.config.eq_presets)

    def get_current_preset_name(self) -> str:
        """Get the name of the currently active preset."""
        return self.config.active_eq_preset

    @staticmethod
    def band_label(index: int) -> str:
        """Get the display label for a band."""
        if 0 <= index < 18:
            return EQ_BAND_LABELS[index]
        return "?"

    @staticmethod
    def band_frequency(index: int) -> int:
        """Get the frequency of a band."""
        if 0 <= index < 18:
            return EQ_BANDS[index]
        return 0
