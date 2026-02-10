#!/usr/bin/env python3
"""
Comprehensive tests for Configuration Management
"""

import json
import os
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "src"))

from cli_music_player.config import (
    DEFAULT_EQ_PRESETS,
    AppConfig,
    EQPreset,
    ServerConfig,
    decrypt_password,
    encrypt_password,
)


class TestPasswordEncryption(unittest.TestCase):
    """Test password encryption/decryption"""

    def test_encrypt_decrypt_roundtrip(self):
        """Test encryption and decryption"""
        password = "test_password_123!@#"
        encrypted = encrypt_password(password)
        decrypted = decrypt_password(encrypted)

        self.assertEqual(password, decrypted)
        self.assertNotEqual(password, encrypted)

    def test_encrypted_password_is_different(self):
        """Test that encrypted password differs from original"""
        password = "mypassword"
        encrypted = encrypt_password(password)

        self.assertNotEqual(password, encrypted)
        self.assertTrue(len(encrypted) > len(password))

    def test_decrypt_invalid_data_raises_error(self):
        """Test decrypting invalid data raises ValueError"""
        with self.assertRaises(ValueError) as context:
            decrypt_password("not_valid_encrypted_data")

        self.assertIn("Cannot decrypt password", str(context.exception))

    def test_same_password_different_encryption(self):
        """Test same password encrypts differently each time (salt)"""
        password = "test123"
        enc1 = encrypt_password(password)
        enc2 = encrypt_password(password)

        # Should be different due to random salt
        self.assertNotEqual(enc1, enc2)
        # But both decrypt to same password
        self.assertEqual(decrypt_password(enc1), password)
        self.assertEqual(decrypt_password(enc2), password)


class TestServerConfig(unittest.TestCase):
    """Test ServerConfig dataclass"""

    def test_server_config_creation(self):
        """Test creating server config"""
        server = ServerConfig(
            name="Test Server", url="http://localhost:4533", username="testuser"
        )

        self.assertEqual(server.name, "Test Server")
        self.assertEqual(server.url, "http://localhost:4533")
        self.assertEqual(server.username, "testuser")
        self.assertEqual(server._encrypted_password, "")

    def test_server_config_to_dict(self):
        """Test converting server config to dict"""
        server = ServerConfig(
            name="Test",
            url="http://test.com",
            username="user",
            _encrypted_password="encrypted",
        )

        data = server.to_dict()

        self.assertEqual(data["name"], "Test")
        self.assertEqual(data["url"], "http://test.com")
        self.assertEqual(data["username"], "user")
        self.assertEqual(data["_encrypted_password"], "encrypted")

    def test_server_config_from_dict(self):
        """Test creating server config from dict"""
        data = {
            "name": "Test",
            "url": "http://test.com",
            "username": "user",
            "_encrypted_password": "encrypted",
        }

        server = ServerConfig.from_dict(data)

        self.assertEqual(server.name, "Test")
        self.assertEqual(server.url, "http://test.com")


class TestEQPreset(unittest.TestCase):
    """Test EQ preset handling"""

    def test_eq_preset_creation(self):
        """Test creating EQ preset"""
        gains = [1.0, 2.0, 3.0] + [0.0] * 15
        preset = EQPreset(name="Test Preset", gains=gains)

        self.assertEqual(preset.name, "Test Preset")
        self.assertEqual(len(preset.gains), 18)
        self.assertEqual(preset.gains[0], 1.0)

    def test_eq_preset_to_dict(self):
        """Test converting preset to dict"""
        preset = EQPreset(name="Test", gains=[0.0] * 18)
        data = preset.to_dict()

        self.assertEqual(data["name"], "Test")
        self.assertEqual(len(data["gains"]), 18)

    def test_eq_preset_from_dict(self):
        """Test creating preset from dict"""
        data = {"name": "Test", "gains": [1.0] * 18}
        preset = EQPreset.from_dict(data)

        self.assertEqual(preset.name, "Test")
        self.assertEqual(len(preset.gains), 18)

    def test_default_eq_presets_exist(self):
        """Test that default EQ presets are defined"""
        self.assertGreater(len(DEFAULT_EQ_PRESETS), 0)

        # Check for common presets
        preset_names = [p.name for p in DEFAULT_EQ_PRESETS]
        self.assertIn("Flat", preset_names)
        self.assertIn("Bass Boost", preset_names)
        self.assertIn("Treble Boost", preset_names)


