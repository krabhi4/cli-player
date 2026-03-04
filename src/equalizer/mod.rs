use crate::config::models::EQPreset;
use crate::config::AppConfig;
use crate::player::Player;

/// 18-band EQ frequencies in Hz.
pub const EQ_BANDS: [u32; 18] = [
    65, 92, 131, 185, 262, 370, 523, 740, 1047, 1480, 2093, 2960, 4186, 5920, 8372, 11840,
    16744, 20000,
];

pub const EQ_BAND_LABELS: [&str; 18] = [
    "65", "92", "131", "185", "262", "370", "523", "740", "1K", "1.5K", "2.1K", "3K", "4.2K",
    "5.9K", "8.4K", "12K", "17K", "20K",
];

pub const GAIN_MIN: f64 = -12.0;
pub const GAIN_MAX: f64 = 12.0;

pub struct Equalizer {
    pub gains: Vec<f64>,
    pub enabled: bool,
    current_preset: String,
}

impl Equalizer {
    pub fn new(config: &AppConfig) -> Self {
        let gains = if let Some(preset) = config.get_eq_preset(&config.active_eq_preset) {
            preset.gains.clone()
        } else {
            config.custom_eq_gains.clone()
        };

        Self {
            gains,
            enabled: true,
            current_preset: config.active_eq_preset.clone(),
        }
    }

    pub fn apply(&self, player: &mut Player) {
        player.set_eq_gains(&self.gains);
        player.set_eq_enabled(self.enabled);
    }

    pub fn set_band(&mut self, index: usize, gain_db: f64, player: &mut Player) {
        if index < 18 {
            self.gains[index] = gain_db.clamp(GAIN_MIN, GAIN_MAX);
            self.apply(player);
        }
    }

    pub fn set_all_bands(&mut self, gains: &[f64], player: &mut Player) {
        for (i, &g) in gains.iter().take(18).enumerate() {
            self.gains[i] = g.clamp(GAIN_MIN, GAIN_MAX);
        }
        self.apply(player);
    }

    pub fn reset(&mut self, player: &mut Player) {
        self.gains = vec![0.0; 18];
        self.apply(player);
    }

    pub fn toggle(&mut self, player: &mut Player) {
        self.enabled = !self.enabled;
        self.apply(player);
    }

    pub fn load_preset(&mut self, preset_name: &str, config: &mut AppConfig, player: &mut Player) {
        if let Some(preset) = config.get_eq_preset(preset_name) {
            self.gains = preset.gains.clone();
            self.current_preset = preset_name.to_string();
            config.active_eq_preset = preset_name.to_string();
            config.save();
            self.apply(player);
        }
    }

    pub fn save_as_preset(&self, name: &str, config: &mut AppConfig) {
        config.save_custom_eq_preset(name, &self.gains);
    }

    pub fn get_presets(&self, config: &AppConfig) -> Vec<EQPreset> {
        config.eq_presets.clone()
    }

    pub fn current_preset_name(&self) -> &str {
        &self.current_preset
    }

    pub fn band_label(index: usize) -> &'static str {
        if index < 18 {
            EQ_BAND_LABELS[index]
        } else {
            "?"
        }
    }

    pub fn band_frequency(index: usize) -> u32 {
        if index < 18 {
            EQ_BANDS[index]
        } else {
            0
        }
    }
}
