use cli_music_player::audio::pipeline::{
    AudioCommand, AudioEvent, PlaybackState, convert_channels,
};
use cli_music_player::config::AppConfig;
use cli_music_player::equalizer::{EQ_BAND_LABELS, EQ_BANDS, Equalizer, GAIN_MAX, GAIN_MIN};
use cli_music_player::queue::{QueueManager, RepeatMode};
use cli_music_player::subsonic::Song;
use tempfile::tempdir;

fn make_song(id: &str, title: &str, duration: u64) -> Song {
    Song {
        id: id.to_string(),
        title: title.to_string(),
        artist: "Test Artist".to_string(),
        album: "Test Album".to_string(),
        duration,
        ..Default::default()
    }
}

fn make_songs(n: usize) -> Vec<Song> {
    (0..n)
        .map(|i| make_song(&format!("s{i}"), &format!("Song {i}"), 180))
        .collect()
}

// ── PlaybackState Tests ─────────────────────────────────────────

#[test]
fn test_playback_states_are_distinct() {
    let states = [
        PlaybackState::Stopped,
        PlaybackState::Playing,
        PlaybackState::Paused,
    ];
    for i in 0..states.len() {
        assert_eq!(states[i], states[i]);
        for j in (i + 1)..states.len() {
            assert_ne!(states[i], states[j]);
        }
    }
}

#[test]
fn test_playback_state_copy_semantics() {
    let state = PlaybackState::Playing;
    let copied = state;
    assert_eq!(state, copied);
}

// ── Scrobble Logic Tests ────────────────────────────────────────

struct ScrobbleTracker {
    reported: bool,
    scrobble_count: u32,
}

impl ScrobbleTracker {
    fn new() -> Self {
        Self {
            reported: false,
            scrobble_count: 0,
        }
    }

    fn check(&mut self, position: f64, duration: f64) {
        if self.reported || duration <= 0.0 {
            return;
        }
        if position >= duration * 0.5 || position >= 240.0 {
            self.reported = true;
            self.scrobble_count += 1;
        }
    }

    fn reset(&mut self) {
        self.reported = false;
    }
}

#[test]
fn test_scrobble_triggers_at_50_percent() {
    let mut tracker = ScrobbleTracker::new();
    tracker.check(100.0, 300.0);
    assert!(!tracker.reported);
    tracker.check(149.0, 300.0);
    assert!(!tracker.reported);
    tracker.check(150.0, 300.0);
    assert!(tracker.reported);
    assert_eq!(tracker.scrobble_count, 1);
}

#[test]
fn test_scrobble_triggers_at_240s_for_long_tracks() {
    let mut tracker = ScrobbleTracker::new();
    // For a 600s track, 50% = 300s. But 240s triggers first.
    tracker.check(239.0, 600.0);
    assert!(!tracker.reported);
    tracker.check(240.0, 600.0);
    assert!(tracker.reported);
    assert_eq!(tracker.scrobble_count, 1);
}

#[test]
fn test_scrobble_no_double_reporting() {
    let mut tracker = ScrobbleTracker::new();
    tracker.check(150.0, 300.0);
    tracker.check(200.0, 300.0);
    tracker.check(250.0, 300.0);
    assert_eq!(tracker.scrobble_count, 1);
}

#[test]
fn test_scrobble_resets_between_tracks() {
    let mut tracker = ScrobbleTracker::new();
    tracker.check(150.0, 300.0);
    assert!(tracker.reported);

    tracker.reset();
    assert!(!tracker.reported);
    tracker.check(150.0, 300.0);
    assert_eq!(tracker.scrobble_count, 2);
}

#[test]
fn test_scrobble_ignores_zero_duration() {
    let mut tracker = ScrobbleTracker::new();
    tracker.check(100.0, 0.0);
    assert!(!tracker.reported);
}

// ── Queue + Playback Integration Tests ──────────────────────────

#[test]
fn test_queue_set_and_play_first() {
    let mut queue = QueueManager::new();
    let songs = make_songs(5);
    queue.set_queue(songs, 0);

    let current = queue.current_song().unwrap();
    assert_eq!(current.id, "s0");
}

#[test]
fn test_queue_play_through_all() {
    let mut queue = QueueManager::new();
    queue.set_queue(make_songs(3), 0);

    assert_eq!(queue.current_song().unwrap().id, "s0");
    assert_eq!(queue.next().unwrap().id, "s1");
    assert_eq!(queue.next().unwrap().id, "s2");
    assert!(queue.next().is_none());
}