class TestAppConfig(unittest.TestCase):
    """Test AppConfig functionality"""

    def setUp(self):
        """Create temporary config directory for each test"""
        self.temp_dir = tempfile.mkdtemp()
        self.config_file = os.path.join(self.temp_dir, "config.json")

    def tearDown(self):
        """Clean up temporary files"""
        if os.path.exists(self.config_file):
            os.remove(self.config_file)
        os.rmdir(self.temp_dir)

    def test_config_initialization(self):
        """Test config initialization"""
        config = AppConfig(config_dir=self.temp_dir)

        self.assertEqual(config.servers, [])
        self.assertEqual(config.active_server_index, 0)
        self.assertEqual(config.volume, 100)
        self.assertFalse(config.shuffle)
        self.assertEqual(config.repeat_mode, "off")

    def test_add_server(self):
        """Test adding server"""
        config = AppConfig(config_dir=self.temp_dir)

        config.add_server("Test Server", "http://test.com", "user", "pass")

        self.assertEqual(len(config.servers), 1)
        self.assertEqual(config.servers[0].name, "Test Server")
        self.assertEqual(config.servers[0].url, "http://test.com")
        self.assertEqual(config.servers[0].username, "user")
        # Password should be encrypted
        self.assertNotEqual(config.servers[0]._encrypted_password, "pass")

    def test_remove_server(self):
        """Test removing server"""
        config = AppConfig(config_dir=self.temp_dir)
        config.add_server("Server 1", "http://s1.com", "u1", "p1")
        config.add_server("Server 2", "http://s2.com", "u2", "p2")

        config.remove_server(0)

        self.assertEqual(len(config.servers), 1)
        self.assertEqual(config.servers[0].name, "Server 2")

    def test_get_password(self):
        """Test getting decrypted password"""
        config = AppConfig(config_dir=self.temp_dir)
        config.add_server("Test", "http://test.com", "user", "mypassword")

        password = config.get_password(config.servers[0])

        self.assertEqual(password, "mypassword")

    def test_get_password_invalid_returns_empty(self):
        """Test getting password with invalid encryption returns empty"""
        config = AppConfig(config_dir=self.temp_dir)
        server = ServerConfig("Test", "http://test.com", "user", "invalid_encrypted")
        config.servers.append(server)

        password = config.get_password(server)

        self.assertEqual(password, "")
        # Invalid password should be cleared
        self.assertEqual(server._encrypted_password, "")

    def test_set_active_server(self):
        """Test setting active server"""
        config = AppConfig(config_dir=self.temp_dir)
        config.add_server("Server 1", "http://s1.com", "u1", "p1")
        config.add_server("Server 2", "http://s2.com", "u2", "p2")

        config.set_active_server(1)

        self.assertEqual(config.active_server_index, 1)
        self.assertEqual(config.active_server.name, "Server 2")

    def test_active_server_property(self):
        """Test active_server property"""
        config = AppConfig(config_dir=self.temp_dir)
        config.add_server("Test", "http://test.com", "user", "pass")

        server = config.active_server

        self.assertIsNotNone(server)
        self.assertEqual(server.name, "Test")

    def test_active_server_none_when_no_servers(self):
        """Test active_server is None when no servers"""
        config = AppConfig(config_dir=self.temp_dir)

        self.assertIsNone(config.active_server)

    def test_save_and_load_config(self):
        """Test saving and loading config"""
        config1 = AppConfig(config_dir=self.temp_dir)
        config1.add_server("Test Server", "http://test.com", "user", "pass")
        config1.volume = 75
        config1.shuffle = True
        config1.save()

        # Load in new instance
        config2 = AppConfig(config_dir=self.temp_dir)
        config2.load()

        self.assertEqual(len(config2.servers), 1)
        self.assertEqual(config2.servers[0].name, "Test Server")
        self.assertEqual(config2.volume, 75)
        self.assertTrue(config2.shuffle)

    def test_get_eq_preset(self):
        """Test getting EQ preset by name"""
        config = AppConfig(config_dir=self.temp_dir)

        preset = config.get_eq_preset("Flat")

        self.assertIsNotNone(preset)
        self.assertEqual(preset.name, "Flat")

    def test_get_nonexistent_eq_preset(self):
        """Test getting non-existent preset returns None"""
        config = AppConfig(config_dir=self.temp_dir)

        preset = config.get_eq_preset("NonExistent")

        self.assertIsNone(preset)

    def test_save_custom_eq_preset(self):
        """Test saving custom EQ preset"""
        config = AppConfig(config_dir=self.temp_dir)
        custom_gains = [5.0] * 18

        config.save_custom_eq_preset("My Preset", custom_gains)

        preset = config.get_eq_preset("My Preset")
        self.assertIsNotNone(preset)
        self.assertEqual(preset.gains[0], 5.0)

    def test_remove_custom_eq_preset(self):
        """Test removing custom EQ preset"""
        config = AppConfig(config_dir=self.temp_dir)
        config.save_custom_eq_preset("Custom", [0.0] * 18)

        config.remove_eq_preset("Custom")

        preset = config.get_eq_preset("Custom")
        self.assertIsNone(preset)

    def test_cannot_remove_builtin_preset(self):
        """Test that built-in presets cannot be removed"""
        config = AppConfig(config_dir=self.temp_dir)

        config.remove_eq_preset("Flat")

        # Should still exist
        preset = config.get_eq_preset("Flat")
        self.assertIsNotNone(preset)

    def test_config_persistence(self):
        """Test config persists across instances"""
        config1 = AppConfig(config_dir=self.temp_dir)
        config1.add_server("Server", "http://s.com", "u", "p")
        config1.volume = 50
        config1.save()

        config2 = AppConfig(config_dir=self.temp_dir)
        config2.load()

        self.assertEqual(config2.volume, 50)
        self.assertEqual(len(config2.servers), 1)


if __name__ == "__main__":
    unittest.main()
