use cli_music_player::config::crypto::{decrypt_password, encrypt_password};
use cli_music_player::config::models::{EQPreset, ServerConfig};
use cli_music_player::config::presets::default_eq_presets;
use cli_music_player::config::AppConfig;
use serde_json::json;
use tempfile::tempdir;

// ── Password Encryption Tests ───────────────────────────────────

#[test]
fn test_encrypt_decrypt_roundtrip() {
    let password = "my_secret_password";
    let encrypted = encrypt_password(password);
    let decrypted = decrypt_password(&encrypted).unwrap();
    assert_eq!(decrypted, password);
}

#[test]
fn test_encrypt_produces_different_outputs() {
    let password = "test_password";
    let enc1 = encrypt_password(password);
    let enc2 = encrypt_password(password);
    // Different nonces should produce different ciphertexts
    assert_ne!(enc1, enc2);
}

#[test]
fn test_decrypt_invalid_data() {
    let result = decrypt_password("not-valid-base64!!!");
    assert!(result.is_err());
}

#[test]
fn test_decrypt_empty_string() {
    let result = decrypt_password("");
    assert!(result.is_err());
}

#[test]
fn test_encrypt_empty_password() {
    let encrypted = encrypt_password("");
    let decrypted = decrypt_password(&encrypted).unwrap();
    assert_eq!(decrypted, "");
}

#[test]
fn test_encrypt_unicode_password() {
    let password = "пароль_密码_🔐";
    let encrypted = encrypt_password(password);
    let decrypted = decrypt_password(&encrypted).unwrap();
    assert_eq!(decrypted, password);
}

#[test]
fn test_encrypt_long_password() {
    let password = "a".repeat(1000);
    let encrypted = encrypt_password(&password);
    let decrypted = decrypt_password(&encrypted).unwrap();
    assert_eq!(decrypted, password);
}

// ── ServerConfig Tests ──────────────────────────────────────────

#[test]
fn test_server_config_to_value() {
    let server = ServerConfig {
        name: "Test Server".to_string(),
        url: "https://music.example.com".to_string(),
        username: "admin".to_string(),
        encrypted_password: "encrypted123".to_string(),
    };
    let value = server.to_value();
    assert_eq!(value["name"], "Test Server");
    assert_eq!(value["url"], "https://music.example.com");
    assert_eq!(value["username"], "admin");
    assert_eq!(value["_encrypted_password"], "encrypted123");
}

#[test]
fn test_server_config_from_value() {
    let value = json!({
        "name": "My Server",
        "url": "https://nd.example.com",
        "username": "user",
        "_encrypted_password": "enc456"
    });
    let server = ServerConfig::from_value(&value).unwrap();
    assert_eq!(server.name, "My Server");
    assert_eq!(server.url, "https://nd.example.com");
    assert_eq!(server.username, "user");
    assert_eq!(server.encrypted_password, "enc456");
}

#[test]
fn test_server_config_from_value_missing_password() {
    let value = json!({
        "name": "Server",
        "url": "https://example.com",
        "username": "user"
    });
    let server = ServerConfig::from_value(&value).unwrap();
    assert_eq!(server.encrypted_password, "");
}

#[test]
fn test_server_config_from_value_invalid() {
    let value = json!({ "name": "Server" });
    // Missing required fields
    assert!(ServerConfig::from_value(&value).is_none());
}

// ── EQPreset Tests ──────────────────────────────────────────────

#[test]
fn test_eq_preset_to_value() {
    let preset = EQPreset {
        name: "Custom".to_string(),
        gains: vec![1.0; 18],
    };
    let value = preset.to_value();
    assert_eq!(value["name"], "Custom");
    assert_eq!(value["gains"].as_array().unwrap().len(), 18);
}

#[test]
fn test_eq_preset_from_value() {
    let value = json!({
        "name": "My Preset",
        "gains": [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0,
                  10.0, 11.0, 12.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0]
    });
    let preset = EQPreset::from_value(&value).unwrap();
    assert_eq!(preset.name, "My Preset");
    assert_eq!(preset.gains.len(), 18);
    assert_eq!(preset.gains[0], 1.0);
}

// ── AppConfig Tests ─────────────────────────────────────────────