#[test]
fn test_queue_play_through_with_repeat_all() {
    let mut queue = QueueManager::new();
    queue.set_queue(make_songs(3), 0);
    queue.set_repeat(RepeatMode::All);

    queue.next(); // s1
    queue.next(); // s2
    let wrapped = queue.next().unwrap();
    assert_eq!(wrapped.id, "s0");
}

#[test]
fn test_queue_repeat_one_never_advances() {
    let mut queue = QueueManager::new();
    queue.set_queue(make_songs(3), 1);
    queue.set_repeat(RepeatMode::One);

    for _ in 0..10 {
        assert_eq!(queue.next().unwrap().id, "s1");
        assert_eq!(queue.current_index(), 1);
    }
}

#[test]
fn test_queue_previous_navigates_back() {
    let mut queue = QueueManager::new();
    queue.set_queue(make_songs(5), 3);
    assert_eq!(queue.previous().unwrap().id, "s2");
    assert_eq!(queue.previous().unwrap().id, "s1");
    assert_eq!(queue.previous().unwrap().id, "s0");
    assert!(queue.previous().is_none());
}

#[test]
fn test_queue_previous_with_repeat_all_wraps() {
    let mut queue = QueueManager::new();
    queue.set_queue(make_songs(3), 0);
    queue.set_repeat(RepeatMode::All);
    let wrapped = queue.previous().unwrap();
    assert_eq!(wrapped.id, "s2");
}

#[test]
fn test_queue_jump_and_continue() {
    let mut queue = QueueManager::new();
    queue.set_queue(make_songs(5), 0);
    queue.jump_to(3);
    assert_eq!(queue.current_song().unwrap().id, "s3");
    assert_eq!(queue.next().unwrap().id, "s4");
    assert!(queue.next().is_none());
}

#[test]
fn test_queue_add_next_inserts_after_current() {
    let mut queue = QueueManager::new();
    queue.set_queue(make_songs(3), 1);
    queue.add_next(make_song("new", "Inserted", 100));

    assert_eq!(queue.length(), 4);
    assert_eq!(queue.next().unwrap().id, "new");
    assert_eq!(queue.next().unwrap().id, "s2");
}

#[test]
fn test_queue_add_songs_batch() {
    let mut queue = QueueManager::new();
    queue.set_queue(make_songs(2), 0);
    queue.add_songs(vec![
        make_song("a", "Added 1", 100),
        make_song("b", "Added 2", 100),
    ]);
    assert_eq!(queue.length(), 4);
    assert_eq!(queue.queue()[2].id, "a");
    assert_eq!(queue.queue()[3].id, "b");
}

#[test]
fn test_queue_has_next_and_prev() {
    let mut queue = QueueManager::new();
    queue.set_queue(make_songs(3), 0);

    assert!(!queue.has_prev());
    assert!(queue.has_next());

    queue.jump_to(1);
    assert!(queue.has_prev());
    assert!(queue.has_next());

    queue.jump_to(2);
    assert!(queue.has_prev());
    assert!(!queue.has_next());
}

#[test]
fn test_queue_has_next_with_repeat() {
    let mut queue = QueueManager::new();
    queue.set_queue(make_songs(3), 2);
    assert!(!queue.has_next());

    queue.set_repeat(RepeatMode::All);
    assert!(queue.has_next());

    queue.set_repeat(RepeatMode::One);
    assert!(queue.has_next());
}

#[test]
fn test_queue_get_upcoming() {
    let mut queue = QueueManager::new();
    queue.set_queue(make_songs(10), 3);

    let upcoming = queue.get_upcoming(3);
    assert_eq!(upcoming.len(), 3);
    assert_eq!(upcoming[0].id, "s4");
    assert_eq!(upcoming[1].id, "s5");
    assert_eq!(upcoming[2].id, "s6");
}

#[test]
fn test_queue_get_upcoming_at_end() {
    let mut queue = QueueManager::new();
    queue.set_queue(make_songs(5), 4);
    let upcoming = queue.get_upcoming(3);
    assert!(upcoming.is_empty());
}

#[test]
fn test_queue_get_upcoming_partial() {
    let mut queue = QueueManager::new();
    queue.set_queue(make_songs(5), 3);
    let upcoming = queue.get_upcoming(5);
    assert_eq!(upcoming.len(), 1);
    assert_eq!(upcoming[0].id, "s4");
}

#[test]
fn test_queue_total_duration() {
    let mut queue = QueueManager::new();
    queue.set_queue(
        vec![
            make_song("a", "A", 60),
            make_song("b", "B", 120),
            make_song("c", "C", 180),
        ],
        0,
    );
    assert_eq!(queue.total_duration(), 360);
}

