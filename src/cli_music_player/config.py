"""Configuration and credential management for CLI Music Player."""

import base64
import hashlib
import json
import os
from dataclasses import dataclass, field
from pathlib import Path

CONFIG_DIR = Path.home() / ".config" / "cli-music-player"
CONFIG_FILE = CONFIG_DIR / "config.json"


@dataclass
class ServerConfig:
    """Configuration for a single Navidrome server."""

    name: str
    url: str
    username: str
    # Password stored encrypted in config
    _encrypted_password: str = ""

    def to_dict(self) -> dict:
        return {
            "name": self.name,
            "url": self.url.rstrip("/"),
            "username": self.username,
            "_encrypted_password": self._encrypted_password,
        }

    @classmethod
    def from_dict(cls, data: dict) -> "ServerConfig":
        return cls(
            name=data["name"],
            url=data["url"],
            username=data["username"],
            _encrypted_password=data.get("_encrypted_password", ""),
        )


@dataclass
class EQPreset:
    """An equalizer preset with 18 band gains."""

    name: str
    gains: list[float] = field(default_factory=lambda: [0.0] * 18)

    def to_dict(self) -> dict:
        return {"name": self.name, "gains": self.gains}

    @classmethod
    def from_dict(cls, data: dict) -> "EQPreset":
        return cls(name=data["name"], gains=data["gains"])