#[test]
fn test_app_config_default_values() {
    let dir = tempdir().unwrap();
    let config = AppConfig::load_from(dir.path());

    assert!(config.servers.is_empty());
    assert_eq!(config.active_server_index, -1);
    assert_eq!(config.volume, 75);
    assert!(!config.shuffle);
    assert_eq!(config.repeat_mode, "off");
    assert_eq!(config.audio_device, "auto");
    assert_eq!(config.active_eq_preset, "Flat");
}

#[test]
fn test_app_config_has_default_presets() {
    let dir = tempdir().unwrap();
    let config = AppConfig::load_from(dir.path());
    let defaults = default_eq_presets();
    assert_eq!(config.eq_presets.len(), defaults.len());
}

#[test]
fn test_app_config_add_server() {
    let dir = tempdir().unwrap();
    let mut config = AppConfig::load_from(dir.path());

    config.add_server("Test", "https://example.com", "user", "pass");

    assert_eq!(config.servers.len(), 1);
    assert_eq!(config.active_server_index, 0);
    assert_eq!(config.servers[0].name, "Test");
}

#[test]
fn test_app_config_remove_server() {
    let dir = tempdir().unwrap();
    let mut config = AppConfig::load_from(dir.path());

    config.add_server("S1", "https://s1.com", "u1", "p1");
    config.add_server("S2", "https://s2.com", "u2", "p2");
    config.set_active_server(1);

    config.remove_server(0);

    assert_eq!(config.servers.len(), 1);
    assert_eq!(config.servers[0].name, "S2");
}

#[test]
fn test_app_config_set_active_server() {
    let dir = tempdir().unwrap();
    let mut config = AppConfig::load_from(dir.path());

    config.add_server("S1", "https://s1.com", "u1", "p1");
    config.add_server("S2", "https://s2.com", "u2", "p2");

    config.set_active_server(1);
    assert_eq!(config.active_server_index, 1);
    assert_eq!(config.active_server().unwrap().name, "S2");
}

#[test]
fn test_app_config_active_server_none() {
    let dir = tempdir().unwrap();
    let config = AppConfig::load_from(dir.path());
    assert!(config.active_server().is_none());
}

#[test]
fn test_app_config_save_and_load() {
    let dir = tempdir().unwrap();

    {
        let mut config = AppConfig::load_from(dir.path());
        config.volume = 50;
        config.shuffle = true;
        config.repeat_mode = "all".to_string();
        config.add_server("Test", "https://example.com", "user", "pass");
        config.save();
    }

    {
        let config = AppConfig::load_from(dir.path());
        assert_eq!(config.volume, 50);
        assert!(config.shuffle);
        assert_eq!(config.repeat_mode, "all");
        assert_eq!(config.servers.len(), 1);
        assert_eq!(config.servers[0].name, "Test");
    }
}

#[test]
fn test_app_config_get_password() {
    let dir = tempdir().unwrap();
    let mut config = AppConfig::load_from(dir.path());
    config.add_server("Test", "https://example.com", "user", "mypassword");

    let password = config.get_password(Some(&config.servers[0]));
    assert_eq!(password, "mypassword");
}

#[test]
fn test_app_config_get_eq_preset() {
    let dir = tempdir().unwrap();
    let config = AppConfig::load_from(dir.path());

    let flat = config.get_eq_preset("Flat");
    assert!(flat.is_some());
    assert!(flat.unwrap().gains.iter().all(|&g| g == 0.0));

    let nonexistent = config.get_eq_preset("NonExistent");
    assert!(nonexistent.is_none());
}

#[test]
fn test_app_config_save_custom_eq_preset() {
    let dir = tempdir().unwrap();
    let mut config = AppConfig::load_from(dir.path());

    let gains = vec![5.0; 18];
    config.save_custom_eq_preset("My Preset", &gains);

    let preset = config.get_eq_preset("My Preset");
    assert!(preset.is_some());
    assert_eq!(preset.unwrap().gains, gains);
}