#[test]
fn test_queue_total_duration_empty() {
    let queue = QueueManager::new();
    assert_eq!(queue.total_duration(), 0);
}

// ── Equalizer State Management Tests ────────────────────────────

#[test]
fn test_equalizer_new_loads_flat_preset() {
    let dir = tempdir().unwrap();
    let config = AppConfig::load_from(dir.path());
    let eq = Equalizer::new(&config);
    assert!(eq.gains.iter().all(|&g| g == 0.0));
    assert!(eq.enabled);
    assert_eq!(eq.current_preset_name(), "Flat");
}

#[test]
fn test_equalizer_band_labels_complete() {
    assert_eq!(EQ_BAND_LABELS.len(), 18);
    assert_eq!(EQ_BAND_LABELS[0], "65");
    assert_eq!(EQ_BAND_LABELS[8], "1K");
    assert_eq!(EQ_BAND_LABELS[17], "20K");
}

#[test]
fn test_equalizer_bands_match_labels() {
    assert_eq!(EQ_BANDS.len(), EQ_BAND_LABELS.len());
    assert_eq!(EQ_BANDS[0], 65);
    assert_eq!(EQ_BANDS[17], 20000);
}

#[test]
fn test_equalizer_gain_limits() {
    assert_eq!(GAIN_MIN, -12.0);
    assert_eq!(GAIN_MAX, 12.0);
    assert!(GAIN_MIN < 0.0);
    assert!(GAIN_MAX > 0.0);
    assert_eq!(GAIN_MIN, -GAIN_MAX);
}

// ── RepeatMode Config Roundtrip ─────────────────────────────────

#[test]
fn test_repeat_mode_config_roundtrip() {
    for mode in [RepeatMode::Off, RepeatMode::All, RepeatMode::One] {
        let s = mode.as_str();
        let parsed = RepeatMode::from_config_str(s);
        assert_eq!(parsed, mode);
    }
}

#[test]
fn test_repeat_mode_from_invalid_string() {
    assert_eq!(RepeatMode::from_config_str("invalid"), RepeatMode::Off);
    assert_eq!(RepeatMode::from_config_str(""), RepeatMode::Off);
}

// ── Config Server Password Tests ────────────────────────────────

#[test]
fn test_config_update_server_password() {
    let dir = tempdir().unwrap();
    let mut config = AppConfig::load_from(dir.path());
    config.add_server("Test", "https://example.com", "user", "old_pass");

    let original_password = config.get_password(Some(&config.servers[0]));
    assert_eq!(original_password, "old_pass");

    config.update_server_password(0, "new_pass");
    let updated_password = config.get_password(Some(&config.servers[0]));
    assert_eq!(updated_password, "new_pass");
}

#[test]
fn test_config_update_password_out_of_bounds() {
    let dir = tempdir().unwrap();
    let mut config = AppConfig::load_from(dir.path());
    config.add_server("Test", "https://example.com", "user", "pass");
    config.update_server_password(999, "new_pass"); // Should be a no-op
    let password = config.get_password(Some(&config.servers[0]));
    assert_eq!(password, "pass");
}

// ── Config Custom EQ Preset Persistence ─────────────────────────

#[test]
fn test_config_custom_eq_preset_survives_reload() {
    let dir = tempdir().unwrap();

    {
        let mut config = AppConfig::load_from(dir.path());
        let gains = vec![3.0; 18];
        config.save_custom_eq_preset("My EQ", &gains);
        config.save();
    }

    {
        let config = AppConfig::load_from(dir.path());
        let preset = config.get_eq_preset("My EQ");
        assert!(preset.is_some());
        assert!(preset.unwrap().gains.iter().all(|&g| g == 3.0));
    }
}

// ── Shuffle Integration Tests ───────────────────────────────────

#[test]
fn test_shuffle_prev_uses_history() {
    let mut queue = QueueManager::new();
    queue.set_queue(make_songs(20), 0);
    queue.set_shuffle(true);

    let first_id = queue.current_song().unwrap().id.clone();
    let _next_song = queue.next().unwrap().id.to_string();

    // Previous should return to the first song
    let prev = queue.previous().unwrap();
    assert_eq!(prev.id, first_id);

    // Forward again should go to a random song (not necessarily the same)
    let _ = queue.next();
    assert!(queue.history().len() >= 1);
}

