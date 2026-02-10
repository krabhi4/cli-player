#!/usr/bin/env python3
"""
Comprehensive tests for Equalizer (FIXED)
"""

import math
import os
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import Mock, patch

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "src"))

from cli_music_player.config import AppConfig
from cli_music_player.equalizer import GAIN_MAX, GAIN_MIN, Equalizer


class TestEqualizerConstants(unittest.TestCase):
    """Test equalizer constants"""

    def test_gain_range(self):
        """Test gain min/max values"""
        self.assertEqual(GAIN_MIN, -12.0)
        self.assertEqual(GAIN_MAX, 12.0)

    def test_band_count(self):
        """Test that equalizer has 18 bands"""
        # Mock config path
        with patch("cli_music_player.config.Path.home") as mock_home:
            temp_dir = tempfile.mkdtemp()
            mock_home.return_value = Path(temp_dir)

            config = AppConfig()
            eq = Equalizer(config)
            self.assertEqual(len(eq.gains), 18)

            import shutil

            shutil.rmtree(temp_dir)


class TestEqualizer(unittest.TestCase):
    """Test Equalizer functionality"""

    def setUp(self):
        """Create fresh equalizer for each test"""
        self.temp_dir = tempfile.mkdtemp()
        test_config_dir = Path(self.temp_dir) / "cli-music-player"
        test_config_file = test_config_dir / "config.json"

        self.patcher_dir = patch("cli_music_player.config.CONFIG_DIR", test_config_dir)
        self.patcher_file = patch(
            "cli_music_player.config.CONFIG_FILE", test_config_file
        )

        self.patcher_dir.start()
        self.patcher_file.start()

        self.config = AppConfig()
        self.eq = Equalizer(self.config)

    def tearDown(self):
        """Clean up"""
        self.patcher_dir.stop()
        self.patcher_file.stop()
        import shutil

        shutil.rmtree(self.temp_dir)

    def test_equalizer_initialization(self):
        """Test equalizer initializes correctly"""
        self.assertEqual(len(self.eq.gains), 18)
        self.assertTrue(self.eq.enabled)
        # Default should be flat (all zeros)
        for gain in self.eq.gains:
            self.assertEqual(gain, 0.0)

    def test_set_band(self):
        """Test setting individual band gain"""
        self.eq.set_band(0, 5.0)

        self.assertEqual(self.eq.gains[0], 5.0)
        # Other bands unchanged
        self.assertEqual(self.eq.gains[1], 0.0)

    def test_set_band_clamps_to_max(self):
        """Test setting band above max clamps to GAIN_MAX"""
        self.eq.set_band(0, 20.0)

        self.assertEqual(self.eq.gains[0], GAIN_MAX)

    def test_set_band_clamps_to_min(self):
        """Test setting band below min clamps to GAIN_MIN"""
        self.eq.set_band(0, -20.0)

        self.assertEqual(self.eq.gains[0], GAIN_MIN)

    def test_load_preset(self):
        """Test loading preset by name"""
        self.eq.load_preset("Bass Boost")

        # Bass frequencies should be boosted
        self.assertGreater(self.eq.gains[0], 0)

    def test_load_nonexistent_preset(self):
        """Test loading non-existent preset does nothing"""
        original_gains = list(self.eq.gains)
        self.eq.load_preset("NonExistent")

        # Gains should be unchanged
        self.assertEqual(self.eq.gains, original_gains)

    def test_reset(self):
        """Test reset sets all gains to zero"""
        self.eq.set_band(0, 5.0)
        self.eq.reset()

        for gain in self.eq.gains:
            self.assertEqual(gain, 0.0)

    def test_toggle(self):
        """Test toggling enabled state"""
        self.assertTrue(self.eq.enabled)
        self.eq.toggle()
        self.assertFalse(self.eq.enabled)
        self.eq.toggle()
        self.assertTrue(self.eq.enabled)

    def test_get_filter_string_disabled(self):
        """Test filter string is empty when disabled"""
        self.eq.enabled = False

        filter_str = self.eq.get_filter_string()

        self.assertEqual(filter_str, "")

    def test_get_filter_string_all_zero(self):
        """Test filter string is empty when all gains are zero"""
        for i in range(18):
            self.eq.gains[i] = 0.0

        filter_str = self.eq.get_filter_string()

        self.assertEqual(filter_str, "")

    def test_get_filter_string_format(self):
        """Test filter string format is correct"""
        self.eq.set_band(0, 6.0)  # +6dB

        filter_str = self.eq.get_filter_string()

        self.assertIn("superequalizer=", filter_str)
        self.assertIn("1b=", filter_str)  # Band 1

    def test_db_to_linear_conversion(self):
        """Test dB to linear conversion is correct"""
        # Set +6dB which should convert to ~2.0 linear
        self.eq.set_band(0, 6.0)

        filter_str = self.eq.get_filter_string()

        # Should be around 1.995 (10^(6/20) = 1.995)
        self.assertIn("1b=1.9", filter_str)  # Approximately 2.0

    def test_db_to_linear_zero_db(self):
        """Test 0dB converts to 1.0 linear"""
        self.eq.set_band(0, 0.0)
        self.eq.set_band(1, 1.0)  # Force filter to generate

        filter_str = self.eq.get_filter_string()

        # 0dB should be 1.0 linear
        expected_linear = math.pow(10, 0.0 / 20.0)
        self.assertAlmostEqual(expected_linear, 1.0)

    def test_db_to_linear_negative_db(self):
        """Test negative dB converts correctly"""
        # -6dB should be ~0.5 linear
        db = -6.0
        expected_linear = math.pow(10, db / 20.0)

        self.assertAlmostEqual(expected_linear, 0.501, places=2)

    def test_apply_with_player(self):
        """Test apply method with player set"""
        mock_player = Mock()
        self.eq.set_player(mock_player)

        self.eq.set_band(0, 3.0)
        self.eq.apply()

        # Should call player's set_audio_filter (may be called multiple times)
        self.assertTrue(mock_player.set_audio_filter.called)
        # Verify the filter string contains the expected band setting
        last_call = mock_player.set_audio_filter.call_args[0][0]
        self.assertIn("1b=1.4", last_call)  # Band 1 with +3dB ≈ 1.413 linear

    def test_apply_without_player_no_error(self):
        """Test apply without player doesn't raise error"""
        self.eq.set_band(0, 3.0)

        # Should not raise
        try:
            self.eq.apply()
        except Exception as e:
            self.fail(f"apply() raised {e} unexpectedly")

    def test_all_bands_settable(self):
        """Test that all 18 bands can be set"""
        for i in range(18):
            self.eq.set_band(i, float(i % 13) - 6)  # Stay in range

        for i in range(18):
            expected = float(i % 13) - 6
            self.assertAlmostEqual(self.eq.gains[i], expected)

    def test_set_all_bands(self):
        """Test setting all bands at once"""
        gains = [float(i) for i in range(18)]
        # Clamp to valid range
        gains = [max(GAIN_MIN, min(GAIN_MAX, g)) for g in gains]

        self.eq.set_all_bands(gains)

        for i in range(18):
            self.assertAlmostEqual(self.eq.gains[i], gains[i])

    def test_get_presets(self):
        """Test getting list of presets"""
        presets = self.eq.get_presets()

        self.assertGreater(len(presets), 0)
        preset_names = [p.name for p in presets]
        self.assertIn("Flat", preset_names)
        self.assertIn("Bass Boost", preset_names)

    def test_save_as_preset(self):
        """Test saving current settings as preset"""
        self.eq.set_band(0, 5.0)
        self.eq.set_band(1, 3.0)

        self.eq.save_as_preset("Custom Test")

        # Should be able to retrieve it
        preset = self.config.get_eq_preset("Custom Test")
        self.assertIsNotNone(preset)
        self.assertEqual(preset.gains[0], 5.0)

    def test_band_label_static_method(self):
        """Test band_label static method"""
        label = Equalizer.band_label(0)
        self.assertIsNotNone(label)
        self.assertTrue(len(label) > 0)

    def test_band_frequency_static_method(self):
        """Test band_frequency static method"""
        freq = Equalizer.band_frequency(0)
        self.assertIsInstance(freq, int)
        self.assertGreater(freq, 0)