#[test]
fn test_app_config_custom_preset_not_overwrite_default() {
    let dir = tempdir().unwrap();
    let mut config = AppConfig::load_from(dir.path());

    let gains = vec![5.0; 18];
    config.save_custom_eq_preset("Flat", &gains);

    // Should save as "Flat (Custom)" not overwrite default
    let flat = config.get_eq_preset("Flat").unwrap();
    assert!(flat.gains.iter().all(|&g| g == 0.0));

    let custom = config.get_eq_preset("Flat (Custom)");
    assert!(custom.is_some());
}

#[test]
fn test_app_config_corrupt_file_uses_defaults() {
    let dir = tempdir().unwrap();
    std::fs::write(dir.path().join("config.json"), "not json!!!").unwrap();

    let config = AppConfig::load_from(dir.path());
    assert_eq!(config.volume, 75); // Default
}

#[test]
fn test_app_config_custom_eq_gains_persist() {
    let dir = tempdir().unwrap();

    {
        let mut config = AppConfig::load_from(dir.path());
        config.custom_eq_gains = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0,
                                       10.0, 11.0, 12.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        config.save();
    }

    let config = AppConfig::load_from(dir.path());
    assert_eq!(config.custom_eq_gains[0], 1.0);
    assert_eq!(config.custom_eq_gains[17], 6.0);
}

#[test]
fn test_app_config_audio_device_persists() {
    let dir = tempdir().unwrap();

    {
        let mut config = AppConfig::load_from(dir.path());
        config.audio_device = "My USB DAC".to_string();
        config.save();
    }

    let config = AppConfig::load_from(dir.path());
    assert_eq!(config.audio_device, "My USB DAC");
}

#[test]
fn test_app_config_url_trailing_slash_stripped() {
    let dir = tempdir().unwrap();
    let mut config = AppConfig::load_from(dir.path());
    config.add_server("Test", "https://example.com/", "user", "pass");
    assert_eq!(config.servers[0].url, "https://example.com");
}

#[test]
fn test_app_config_remove_only_server() {
    let dir = tempdir().unwrap();
    let mut config = AppConfig::load_from(dir.path());
    config.add_server("Only", "https://only.com", "u", "p");
    assert_eq!(config.active_server_index, 0);

    config.remove_server(0);
    assert!(config.servers.is_empty());
    assert_eq!(config.active_server_index, -1);
}

#[test]
fn test_app_config_remove_active_server_adjusts_index() {
    let dir = tempdir().unwrap();
    let mut config = AppConfig::load_from(dir.path());
    config.add_server("S1", "https://s1.com", "u1", "p1");
    config.add_server("S2", "https://s2.com", "u2", "p2");
    config.add_server("S3", "https://s3.com", "u3", "p3");
    config.set_active_server(2);

    config.remove_server(2);
    // Active index should be clamped to remaining servers
    assert!(config.active_server_index < config.servers.len() as i32);
}

#[test]
fn test_app_config_get_password_no_server() {
    let dir = tempdir().unwrap();
    let config = AppConfig::load_from(dir.path());
    assert_eq!(config.get_password(None), "");
}

#[test]
fn test_app_config_set_active_server_out_of_bounds() {
    let dir = tempdir().unwrap();
    let mut config = AppConfig::load_from(dir.path());
    config.add_server("S1", "https://s1.com", "u1", "p1");
    config.set_active_server(999); // Should be no-op
    assert_eq!(config.active_server_index, 0);
}

#[test]
fn test_app_config_multiple_custom_presets() {
    let dir = tempdir().unwrap();
    let mut config = AppConfig::load_from(dir.path());

    config.save_custom_eq_preset("Preset A", &vec![1.0; 18]);
    config.save_custom_eq_preset("Preset B", &vec![2.0; 18]);

    assert!(config.get_eq_preset("Preset A").is_some());
    assert!(config.get_eq_preset("Preset B").is_some());
    assert_eq!(config.get_eq_preset("Preset A").unwrap().gains[0], 1.0);
    assert_eq!(config.get_eq_preset("Preset B").unwrap().gains[0], 2.0);
}