#[test]
fn test_shuffle_plays_all_songs_eventually() {
    let mut queue = QueueManager::new();
    queue.set_queue(make_songs(5), 0);
    queue.set_shuffle(true);
    queue.set_repeat(RepeatMode::All);

    let mut seen = std::collections::HashSet::new();
    seen.insert(queue.current_song().unwrap().id.clone());

    // With 5 songs and repeat, we should see all within ~50 advances
    for _ in 0..50 {
        let song = queue.next().unwrap();
        seen.insert(song.id.clone());
        if seen.len() == 5 {
            break;
        }
    }
    assert_eq!(seen.len(), 5);
}

// ── Scrobble Edge Cases ──────────────────────────────────────────

#[test]
fn test_scrobble_exactly_at_threshold() {
    let mut tracker = ScrobbleTracker::new();
    // Exactly at 50% boundary
    tracker.check(150.0, 300.0);
    assert!(tracker.reported);
    assert_eq!(tracker.scrobble_count, 1);
}

#[test]
fn test_scrobble_short_track() {
    let mut tracker = ScrobbleTracker::new();
    // Very short track (10s), 50% = 5s
    tracker.check(4.9, 10.0);
    assert!(!tracker.reported);
    tracker.check(5.0, 10.0);
    assert!(tracker.reported);
}

#[test]
fn test_scrobble_very_long_track_240s_wins() {
    let mut tracker = ScrobbleTracker::new();
    // 1 hour track, 50% = 1800s, but 240s triggers first
    tracker.check(239.9, 3600.0);
    assert!(!tracker.reported);
    tracker.check(240.0, 3600.0);
    assert!(tracker.reported);
}

#[test]
fn test_scrobble_negative_position_ignored() {
    let mut tracker = ScrobbleTracker::new();
    tracker.check(-1.0, 300.0);
    assert!(!tracker.reported);
}

#[test]
fn test_scrobble_negative_duration_ignored() {
    let mut tracker = ScrobbleTracker::new();
    tracker.check(100.0, -1.0);
    assert!(!tracker.reported);
}

#[test]
fn test_scrobble_multiple_resets() {
    let mut tracker = ScrobbleTracker::new();

    // Simulate 5 track plays
    for i in 0..5 {
        tracker.check(200.0, 300.0);
        assert!(tracker.reported);
        tracker.reset();
        assert!(!tracker.reported);
        assert_eq!(tracker.scrobble_count, (i + 1) as u32);
    }
}

// ── Queue + Playback Edge Cases ─────────────────────────────────

#[test]
fn test_queue_play_through_then_wrap_with_repeat() {
    let mut queue = QueueManager::new();
    queue.set_queue(make_songs(3), 0);
    queue.set_repeat(RepeatMode::All);

    // Play through entire queue twice
    let mut played: Vec<String> = vec![queue.current_song().unwrap().id.clone()];
    for _ in 0..5 {
        played.push(queue.next().unwrap().id.to_string());
    }

    // Should wrap: s0, s1, s2, s0, s1, s2
    assert_eq!(played[0], "s0");
    assert_eq!(played[3], "s0");
    assert_eq!(played[5], "s2");
}

#[test]
fn test_queue_set_and_immediately_remove() {
    let mut queue = QueueManager::new();
    queue.set_queue(make_songs(3), 0);

    // Remove current song immediately
    queue.remove(0);
    assert_eq!(queue.length(), 2);

    // Should be able to play remaining songs
    if queue.current_index() >= 0 {
        let song = queue.current_song().unwrap();
        assert_eq!(song.id, "s1");
    }
}

#[test]
fn test_queue_add_next_preserves_playback_order() {
    let mut queue = QueueManager::new();
    queue.set_queue(make_songs(5), 2); // Current: s2

    let inserted = make_song("ins", "Inserted", 100);
    queue.add_next(inserted);

    // Next should be the inserted song
    let next = queue.next().unwrap();
    assert_eq!(next.id, "ins");

    // Then continue with s3
    let after = queue.next().unwrap();
    assert_eq!(after.id, "s3");
}

// ── Equalizer State Edge Cases ──────────────────────────────────

#[test]
fn test_equalizer_loads_saved_preset() {
    let dir = tempdir().unwrap();
    let mut config = AppConfig::load_from(dir.path());
    let gains = vec![5.0; 18];
    config.save_custom_eq_preset("Custom EQ", &gains);
    config.active_eq_preset = "Custom EQ".to_string();
    config.save();

    let config = AppConfig::load_from(dir.path());
    let eq = Equalizer::new(&config);
    assert_eq!(eq.gains, gains);
    assert_eq!(eq.current_preset_name(), "Custom EQ");
}