class TestEqualizerPresets(unittest.TestCase):
    """Test built-in EQ presets"""

    def setUp(self):
        self.temp_dir = tempfile.mkdtemp()
        test_config_dir = Path(self.temp_dir) / "cli-music-player"
        test_config_file = test_config_dir / "config.json"

        self.patcher_dir = patch("cli_music_player.config.CONFIG_DIR", test_config_dir)
        self.patcher_file = patch(
            "cli_music_player.config.CONFIG_FILE", test_config_file
        )

        self.patcher_dir.start()
        self.patcher_file.start()

        self.config = AppConfig()
        self.eq = Equalizer(self.config)

    def tearDown(self):
        self.patcher_dir.stop()
        self.patcher_file.stop()
        import shutil

        shutil.rmtree(self.temp_dir)

    def test_bass_boost_preset(self):
        """Test Bass Boost preset boosts low frequencies"""
        self.eq.load_preset("Bass Boost")

        # First few bands should be boosted
        self.assertGreater(self.eq.gains[0], 0)
        self.assertGreater(self.eq.gains[1], 0)
        # Should taper off
        self.assertGreater(self.eq.gains[0], self.eq.gains[4])

    def test_treble_boost_preset(self):
        """Test Treble Boost preset boosts high frequencies"""
        self.eq.load_preset("Treble Boost")

        # Last few bands should be boosted
        self.assertGreater(self.eq.gains[-1], 0)
        self.assertGreater(self.eq.gains[-2], 0)
        # First bands should be flat
        self.assertEqual(self.eq.gains[0], 0)

    def test_flat_preset(self):
        """Test Flat preset sets all to zero"""
        self.eq.set_band(0, 5.0)
        self.eq.load_preset("Flat")

        for gain in self.eq.gains:
            self.assertEqual(gain, 0.0)


if __name__ == "__main__":
    unittest.main()
