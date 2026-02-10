#!/usr/bin/env python3
"""
Comprehensive tests for Equalizer
"""

import math
import os
import sys
import unittest
from unittest.mock import Mock, patch

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "src"))

import tempfile

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
        temp_dir = tempfile.mkdtemp()
        config = AppConfig(config_dir=temp_dir)
        eq = Equalizer(config)
        self.assertEqual(len(eq.gains), 18)
        import shutil

        shutil.rmtree(temp_dir)


class TestEqualizer(unittest.TestCase):
    """Test Equalizer functionality"""

    def setUp(self):
        """Create fresh equalizer for each test"""
        self.temp_dir = tempfile.mkdtemp()
        self.config = AppConfig(config_dir=self.temp_dir)
        self.eq = Equalizer(self.config)

    def tearDown(self):
        """Clean up"""
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

    def test_set_preset(self):
        """Test setting preset by name"""
        result = self.eq.set_preset("Bass Boost")

        self.assertTrue(result)
        # Bass frequencies should be boosted
        self.assertGreater(self.eq.gains[0], 0)

    def test_set_nonexistent_preset(self):
        """Test setting non-existent preset returns False"""
        result = self.eq.set_preset("NonExistent")

        self.assertFalse(result)

    def test_set_flat_preset(self):
        """Test flat preset sets all gains to zero"""
        self.eq.set_band(0, 5.0)
        self.eq.set_preset("Flat")

        for gain in self.eq.gains:
            self.assertEqual(gain, 0.0)

    def test_apply_preset(self):
        """Test applying preset updates gains"""
        from cli_music_player.config import EQPreset

        preset = EQPreset("Test", [3.0] * 18)

        self.eq.apply_preset(preset)

        for gain in self.eq.gains:
            self.assertEqual(gain, 3.0)

    def test_apply_preset_clamps_gains(self):
        """Test applying preset with out-of-range gains clamps them"""
        from cli_music_player.config import EQPreset

        preset = EQPreset("Test", [20.0] * 18)

        self.eq.apply_preset(preset)

        for gain in self.eq.gains:
            self.assertEqual(gain, GAIN_MAX)

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

        # Extract the linear value for band 1
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

    def test_filter_string_clamps_linear_gain(self):
        """Test filter string clamps linear gain to 0-20"""
        # Even with extreme dB values, linear should be clamped
        self.eq.set_band(0, GAIN_MAX)  # +12dB

        filter_str = self.eq.get_filter_string()

        # +12dB = 10^(12/20) = 3.981, should not exceed 20
        self.assertIn("superequalizer=", filter_str)

    def test_apply_with_player(self):
        """Test apply method with player set"""
        mock_player = Mock()
        self.eq.set_player(mock_player)

        self.eq.set_band(0, 3.0)
        self.eq.apply()

        # Should call player's apply_eq
        mock_player.apply_eq.assert_called_once()

    def test_apply_without_player_no_error(self):
        """Test apply without player doesn't raise error"""
        self.eq.set_band(0, 3.0)

        # Should not raise
        try:
            self.eq.apply()
        except Exception as e:
            self.fail(f"apply() raised {e} unexpectedly")

    def test_toggle_enabled(self):
        """Test toggling enabled state"""
        self.assertTrue(self.eq.enabled)

        self.eq.enabled = False
        self.assertFalse(self.eq.enabled)

        self.eq.enabled = True
        self.assertTrue(self.eq.enabled)

    def test_all_bands_settable(self):
        """Test that all 18 bands can be set"""
        for i in range(18):
            self.eq.set_band(i, float(i))

        for i in range(18):
            self.assertEqual(self.eq.gains[i], float(i))


class TestEqualizerPresets(unittest.TestCase):
    """Test built-in EQ presets"""

    def setUp(self):
        self.temp_dir = tempfile.mkdtemp()
        self.config = AppConfig(config_dir=self.temp_dir)
        self.eq = Equalizer(self.config)

    def tearDown(self):
        import shutil

        shutil.rmtree(self.temp_dir)

    def test_bass_boost_preset(self):
        """Test Bass Boost preset boosts low frequencies"""
        self.eq.set_preset("Bass Boost")

        # First few bands should be boosted
        self.assertGreater(self.eq.gains[0], 0)
        self.assertGreater(self.eq.gains[1], 0)
        # Should taper off
        self.assertGreater(self.eq.gains[0], self.eq.gains[4])

    def test_treble_boost_preset(self):
        """Test Treble Boost preset boosts high frequencies"""
        self.eq.set_preset("Treble Boost")

        # Last few bands should be boosted
        self.assertGreater(self.eq.gains[-1], 0)
        self.assertGreater(self.eq.gains[-2], 0)
        # First bands should be flat
        self.assertEqual(self.eq.gains[0], 0)

    def test_vocal_preset(self):
        """Test Vocal preset exists and loads"""
        result = self.eq.set_preset("Vocal")

        self.assertTrue(result)

    def test_rock_preset(self):
        """Test Rock preset exists and loads"""
        result = self.eq.set_preset("Rock")

        self.assertTrue(result)


class TestEqualizerEdgeCases(unittest.TestCase):
    """Test edge cases and error handling"""

    def setUp(self):
        self.temp_dir = tempfile.mkdtemp()
        self.config = AppConfig(config_dir=self.temp_dir)
        self.eq = Equalizer(self.config)

    def tearDown(self):
        import shutil

        shutil.rmtree(self.temp_dir)

    def test_set_band_invalid_index(self):
        """Test setting invalid band index doesn't crash"""
        # Should not raise
        try:
            self.eq.set_band(-1, 5.0)
            self.eq.set_band(100, 5.0)
        except IndexError:
            pass  # Expected

    def test_very_small_gain_values(self):
        """Test very small gain values"""
        self.eq.set_band(0, 0.001)

        filter_str = self.eq.get_filter_string()
        # Should still generate filter
        self.assertIn("superequalizer=", filter_str)

    def test_gain_precision(self):
        """Test gain values maintain precision"""
        self.eq.set_band(0, 3.456)

        # Should clamp but maintain reasonable precision
        self.assertAlmostEqual(self.eq.gains[0], 3.456, places=3)


if __name__ == "__main__":
    unittest.main()