#[test]
fn test_equalizer_missing_preset_falls_back_flat() {
    let dir = tempdir().unwrap();
    let mut config = AppConfig::load_from(dir.path());
    config.active_eq_preset = "NonExistent Preset".to_string();
    config.save();

    let config = AppConfig::load_from(dir.path());
    let eq = Equalizer::new(&config);
    // Should fall back to Flat when preset doesn't exist
    assert!(eq.gains.iter().all(|&g| g == 0.0));
}

#[test]
fn test_equalizer_toggle_state() {
    let dir = tempdir().unwrap();
    let config = AppConfig::load_from(dir.path());
    let mut eq = Equalizer::new(&config);

    assert!(eq.enabled);
    eq.enabled = false;
    assert!(!eq.enabled);
}

// ── Config Concurrent Access Simulation ─────────────────────────

#[test]
fn test_config_rapid_save_load_cycle() {
    let dir = tempdir().unwrap();

    for i in 0..20 {
        let mut config = AppConfig::load_from(dir.path());
        config.volume = i;
        config.add_server(
            &format!("Server{i}"),
            &format!("https://s{i}.com"),
            "user",
            "pass",
        );
        config.save();
    }

    let final_config = AppConfig::load_from(dir.path());
    assert_eq!(final_config.volume, 19);
    assert_eq!(final_config.servers.len(), 20);
}

#[test]
fn test_config_server_removal_preserves_passwords() {
    let dir = tempdir().unwrap();
    let mut config = AppConfig::load_from(dir.path());

    config.add_server("S1", "https://s1.com", "u1", "pass1");
    config.add_server("S2", "https://s2.com", "u2", "pass2");
    config.add_server("S3", "https://s3.com", "u3", "pass3");

    // Remove middle server
    config.remove_server(1);

    // S1 and S3 passwords should still work
    assert_eq!(config.get_password(Some(&config.servers[0])), "pass1");
    assert_eq!(config.get_password(Some(&config.servers[1])), "pass3");
}

// ── Playback State Completeness ─────────────────────────────────

#[test]
fn test_playback_state_debug_format() {
    // Verify Debug is derived and works
    let state = PlaybackState::Playing;
    let debug = format!("{:?}", state);
    assert!(debug.contains("Playing"));
}

#[test]
fn test_playback_state_clone() {
    let state = PlaybackState::Paused;
    let cloned = state;
    assert_eq!(state, cloned);
}

// ── Equalizer Preset Cycling Tests ──────────────────────────────

#[test]
fn test_equalizer_get_presets_returns_all_defaults() {
    let dir = tempdir().unwrap();
    let config = AppConfig::load_from(dir.path());
    let eq = Equalizer::new(&config);
    let presets = eq.get_presets(&config);
    assert_eq!(presets.len(), 10);
    let names: Vec<&str> = presets.iter().map(|p| p.name.as_str()).collect();
    assert!(names.contains(&"Flat"));
    assert!(names.contains(&"Bass Boost"));
    assert!(names.contains(&"Treble Boost"));
    assert!(names.contains(&"Rock"));
    assert!(names.contains(&"Pop"));
    assert!(names.contains(&"Jazz"));
    assert!(names.contains(&"Classical"));
    assert!(names.contains(&"Electronic"));
    assert!(names.contains(&"Loudness"));
    assert!(names.contains(&"Vocal"));
}

#[test]
fn test_equalizer_get_presets_includes_custom() {
    let dir = tempdir().unwrap();
    let mut config = AppConfig::load_from(dir.path());
    config.save_custom_eq_preset("My Custom", &vec![1.0; 18]);
    config.save();

    let config = AppConfig::load_from(dir.path());
    let eq = Equalizer::new(&config);
    let presets = eq.get_presets(&config);
    assert_eq!(presets.len(), 11);
    assert!(presets.iter().any(|p| p.name == "My Custom"));
}

#[test]
fn test_equalizer_preset_cycling_order() {
    let dir = tempdir().unwrap();
    let config = AppConfig::load_from(dir.path());
    let eq = Equalizer::new(&config);
    let presets = eq.get_presets(&config);

    // Simulate cycling through all presets
    let mut current = "Flat".to_string();
    let mut visited = vec![current.clone()];
    for _ in 0..presets.len() {
        let idx = presets.iter().position(|p| p.name == current).unwrap_or(0);
        let next_idx = (idx + 1) % presets.len();
        current = presets[next_idx].name.clone();
        visited.push(current.clone());
    }
    // After cycling through all presets, should return to Flat
    assert_eq!(visited.last().unwrap(), "Flat");
    // Should have visited all presets
    assert_eq!(visited.len(), presets.len() + 1);
}