# ─── Built-in EQ Presets ────────────────────────────────────────────
DEFAULT_EQ_PRESETS: list[EQPreset] = [
    EQPreset("Flat", [0.0] * 18),
    EQPreset(
        "Bass Boost",
        [10, 8, 6, 4, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    ),
    EQPreset(
        "Treble Boost",
        [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 2, 4, 6, 8, 10, 10, 10],
    ),
    EQPreset(
        "Vocal",
        [-2, -2, -1, 0, 2, 4, 5, 5, 4, 3, 2, 1, 0, -1, -2, -2, -3, -3],
    ),
    EQPreset(
        "Rock",
        [5, 4, 3, 2, -1, -2, -1, 1, 3, 4, 5, 5, 4, 3, 2, 1, 0, 0],
    ),
    EQPreset(
        "Pop",
        [-2, -1, 0, 2, 4, 5, 4, 2, 0, -1, -2, -1, 0, 2, 3, 4, 3, 2],
    ),
    EQPreset(
        "Jazz",
        [3, 2, 1, 2, -1, -1, 0, 1, 2, 3, 3, 3, 2, 2, 3, 3, 4, 4],
    ),
    EQPreset(
        "Classical",
        [4, 3, 2, 1, -1, -1, 0, 0, 1, 2, 2, 3, 3, 2, 1, 2, 3, 4],
    ),
    EQPreset(
        "Electronic",
        [6, 5, 4, 2, 0, -2, -1, 0, 1, 2, 0, -1, 0, 2, 4, 5, 6, 5],
    ),
    EQPreset(
        "Loudness",
        [6, 5, 3, 0, -2, -3, -2, 0, 0, 1, 2, 4, 5, 3, 0, -1, 2, 5],
    ),
]


def _derive_key() -> bytes:
    """Derive a machine-specific encryption key for password storage."""
    # Use a combination of hostname and username as salt
    try:
        username = os.getlogin()
    except OSError:
        username = os.environ.get("USER", os.environ.get("USERNAME", "default"))
    machine_id = f"{os.uname().nodename}:{username}"
    return hashlib.pbkdf2_hmac(
        "sha256",
        machine_id.encode(),
        b"cli-music-player-salt-v1",
        100000,
    )[:32]


def encrypt_password(password: str) -> str:
    """Encrypt a password for storage."""
    from cryptography.fernet import Fernet  # noqa: PLC0415

    key = base64.urlsafe_b64encode(_derive_key())
    f = Fernet(key)
    return f.encrypt(password.encode()).decode()


def decrypt_password(encrypted: str) -> str:
    """Decrypt a stored password.

    Raises:
        ValueError: If the encrypted password cannot be decrypted (e.g., wrong key/machine).
    """
    from cryptography.fernet import Fernet, InvalidToken  # noqa: PLC0415

    key = base64.urlsafe_b64encode(_derive_key())
    f = Fernet(key)
    try:
        return f.decrypt(encrypted.encode()).decode()
    except InvalidToken as e:
        raise ValueError(
            "Cannot decrypt password. This may happen if the password was "
            "encrypted on a different machine. Please re-enter your password."
        ) from e


class AppConfig:
    """Application configuration manager."""

    def __init__(self):
        self.servers: list[ServerConfig] = []
        self.active_server_index: int = -1
        self.eq_presets: list[EQPreset] = list(DEFAULT_EQ_PRESETS)
        self.active_eq_preset: str = "Flat"
        self.custom_eq_gains: list[float] = [0.0] * 18
        self.volume: int = 75
        self.shuffle: bool = False
        self.repeat_mode: str = "off"  # off, all, one
        self.audio_device: str = "auto"  # alsa audio device
        self._load()

    def _ensure_dir(self):
        CONFIG_DIR.mkdir(parents=True, exist_ok=True)

    def _load(self):
        """Load configuration from disk."""
        if not CONFIG_FILE.exists():
            return
        try:
            with CONFIG_FILE.open() as f:
                data = json.load(f)
            self.servers = [ServerConfig.from_dict(s) for s in data.get("servers", [])]
            self.active_server_index = data.get("active_server_index", -1)
            self.active_eq_preset = data.get("active_eq_preset", "Flat")
            self.custom_eq_gains = data.get("custom_eq_gains", [0.0] * 18)
            self.volume = data.get("volume", 75)
            self.shuffle = data.get("shuffle", False)
            self.repeat_mode = data.get("repeat_mode", "off")
            self.audio_device = data.get("audio_device", "auto")

            # Load custom presets (merge with defaults)
            custom_presets = data.get("custom_eq_presets", [])
            default_names = {p.name for p in DEFAULT_EQ_PRESETS}
            self.eq_presets = list(DEFAULT_EQ_PRESETS) + [
                EQPreset.from_dict(p) for p in custom_presets if p["name"] not in default_names
            ]
        except (json.JSONDecodeError, KeyError):
            pass  # Use defaults on corrupt config

    def save(self):
        """Save configuration to disk."""
        self._ensure_dir()
        default_names = {p.name for p in DEFAULT_EQ_PRESETS}
        custom_presets = [p.to_dict() for p in self.eq_presets if p.name not in default_names]
        data = {
            "servers": [s.to_dict() for s in self.servers],
            "active_server_index": self.active_server_index,
            "active_eq_preset": self.active_eq_preset,
            "custom_eq_gains": self.custom_eq_gains,
            "custom_eq_presets": custom_presets,
            "volume": self.volume,
            "shuffle": self.shuffle,
            "repeat_mode": self.repeat_mode,
            "audio_device": self.audio_device,
        }
        with CONFIG_FILE.open("w") as f:
            json.dump(data, f, indent=2)

    @property
    def active_server(self) -> ServerConfig | None:
        if 0 <= self.active_server_index < len(self.servers):
            return self.servers[self.active_server_index]
        return None

    def add_server(self, name: str, url: str, username: str, password: str) -> ServerConfig:
        """Add a new server configuration."""
        server = ServerConfig(
            name=name,
            url=url.rstrip("/"),
            username=username,
            _encrypted_password=encrypt_password(password),
        )
        self.servers.append(server)
        self.active_server_index = max(self.active_server_index, 0)
        self.save()
        return server

    def remove_server(self, index: int):
        """Remove a server by index."""
        if 0 <= index < len(self.servers):
            self.servers.pop(index)
            if self.active_server_index >= len(self.servers):
                self.active_server_index = len(self.servers) - 1
            self.save()

    def set_active_server(self, index: int):
        """Switch active server."""
        if 0 <= index < len(self.servers):
            self.active_server_index = index
            self.save()

    def get_password(self, server: ServerConfig | None = None) -> str:
        """Get decrypted password for a server.

        Returns:
            The decrypted password, or empty string if decryption fails.
        """
        if server is None:
            server = self.active_server
        if server and server._encrypted_password:
            try:
                return decrypt_password(server._encrypted_password)
            except ValueError:
                # Password cannot be decrypted (e.g., encrypted on different machine)
                # Clear the invalid password and return empty string
                server._encrypted_password = ""
                self.save()
                return ""
        return ""

    def update_server_password(self, index: int, password: str):
        """Update a server's password."""
        if 0 <= index < len(self.servers):
            self.servers[index]._encrypted_password = encrypt_password(password)
            self.save()

    def get_eq_preset(self, name: str) -> EQPreset | None:
        """Get an EQ preset by name."""
        for p in self.eq_presets:
            if p.name == name:
                return p
        return None

    def save_custom_eq_preset(self, name: str, gains: list[float]):
        """Save a custom EQ preset."""
        # Remove existing with same name (if custom)
        default_names = {p.name for p in DEFAULT_EQ_PRESETS}
        if name in default_names:
            name = f"{name} (Custom)"
        self.eq_presets = [p for p in self.eq_presets if p.name != name]
        self.eq_presets.append(EQPreset(name, list(gains)))
        self.save()