#[test]
fn test_app_config_overwrite_custom_preset() {
    let dir = tempdir().unwrap();
    let mut config = AppConfig::load_from(dir.path());

    config.save_custom_eq_preset("My EQ", &vec![3.0; 18]);
    config.save_custom_eq_preset("My EQ", &vec![7.0; 18]);

    let preset = config.get_eq_preset("My EQ").unwrap();
    assert_eq!(preset.gains[0], 7.0);
    // Should not have duplicates
    let count = config.eq_presets.iter().filter(|p| p.name == "My EQ").count();
    assert_eq!(count, 1);
}

// ── Server Edge Cases ───────────────────────────────────────────

#[test]
fn test_app_config_add_many_servers() {
    let dir = tempdir().unwrap();
    let mut config = AppConfig::load_from(dir.path());

    for i in 0..20 {
        config.add_server(
            &format!("Server {i}"),
            &format!("https://s{i}.example.com"),
            &format!("user{i}"),
            &format!("pass{i}"),
        );
    }

    assert_eq!(config.servers.len(), 20);
    // Active stays at 0 (first server added) — add_server only sets index from -1 to 0
    assert_eq!(config.active_server_index, 0);
}

#[test]
fn test_app_config_remove_middle_server_adjusts_active() {
    let dir = tempdir().unwrap();
    let mut config = AppConfig::load_from(dir.path());

    config.add_server("S1", "https://s1.com", "u", "p");
    config.add_server("S2", "https://s2.com", "u", "p");
    config.add_server("S3", "https://s3.com", "u", "p");
    config.set_active_server(2); // S3 is active

    config.remove_server(1); // Remove S2

    // S3 is now at index 1, active index should adjust
    assert_eq!(config.servers.len(), 2);
    assert_eq!(config.servers[0].name, "S1");
    assert_eq!(config.servers[1].name, "S3");
}

#[test]
fn test_app_config_set_active_negative_ignored() {
    let dir = tempdir().unwrap();
    let mut config = AppConfig::load_from(dir.path());
    config.add_server("S1", "https://s1.com", "u", "p");

    // Try invalid index - should be no-op
    config.set_active_server(999);
    // Active should remain at 0 (set during add_server)
    assert_eq!(config.active_server_index, 0);
}

#[test]
fn test_app_config_server_password_survives_save_reload() {
    let dir = tempdir().unwrap();

    {
        let mut config = AppConfig::load_from(dir.path());
        config.add_server("Test", "https://example.com", "admin", "secret_password");
        config.save();
    }

    {
        let config = AppConfig::load_from(dir.path());
        let password = config.get_password(Some(&config.servers[0]));
        assert_eq!(password, "secret_password");
    }
}

#[test]
fn test_app_config_unicode_server_name() {
    let dir = tempdir().unwrap();
    let mut config = AppConfig::load_from(dir.path());
    config.add_server("Сервер 🎵", "https://example.com", "пользователь", "пароль");

    assert_eq!(config.servers[0].name, "Сервер 🎵");

    let password = config.get_password(Some(&config.servers[0]));
    assert_eq!(password, "пароль");
}

// ── Config File Corruption & Recovery ───────────────────────────

#[test]
fn test_app_config_empty_json_file() {
    let dir = tempdir().unwrap();
    std::fs::write(dir.path().join("config.json"), "{}").unwrap();

    let config = AppConfig::load_from(dir.path());
    assert_eq!(config.volume, 75); // Default
    assert!(config.servers.is_empty());
}

#[test]
fn test_app_config_partial_json() {
    let dir = tempdir().unwrap();
    std::fs::write(
        dir.path().join("config.json"),
        r#"{"volume": 42}"#,
    )
    .unwrap();

    let config = AppConfig::load_from(dir.path());
    assert_eq!(config.volume, 42);
    // Other fields should be defaults
    assert!(config.servers.is_empty());
    assert!(!config.shuffle);
}

#[test]
fn test_app_config_extra_unknown_fields_ignored() {
    let dir = tempdir().unwrap();
    std::fs::write(
        dir.path().join("config.json"),
        r#"{"volume": 50, "unknown_field": "ignored", "future_feature": true}"#,
    )
    .unwrap();

    let config = AppConfig::load_from(dir.path());
    assert_eq!(config.volume, 50);
    // Should not crash on unknown fields
}

// ── Password Encryption Edge Cases ──────────────────────────────