#[test]
fn test_equalizer_preset_reverse_cycling() {
    let dir = tempdir().unwrap();
    let config = AppConfig::load_from(dir.path());
    let eq = Equalizer::new(&config);
    let presets = eq.get_presets(&config);

    // Simulate reverse cycling (Shift+P)
    let idx = presets.iter().position(|p| p.name == "Flat").unwrap_or(0);
    let prev_idx = if idx == 0 { presets.len() - 1 } else { idx - 1 };
    // Previous preset from Flat should be the last preset
    assert_eq!(presets[prev_idx].name, presets.last().unwrap().name);
}

#[test]
fn test_equalizer_load_preset_changes_gains() {
    let dir = tempdir().unwrap();
    let config = AppConfig::load_from(dir.path());
    let eq = Equalizer::new(&config);

    // Initially flat
    assert!(eq.gains.iter().all(|&g| g == 0.0));

    // Load bass boost (can't call load_preset without Player, so test via config)
    let bass_boost = config.get_eq_preset("Bass Boost").unwrap();
    assert!(
        bass_boost.gains[0] > 0.0,
        "Bass Boost should have positive first band"
    );
    assert_eq!(bass_boost.gains.len(), 18);
}

#[test]
fn test_equalizer_preset_name_tracks_loaded_preset() {
    let dir = tempdir().unwrap();
    let mut config = AppConfig::load_from(dir.path());
    config.active_eq_preset = "Rock".to_string();
    config.save();

    let config = AppConfig::load_from(dir.path());
    let eq = Equalizer::new(&config);
    assert_eq!(eq.current_preset_name(), "Rock");
    // Gains should match Rock preset
    let rock = config.get_eq_preset("Rock").unwrap();
    assert_eq!(eq.gains, rock.gains);
}

// ── Repeat Mode Edge Cases ──────────────────────────────────────

#[test]
fn test_repeat_mode_from_invalid_config() {
    assert_eq!(RepeatMode::from_config_str("invalid"), RepeatMode::Off);
    assert_eq!(RepeatMode::from_config_str(""), RepeatMode::Off);
    assert_eq!(RepeatMode::from_config_str("ALL"), RepeatMode::Off); // Case sensitive
    assert_eq!(RepeatMode::from_config_str("One"), RepeatMode::Off); // Case sensitive
}

#[test]
fn test_repeat_mode_cycle_is_deterministic() {
    // Verify the cycle order is always: Off -> All -> One -> Off
    let mut mode = RepeatMode::Off;
    let expected = [RepeatMode::All, RepeatMode::One, RepeatMode::Off];
    for expected_mode in &expected {
        mode = mode.cycle();
        assert_eq!(mode, *expected_mode);
    }
}

// ── Channel Conversion Tests ────────────────────────────────────

#[test]
fn test_convert_channels_same_count_is_identity() {
    let samples = vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6];
    let result = convert_channels(&samples, 2, 2);
    assert_eq!(result, samples);
}

#[test]
fn test_convert_channels_mono_to_stereo() {
    let mono = vec![0.5, -0.3, 0.8];
    let stereo = convert_channels(&mono, 1, 2);
    // Each mono sample should be duplicated to L and R
    assert_eq!(stereo, vec![0.5, 0.5, -0.3, -0.3, 0.8, 0.8]);
}

#[test]
fn test_convert_channels_stereo_to_mono() {
    let stereo = vec![0.4, 0.6, -0.2, 0.8, 1.0, 0.0];
    let mono = convert_channels(&stereo, 2, 1);
    // Each pair should be averaged
    assert_eq!(mono.len(), 3);
    assert!((mono[0] - 0.5).abs() < 1e-6); // (0.4 + 0.6) / 2
    assert!((mono[1] - 0.3).abs() < 1e-6); // (-0.2 + 0.8) / 2
    assert!((mono[2] - 0.5).abs() < 1e-6); // (1.0 + 0.0) / 2
}

#[test]
fn test_convert_channels_mono_to_stereo_preserves_energy() {
    let mono: Vec<f32> = (0..100)
        .map(|i| (2.0 * std::f32::consts::PI * 440.0 * i as f32 / 44100.0).sin())
        .collect();
    let stereo = convert_channels(&mono, 1, 2);

    // Stereo should have exactly 2x the number of samples
    assert_eq!(stereo.len(), mono.len() * 2);

    // Each stereo frame should have identical L and R
    for i in 0..mono.len() {
        assert_eq!(stereo[i * 2], mono[i]);
        assert_eq!(stereo[i * 2 + 1], mono[i]);
    }
}

