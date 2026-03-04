pub mod crypto;
pub mod models;
pub mod presets;

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

use self::crypto::{decrypt_password, encrypt_password};
use self::models::{EQPreset, ServerConfig};
use self::presets::{default_eq_presets, default_preset_names};

fn default_config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("cli-music-player")
}

pub struct AppConfig {
    config_dir: PathBuf,
    pub servers: Vec<ServerConfig>,
    pub active_server_index: i32,
    pub eq_presets: Vec<EQPreset>,
    pub active_eq_preset: String,
    pub custom_eq_gains: Vec<f64>,
    pub volume: u32,
    pub shuffle: bool,
    pub repeat_mode: String,
    pub audio_device: String,
}

impl AppConfig {
    pub fn load() -> Self {
        let dir = default_config_dir();
        Self::load_from(&dir)
    }

    pub fn load_from(config_dir: &Path) -> Self {
        let mut config = Self {
            config_dir: config_dir.to_path_buf(),
            servers: Vec::new(),
            active_server_index: -1,
            eq_presets: default_eq_presets(),
            active_eq_preset: "Flat".to_string(),
            custom_eq_gains: vec![0.0; 18],
            volume: 75,
            shuffle: false,
            repeat_mode: "off".to_string(),
            audio_device: "auto".to_string(),
        };
        config.load_file();
        config
    }

    fn config_file(&self) -> PathBuf {
        self.config_dir.join("config.json")
    }

    fn load_file(&mut self) {
        let path = self.config_file();
        if !path.exists() {
            return;
        }
        let Ok(content) = fs::read_to_string(&path) else {
            return;
        };
        let Ok(data) = serde_json::from_str::<Value>(&content) else {
            return;
        };

        if let Some(servers) = data["servers"].as_array() {
            self.servers = servers.iter().filter_map(ServerConfig::from_value).collect();
        }
        if let Some(idx) = data["active_server_index"].as_i64() {
            self.active_server_index = idx as i32;
        }
        if let Some(preset) = data["active_eq_preset"].as_str() {
            self.active_eq_preset = preset.to_string();
        }
        if let Some(gains) = data["custom_eq_gains"].as_array() {
            self.custom_eq_gains = gains
                .iter()
                .map(|v| v.as_f64().unwrap_or(0.0))
                .collect();
        }
        if let Some(vol) = data["volume"].as_u64() {
            self.volume = vol as u32;
        }
        if let Some(shuf) = data["shuffle"].as_bool() {
            self.shuffle = shuf;
        }
        if let Some(rm) = data["repeat_mode"].as_str() {
            self.repeat_mode = rm.to_string();
        }
        if let Some(dev) = data["audio_device"].as_str() {
            self.audio_device = dev.to_string();
        }

        // Merge custom presets with defaults
        let default_names = default_preset_names();
        if let Some(custom) = data["custom_eq_presets"].as_array() {
            for cp in custom {
                if let Some(preset) = EQPreset::from_value(cp)
                    && !default_names.contains(&preset.name)
                {
                    self.eq_presets.push(preset);
                }
            }
        }
    }

    pub fn save(&self) {
        let _ = fs::create_dir_all(&self.config_dir);
        let default_names = default_preset_names();
        let custom_presets: Vec<Value> = self
            .eq_presets
            .iter()
            .filter(|p| !default_names.contains(&p.name))
            .map(|p| p.to_value())
            .collect();

        let data = serde_json::json!({
            "servers": self.servers.iter().map(|s| s.to_value()).collect::<Vec<_>>(),
            "active_server_index": self.active_server_index,
            "active_eq_preset": self.active_eq_preset,
            "custom_eq_gains": self.custom_eq_gains,
            "custom_eq_presets": custom_presets,
            "volume": self.volume,
            "shuffle": self.shuffle,
            "repeat_mode": self.repeat_mode,
            "audio_device": self.audio_device,
        });

        let content = serde_json::to_string_pretty(&data).unwrap_or_default();
        let _ = fs::write(self.config_file(), content);
    }

    pub fn active_server(&self) -> Option<&ServerConfig> {
        if self.active_server_index >= 0
            && (self.active_server_index as usize) < self.servers.len()
        {
            Some(&self.servers[self.active_server_index as usize])
        } else {
            None
        }
    }

    pub fn add_server(
        &mut self,
        name: &str,
        url: &str,
        username: &str,
        password: &str,
    ) -> &ServerConfig {
        let server = ServerConfig {
            name: name.to_string(),
            url: url.trim_end_matches('/').to_string(),
            username: username.to_string(),
            encrypted_password: encrypt_password(password),
        };
        self.servers.push(server);
        if self.active_server_index < 0 {
            self.active_server_index = 0;
        }
        self.save();
        self.servers.last().unwrap()
    }

    pub fn remove_server(&mut self, index: usize) {
        if index < self.servers.len() {
            self.servers.remove(index);
            if self.active_server_index >= self.servers.len() as i32 {
                self.active_server_index = self.servers.len() as i32 - 1;
            }
            self.save();
        }
    }

    pub fn set_active_server(&mut self, index: usize) {
        if index < self.servers.len() {
            self.active_server_index = index as i32;
            self.save();
        }
    }

    pub fn get_password(&self, server: Option<&ServerConfig>) -> String {
        let srv = server.or(self.active_server());
        if let Some(s) = srv
            && !s.encrypted_password.is_empty()
        {
            return decrypt_password(&s.encrypted_password).unwrap_or_default();
        }
        String::new()
    }

    pub fn update_server_password(&mut self, index: usize, password: &str) {
        if index < self.servers.len() {
            self.servers[index].encrypted_password = encrypt_password(password);
            self.save();
        }
    }

    pub fn get_eq_preset(&self, name: &str) -> Option<&EQPreset> {
        self.eq_presets.iter().find(|p| p.name == name)
    }

    pub fn save_custom_eq_preset(&mut self, name: &str, gains: &[f64]) {
        let default_names = default_preset_names();
        let actual_name = if default_names.contains(name) {
            format!("{name} (Custom)")
        } else {
            name.to_string()
        };
        self.eq_presets.retain(|p| p.name != actual_name);
        self.eq_presets.push(EQPreset {
            name: actual_name,
            gains: gains.to_vec(),
        });
        self.save();
    }
}