#[test]
fn test_encrypt_special_characters() {
    let password = r#"p@$$w0rd!@#$%^&*()_+-={}[]|;':",./<>?"#;
    let encrypted = encrypt_password(password);
    let decrypted = decrypt_password(&encrypted).unwrap();
    assert_eq!(decrypted, password);
}

#[test]
fn test_encrypt_very_long_password() {
    let password = "x".repeat(10000);
    let encrypted = encrypt_password(&password);
    let decrypted = decrypt_password(&encrypted).unwrap();
    assert_eq!(decrypted, password);
}

#[test]
fn test_decrypt_truncated_data() {
    let encrypted = encrypt_password("test");
    // Truncate the base64 string
    let truncated = &encrypted[..encrypted.len() / 2];
    assert!(decrypt_password(truncated).is_err());
}

#[test]
fn test_decrypt_tampered_data() {
    let encrypted = encrypt_password("test");
    // Tamper with some bytes
    let mut bytes = encrypted.as_bytes().to_vec();
    if bytes.len() > 20 {
        bytes[20] ^= 0xFF;
    }
    let tampered = String::from_utf8_lossy(&bytes).to_string();
    // Should fail decryption (authentication tag check)
    assert!(decrypt_password(&tampered).is_err());
}

// ── EQ Preset Persistence Edge Cases ────────────────────────────

#[test]
fn test_app_config_many_custom_presets_persist() {
    let dir = tempdir().unwrap();

    {
        let mut config = AppConfig::load_from(dir.path());
        for i in 0..50 {
            config.save_custom_eq_preset(
                &format!("Custom {i}"),
                &vec![(i as f64) % 12.0; 18],
            );
        }
        config.save();
    }

    {
        let config = AppConfig::load_from(dir.path());
        for i in 0..50 {
            let preset = config.get_eq_preset(&format!("Custom {i}"));
            assert!(preset.is_some(), "Custom preset {i} should persist");
        }
    }
}

#[test]
fn test_eq_preset_from_value_wrong_gains_count() {
    let value = json!({
        "name": "Bad Preset",
        "gains": [1.0, 2.0, 3.0]
    });
    // Should still parse (gains will just have 3 elements)
    let preset = EQPreset::from_value(&value);
    assert!(preset.is_some());
    assert_eq!(preset.unwrap().gains.len(), 3);
}

#[test]
fn test_eq_preset_from_value_missing_name() {
    let value = json!({
        "gains": [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
                  0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0]
    });
    // Missing name should return None
    assert!(EQPreset::from_value(&value).is_none());
}

#[test]
fn test_server_config_roundtrip() {
    let original = ServerConfig {
        name: "Test Server".to_string(),
        url: "https://music.example.com".to_string(),
        username: "admin".to_string(),
        encrypted_password: "enc123".to_string(),
    };
    let value = original.to_value();
    let restored = ServerConfig::from_value(&value).unwrap();

    assert_eq!(original.name, restored.name);
    assert_eq!(original.url, restored.url);
    assert_eq!(original.username, restored.username);
    assert_eq!(original.encrypted_password, restored.encrypted_password);
}

// ── Volume & Settings Bounds ────────────────────────────────────

#[test]
fn test_app_config_volume_boundaries() {
    let dir = tempdir().unwrap();

    {
        let mut config = AppConfig::load_from(dir.path());
        config.volume = 0;
        config.save();
    }
    {
        let config = AppConfig::load_from(dir.path());
        assert_eq!(config.volume, 0);
    }

    {
        let mut config = AppConfig::load_from(dir.path());
        config.volume = 100;
        config.save();
    }
    {
        let config = AppConfig::load_from(dir.path());
        assert_eq!(config.volume, 100);
    }
}

#[test]
fn test_app_config_active_server_none_after_clear() {
    let dir = tempdir().unwrap();
    let mut config = AppConfig::load_from(dir.path());
    config.add_server("S1", "https://s1.com", "u", "p");
    config.add_server("S2", "https://s2.com", "u", "p");

    // Remove all servers
    config.remove_server(1);
    config.remove_server(0);

    assert!(config.servers.is_empty());
    assert!(config.active_server().is_none());
    assert_eq!(config.active_server_index, -1);
}