#[test]
fn test_convert_channels_stereo_to_mono_roundtrip() {
    // mono → stereo → mono should give back the original
    let original = vec![0.1f32, 0.5, -0.3, 0.9, 0.0];
    let stereo = convert_channels(&original, 1, 2);
    let mono_again = convert_channels(&stereo, 2, 1);
    assert_eq!(mono_again.len(), original.len());
    for (a, b) in original.iter().zip(&mono_again) {
        assert!((a - b).abs() < 1e-6, "Roundtrip mismatch: {a} vs {b}");
    }
}

#[test]
fn test_convert_channels_empty_input() {
    let empty: Vec<f32> = vec![];
    assert!(convert_channels(&empty, 1, 2).is_empty());
    assert!(convert_channels(&empty, 2, 1).is_empty());
    assert!(convert_channels(&empty, 2, 2).is_empty());
}

#[test]
fn test_convert_channels_general_case_expand() {
    // 1 channel → 4 channels: duplicate the single channel
    let mono = vec![0.5f32, -0.3];
    let quad = convert_channels(&mono, 1, 4);
    assert_eq!(quad.len(), 8); // 2 frames * 4 channels
    assert_eq!(quad, vec![0.5, 0.5, 0.5, 0.5, -0.3, -0.3, -0.3, -0.3]);
}

#[test]
fn test_convert_channels_general_case_shrink() {
    // 4 channels → 2 channels: take first 2
    let quad = vec![0.1f32, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8];
    let stereo = convert_channels(&quad, 4, 2);
    assert_eq!(stereo.len(), 4); // 2 frames * 2 channels
    assert_eq!(stereo, vec![0.1, 0.2, 0.5, 0.6]);
}

// ── Resampler Tests ─────────────────────────────────────────────

#[test]
fn test_resampler_same_rate_not_needed() {
    // When input and output rates match, no resampler should be used.
    // This is a design test — verify the pipeline doesn't create one.
    // We can't easily test the pipeline directly, but we can verify
    // the Resampler works for a rate change scenario.
    use cli_music_player::audio::resampler::Resampler;

    let mut rs = Resampler::new(44100, 48000, 2, 1024).unwrap();
    // Feed enough data to produce output
    let input: Vec<f32> = (0..2048)
        .map(|i| (2.0 * std::f32::consts::PI * 440.0 * i as f32 / 44100.0).sin() * 0.3)
        .collect();

    // Interleave as stereo
    let mut stereo_input = Vec::with_capacity(input.len() * 2);
    for &s in &input {
        stereo_input.push(s);
        stereo_input.push(s);
    }

    let output = rs.process(&stereo_input).unwrap();
    // Output may be empty if still buffering, or non-empty if enough frames
    // The key test is that it doesn't panic and produces valid samples
    for &s in &output {
        assert!(s.is_finite(), "Resampled output should be finite");
    }
}

#[test]
fn test_resampler_accumulates_correctly() {
    use cli_music_player::audio::resampler::Resampler;

    let mut rs = Resampler::new(44100, 48000, 1, 1024).unwrap();

    // Feed small chunks — resampler should accumulate until it has enough
    let small_chunk: Vec<f32> = (0..256)
        .map(|i| (2.0 * std::f32::consts::PI * 1000.0 * i as f32 / 44100.0).sin() * 0.2)
        .collect();

    let mut total_output = Vec::new();
    for _ in 0..20 {
        let out = rs.process(&small_chunk).unwrap();
        total_output.extend_from_slice(&out);
    }

    // After 20 * 256 = 5120 input frames, we should have produced some output
    assert!(
        !total_output.is_empty(),
        "Resampler should produce output after enough input"
    );

    // Output should be finite
    for &s in &total_output {
        assert!(s.is_finite());
    }
}

#[test]
fn test_resampler_empty_input() {
    use cli_music_player::audio::resampler::Resampler;

    let mut rs = Resampler::new(44100, 48000, 2, 1024).unwrap();
    let output = rs.process(&[]).unwrap();
    assert!(output.is_empty());
}

#[test]
fn test_resampler_output_rate_ratio() {
    use cli_music_player::audio::resampler::Resampler;

    let mut rs = Resampler::new(44100, 48000, 1, 1024).unwrap();

    // Feed exactly enough input for a clean ratio check
    let input: Vec<f32> = vec![0.0; 44100]; // 1 second at 44100 Hz
    let output = rs.process(&input).unwrap();

    // Output should be approximately 48000/44100 * input_frames
    // Allow some tolerance due to internal buffering
    let expected_ratio = 48000.0 / 44100.0;
    if !output.is_empty() {
        let actual_ratio = output.len() as f64 / input.len() as f64;
        // Should be close to the expected ratio (within the resampler's chunk size error)
        assert!(
            (actual_ratio - expected_ratio).abs() < 0.1,
            "Output/input ratio {actual_ratio} should be close to {expected_ratio}"
        );
    }
}

// ── AudioCommand / AudioEvent Enum Tests ────────────────────────

#[test]
fn test_audio_command_play_carries_url() {
    let cmd = AudioCommand::Play {
        url: "https://example.com/song.mp3".to_string(),
    };
    if let AudioCommand::Play { url } = cmd {
        assert_eq!(url, "https://example.com/song.mp3");
    } else {
        panic!("Expected Play command");
    }
}

#[test]
fn test_audio_command_seek_carries_position() {
    let cmd = AudioCommand::Seek(42.5);
    if let AudioCommand::Seek(pos) = cmd {
        assert!((pos - 42.5).abs() < f64::EPSILON);
    } else {
        panic!("Expected Seek command");
    }
}

#[test]
fn test_audio_command_set_volume_range() {
    // Volume 0-100
    for v in [0, 50, 75, 100] {
        let cmd = AudioCommand::SetVolume(v);
        if let AudioCommand::SetVolume(vol) = cmd {
            assert_eq!(vol, v);
        }
    }
}

#[test]
fn test_audio_command_set_eq_gains_18_bands() {
    let gains = vec![3.0; 18];
    let cmd = AudioCommand::SetEqGains(gains.clone());
    if let AudioCommand::SetEqGains(g) = cmd {
        assert_eq!(g.len(), 18);
        assert_eq!(g, gains);
    }
}

#[test]
fn test_audio_command_debug_format() {
    // All variants should implement Debug
    let cmds: Vec<AudioCommand> = vec![
        AudioCommand::Play { url: "test".into() },
        AudioCommand::Pause,
        AudioCommand::Resume,
        AudioCommand::Stop,
        AudioCommand::Seek(0.0),
        AudioCommand::SetVolume(75),
        AudioCommand::SetMuted(false),
        AudioCommand::SetEqGains(vec![0.0; 18]),
        AudioCommand::SetEqEnabled(true),
        AudioCommand::Shutdown,
    ];
    for cmd in &cmds {
        let debug = format!("{:?}", cmd);
        assert!(!debug.is_empty());
    }
}

#[test]
fn test_audio_event_position_update() {
    let event = AudioEvent::PositionUpdate {
        position: 30.5,
        duration: 240.0,
    };
    if let AudioEvent::PositionUpdate { position, duration } = event {
        assert!((position - 30.5).abs() < f64::EPSILON);
        assert!((duration - 240.0).abs() < f64::EPSILON);
    }
}

#[test]
fn test_audio_event_state_change_carries_state() {
    let event = AudioEvent::StateChange(PlaybackState::Playing);
    if let AudioEvent::StateChange(state) = event {
        assert_eq!(state, PlaybackState::Playing);
    }
}

#[test]
fn test_audio_event_clone() {
    let event = AudioEvent::PositionUpdate {
        position: 10.0,
        duration: 100.0,
    };
    let cloned = event.clone();
    if let (
        AudioEvent::PositionUpdate {
            position: p1,
            duration: d1,
        },
        AudioEvent::PositionUpdate {
            position: p2,
            duration: d2,
        },
    ) = (&event, &cloned)
    {
        assert_eq!(p1, p2);
        assert_eq!(d1, d2);
    }
}

#[test]
fn test_audio_event_debug_format() {
    let events: Vec<AudioEvent> = vec![
        AudioEvent::PositionUpdate {
            position: 0.0,
            duration: 0.0,
        },
        AudioEvent::StateChange(PlaybackState::Stopped),
        AudioEvent::TrackEnd,
        AudioEvent::Error("test error".into()),
    ];
    for event in &events {
        let debug = format!("{:?}", event);
        assert!(!debug.is_empty());
    }
}

#[test]
fn test_audio_event_error_carries_message() {
    let event = AudioEvent::Error("Decode failed: invalid header".to_string());
    if let AudioEvent::Error(msg) = event {
        assert!(msg.contains("Decode failed"));
    }
}
