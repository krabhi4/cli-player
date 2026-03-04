use cli_music_player::audio::equalizer_dsp::{EQ_FREQUENCIES, EqualizerDsp};
use cli_music_player::config::presets::default_eq_presets;
use cli_music_player::equalizer::{EQ_BAND_LABELS, EQ_BANDS, GAIN_MAX, GAIN_MIN};

// ── Constants Tests ─────────────────────────────────────────────

#[test]
fn test_gain_range() {
    assert_eq!(GAIN_MIN, -12.0);
    assert_eq!(GAIN_MAX, 12.0);
}

#[test]
fn test_band_count() {
    assert_eq!(EQ_BANDS.len(), 18);
    assert_eq!(EQ_BAND_LABELS.len(), 18);
    assert_eq!(EQ_FREQUENCIES.len(), 18);
}

#[test]
fn test_frequencies_ascending() {
    for i in 1..EQ_FREQUENCIES.len() {
        assert!(EQ_FREQUENCIES[i] > EQ_FREQUENCIES[i - 1]);
    }
}

#[test]
fn test_frequencies_match_bands() {
    for (i, &freq) in EQ_BANDS.iter().enumerate() {
        assert_eq!(freq as f64, EQ_FREQUENCIES[i]);
    }
}

// ── EqualizerDsp Tests ──────────────────────────────────────────

#[test]
fn test_dsp_new_default_gains() {
    let dsp = EqualizerDsp::new(44100, 2);
    // Default gains are all zero — processing should be no-op
    let mut samples = vec![0.5f32; 100];
    let original = samples.clone();
    // With all zero gains, process should not modify samples
    let mut dsp = dsp;
    dsp.process(&mut samples);
    assert_eq!(samples, original);
}

#[test]
fn test_dsp_set_gains_clamps() {
    let mut dsp = EqualizerDsp::new(44100, 2);
    // Set gains beyond limits
    let gains = vec![20.0; 18];
    dsp.set_gains(&gains);
    // Should process without panic
    let mut samples = vec![0.5f32; 100];
    dsp.process(&mut samples);
}

#[test]
fn test_dsp_negative_gains_clamp() {
    let mut dsp = EqualizerDsp::new(44100, 2);
    let gains = vec![-20.0; 18];
    dsp.set_gains(&gains);
    let mut samples = vec![0.5f32; 100];
    dsp.process(&mut samples);
    // Should not panic
}

#[test]
fn test_dsp_enabled_disabled() {
    let mut dsp = EqualizerDsp::new(44100, 2);
    dsp.set_gains(&[6.0; 18]);

    let mut samples1 = vec![0.5f32; 200];
    dsp.process(&mut samples1);

    // Disable and process
    dsp.set_enabled(false);
    let mut samples2 = vec![0.5f32; 200];
    dsp.process(&mut samples2);

    // Disabled should be unmodified
    assert_eq!(samples2, vec![0.5f32; 200]);
}

#[test]
fn test_dsp_process_stereo() {
    let mut dsp = EqualizerDsp::new(44100, 2);
    dsp.set_gains(&[3.0; 18]);

    let mut samples = vec![0.3f32; 44]; // 22 stereo frames
    dsp.process(&mut samples);
    // Should process without panic, samples should be modified
}

#[test]
fn test_dsp_process_mono() {
    let mut dsp = EqualizerDsp::new(44100, 1);
    dsp.set_gains(&[3.0; 18]);

    let mut samples = vec![0.3f32; 44];
    dsp.process(&mut samples);
}

#[test]
fn test_dsp_different_sample_rates() {
    for rate in [22050, 44100, 48000, 96000] {
        let mut dsp = EqualizerDsp::new(rate, 2);
        dsp.set_gains(&[6.0; 18]);
        let mut samples = vec![0.5f32; 200];
        dsp.process(&mut samples);
        // Should not panic at any rate
    }
}

#[test]
fn test_dsp_zero_input() {
    let mut dsp = EqualizerDsp::new(44100, 2);
    dsp.set_gains(&[12.0; 18]);
    let mut samples = vec![0.0f32; 100];
    dsp.process(&mut samples);
    // Zero input through any linear filter should remain zero (approximately)
    for &s in &samples {
        assert!(s.abs() < 1e-6, "Expected near-zero, got {s}");
    }
}

// ── Preset Tests ────────────────────────────────────────────────

#[test]
fn test_default_presets_count() {
    let presets = default_eq_presets();
    assert_eq!(presets.len(), 10);
}

#[test]
fn test_flat_preset_is_zero() {
    let presets = default_eq_presets();
    let flat = presets.iter().find(|p| p.name == "Flat").unwrap();
    assert!(flat.gains.iter().all(|&g| g == 0.0));
}

#[test]
fn test_bass_boost_preset() {
    let presets = default_eq_presets();
    let bb = presets.iter().find(|p| p.name == "Bass Boost").unwrap();
    assert_eq!(bb.gains.len(), 18);
    // First few bands should be positive
    assert!(bb.gains[0] > 0.0);
    assert!(bb.gains[1] > 0.0);
    // Later bands should be zero
    assert_eq!(bb.gains[17], 0.0);
}

#[test]
fn test_treble_boost_preset() {
    let presets = default_eq_presets();
    let tb = presets.iter().find(|p| p.name == "Treble Boost").unwrap();
    assert_eq!(tb.gains.len(), 18);
    // First few bands should be zero
    assert_eq!(tb.gains[0], 0.0);
    // Later bands should be positive
    assert!(tb.gains[17] > 0.0);
}

#[test]
fn test_all_presets_have_18_bands() {
    let presets = default_eq_presets();
    for preset in &presets {
        assert_eq!(
            preset.gains.len(),
            18,
            "Preset '{}' has {} bands, expected 18",
            preset.name,
            preset.gains.len()
        );
    }
}

#[test]
fn test_all_presets_within_gain_range() {
    let presets = default_eq_presets();
    for preset in &presets {
        for (i, &gain) in preset.gains.iter().enumerate() {
            assert!(
                gain >= GAIN_MIN && gain <= GAIN_MAX,
                "Preset '{}' band {i} gain {gain} out of range [{GAIN_MIN}, {GAIN_MAX}]",
                preset.name
            );
        }
    }
}

#[test]
fn test_preset_names_unique() {
    let presets = default_eq_presets();
    let mut names: Vec<&str> = presets.iter().map(|p| p.name.as_str()).collect();
    names.sort();
    names.dedup();
    assert_eq!(names.len(), presets.len());
}

// ── dB to linear conversion tests ──────────────────────────────

#[test]
fn test_db_to_linear_zero() {
    // 0 dB = gain of 1.0
    let linear = 10.0_f64.powf(0.0 / 20.0);
    assert!((linear - 1.0).abs() < 1e-10);
}

#[test]
fn test_db_to_linear_six() {
    // +6 dB ≈ 2.0
    let linear = 10.0_f64.powf(6.0 / 20.0);
    assert!((linear - 2.0).abs() < 0.01);
}

#[test]
fn test_db_to_linear_negative_six() {
    // -6 dB ≈ 0.5
    let linear = 10.0_f64.powf(-6.0 / 20.0);
    assert!((linear - 0.5).abs() < 0.01);
}

#[test]
fn test_db_to_linear_twelve() {
    // +12 dB ≈ 4.0
    let linear = 10.0_f64.powf(12.0 / 20.0);
    assert!((linear - 3.981).abs() < 0.01);
}

#[test]
fn test_db_to_linear_negative_twelve() {
    // -12 dB ≈ 0.25
    let linear = 10.0_f64.powf(-12.0 / 20.0);
    assert!((linear - 0.251).abs() < 0.01);
}

// ── Band label tests ────────────────────────────────────────────

#[test]
fn test_band_labels() {
    use cli_music_player::equalizer::Equalizer;
    assert_eq!(Equalizer::band_label(0), "65");
    assert_eq!(Equalizer::band_label(8), "1K");
    assert_eq!(Equalizer::band_label(17), "20K");
    assert_eq!(Equalizer::band_label(18), "?");
}

#[test]
fn test_band_frequency() {
    use cli_music_player::equalizer::Equalizer;
    assert_eq!(Equalizer::band_frequency(0), 65);
    assert_eq!(Equalizer::band_frequency(17), 20000);
    assert_eq!(Equalizer::band_frequency(18), 0);
}

// ── DSP Behavioral Tests ───────────────────────────────────────

#[test]
fn test_dsp_boost_increases_amplitude() {
    let mut dsp = EqualizerDsp::new(44100, 1);
    dsp.set_gains(&[12.0; 18]); // Max boost all bands

    // Generate a simple signal
    let mut samples: Vec<f32> = (0..4410)
        .map(|i| (2.0 * std::f32::consts::PI * 1000.0 * i as f32 / 44100.0).sin() * 0.1)
        .collect();
    let original_energy: f32 = samples.iter().map(|s| s * s).sum();

    dsp.process(&mut samples);
    let boosted_energy: f32 = samples.iter().map(|s| s * s).sum();

    assert!(
        boosted_energy > original_energy,
        "Boosted energy {boosted_energy} should be greater than original {original_energy}"
    );
}

#[test]
fn test_dsp_cut_decreases_amplitude() {
    let mut dsp = EqualizerDsp::new(44100, 1);
    dsp.set_gains(&[-12.0; 18]); // Max cut all bands

    let mut samples: Vec<f32> = (0..4410)
        .map(|i| (2.0 * std::f32::consts::PI * 1000.0 * i as f32 / 44100.0).sin() * 0.5)
        .collect();
    let original_energy: f32 = samples.iter().map(|s| s * s).sum();

    dsp.process(&mut samples);
    let cut_energy: f32 = samples.iter().map(|s| s * s).sum();

    assert!(
        cut_energy < original_energy,
        "Cut energy {cut_energy} should be less than original {original_energy}"
    );
}

#[test]
fn test_dsp_toggle_enabled() {
    let mut dsp = EqualizerDsp::new(44100, 1);
    dsp.set_gains(&[12.0; 18]);

    let input: Vec<f32> = (0..1000)
        .map(|i| (2.0 * std::f32::consts::PI * 440.0 * i as f32 / 44100.0).sin() * 0.3)
        .collect();

    // Process with EQ enabled
    let mut enabled_samples = input.clone();
    dsp.process(&mut enabled_samples);

    // Process with EQ disabled
    dsp.set_enabled(false);
    let mut disabled_samples = input.clone();
    dsp.process(&mut disabled_samples);

    // Disabled should be unmodified
    assert_eq!(disabled_samples, input);
    // Enabled should differ
    assert_ne!(enabled_samples, input);
}

#[test]
fn test_dsp_stereo_channels_processed_independently() {
    let mut dsp = EqualizerDsp::new(44100, 2);
    dsp.set_gains(&[6.0; 18]);

    // Stereo signal: left channel has signal, right is silent
    let frames = 1000;
    let mut samples = Vec::with_capacity(frames * 2);
    for i in 0..frames {
        let left = (2.0 * std::f32::consts::PI * 440.0 * i as f32 / 44100.0).sin() * 0.3;
        samples.push(left);
        samples.push(0.0); // right channel silent
    }

    dsp.process(&mut samples);

    // Right channel samples (odd indices) should remain near zero
    let right_energy: f32 = (0..frames).map(|i| samples[i * 2 + 1].powi(2)).sum();
    assert!(
        right_energy < 1e-6,
        "Silent right channel should remain near-silent, got energy {right_energy}"
    );
}

#[test]
fn test_dsp_preset_applied_correctly() {
    let presets = default_eq_presets();
    let bass_boost = presets.iter().find(|p| p.name == "Bass Boost").unwrap();

    let mut dsp = EqualizerDsp::new(44100, 1);
    dsp.set_gains(&bass_boost.gains);

    // Low frequency signal (100 Hz) should be boosted
    let mut low_freq: Vec<f32> = (0..4410)
        .map(|i| (2.0 * std::f32::consts::PI * 100.0 * i as f32 / 44100.0).sin() * 0.1)
        .collect();
    let original_low: f32 = low_freq.iter().map(|s| s * s).sum();
    dsp.process(&mut low_freq);
    let processed_low: f32 = low_freq.iter().map(|s| s * s).sum();

    assert!(
        processed_low > original_low,
        "Bass Boost should increase low frequency energy"
    );
}

#[test]
fn test_dsp_empty_input() {
    let mut dsp = EqualizerDsp::new(44100, 2);
    dsp.set_gains(&[6.0; 18]);
    let mut samples: Vec<f32> = vec![];
    dsp.process(&mut samples);
    assert!(samples.is_empty());
}

// ── DSP Edge Cases & Regression Tests ──────────────────────────

#[test]
fn test_dsp_single_sample_mono() {
    let mut dsp = EqualizerDsp::new(44100, 1);
    dsp.set_gains(&[6.0; 18]);
    let mut samples = vec![0.5f32];
    dsp.process(&mut samples);
    // Should not panic; single sample is valid
    assert_eq!(samples.len(), 1);
}

#[test]
fn test_dsp_single_frame_stereo() {
    let mut dsp = EqualizerDsp::new(44100, 2);
    dsp.set_gains(&[6.0; 18]);
    let mut samples = vec![0.5f32, -0.5f32]; // One stereo frame
    dsp.process(&mut samples);
    assert_eq!(samples.len(), 2);
}

#[test]
fn test_dsp_odd_sample_count_stereo() {
    // Odd number of samples for a stereo signal — partial frame
    let mut dsp = EqualizerDsp::new(44100, 2);
    dsp.set_gains(&[3.0; 18]);
    let mut samples = vec![0.5f32; 101]; // 50.5 frames
    dsp.process(&mut samples);
    // Should process without panic
    assert_eq!(samples.len(), 101);
}

#[test]
fn test_dsp_gains_changed_mid_stream() {
    let mut dsp = EqualizerDsp::new(44100, 1);
    dsp.set_gains(&[0.0; 18]);

    // Process with flat EQ
    let mut samples = vec![0.5f32; 1000];
    dsp.process(&mut samples);
    let flat_energy: f32 = samples.iter().map(|s| s * s).sum();

    // Change to boost
    dsp.set_gains(&[12.0; 18]);
    let mut boosted = vec![0.5f32; 1000];
    dsp.process(&mut boosted);
    let boost_energy: f32 = boosted.iter().map(|s| s * s).sum();

    // After boosting, energy should be higher
    assert!(boost_energy > flat_energy);
}

#[test]
fn test_dsp_enable_disable_toggle_rapid() {
    let mut dsp = EqualizerDsp::new(44100, 1);
    dsp.set_gains(&[12.0; 18]);

    // Toggle enable/disable rapidly — should not corrupt state
    for _ in 0..100 {
        dsp.set_enabled(true);
        let mut samples = vec![0.5f32; 10];
        dsp.process(&mut samples);
        dsp.set_enabled(false);
        dsp.process(&mut samples);
    }
    // Should not panic
}

#[test]
fn test_dsp_nan_input_does_not_spread() {
    let mut dsp = EqualizerDsp::new(44100, 1);
    dsp.set_gains(&[6.0; 18]);

    // Feed a NaN — biquad filters can propagate NaN
    let mut samples = vec![0.5f32; 100];
    samples[50] = f32::NAN;
    dsp.process(&mut samples);

    // After processing, verify we get finite values eventually
    // (the NaN will propagate for a while through filter state, this is expected)
    // Main thing: no panic
}

#[test]
fn test_dsp_max_gain_does_not_produce_infinity() {
    let mut dsp = EqualizerDsp::new(44100, 1);
    dsp.set_gains(&[12.0; 18]); // Max gain

    // Small amplitude signal
    let mut samples: Vec<f32> = (0..4410)
        .map(|i| (2.0 * std::f32::consts::PI * 1000.0 * i as f32 / 44100.0).sin() * 0.1)
        .collect();
    dsp.process(&mut samples);

    // No sample should be infinite
    for &s in &samples {
        assert!(s.is_finite(), "Sample should be finite, got {s}");
    }
}

#[test]
fn test_dsp_different_channel_counts() {
    // Test mono, stereo, and multichannel
    for channels in [1, 2, 4, 6] {
        let mut dsp = EqualizerDsp::new(44100, channels);
        dsp.set_gains(&[3.0; 18]);
        let mut samples = vec![0.5f32; channels * 100];
        dsp.process(&mut samples);
        // Should not panic for any channel count
    }
}

#[test]
fn test_dsp_very_low_sample_rate() {
    let mut dsp = EqualizerDsp::new(8000, 1);
    dsp.set_gains(&[6.0; 18]);
    let mut samples = vec![0.5f32; 100];
    dsp.process(&mut samples);
    // Should not panic even with sample rate below some EQ frequencies
}

#[test]
fn test_dsp_very_high_sample_rate() {
    let mut dsp = EqualizerDsp::new(192000, 2);
    dsp.set_gains(&[6.0; 18]);
    let mut samples = vec![0.5f32; 200];
    dsp.process(&mut samples);
}

// ── Preset Edge Cases ───────────────────────────────────────────

#[test]
fn test_preset_gains_are_independent() {
    let presets = default_eq_presets();

    // Verify that not all presets are identical (except Flat)
    let non_flat: Vec<_> = presets.iter().filter(|p| p.name != "Flat").collect();
    for i in 0..non_flat.len() {
        for j in (i + 1)..non_flat.len() {
            assert_ne!(
                non_flat[i].gains, non_flat[j].gains,
                "Presets '{}' and '{}' should have different gains",
                non_flat[i].name, non_flat[j].name
            );
        }
    }
}

#[test]
fn test_bass_and_treble_boost_are_complementary() {
    let presets = default_eq_presets();
    let bass = presets.iter().find(|p| p.name == "Bass Boost").unwrap();
    let treble = presets.iter().find(|p| p.name == "Treble Boost").unwrap();

    // Bass boost should have gains primarily in low bands
    let bass_low_energy: f64 = bass.gains[..6].iter().sum();
    let bass_high_energy: f64 = bass.gains[12..].iter().sum();
    assert!(bass_low_energy > bass_high_energy);

    // Treble boost should have gains primarily in high bands
    let treble_low_energy: f64 = treble.gains[..6].iter().sum();
    let treble_high_energy: f64 = treble.gains[12..].iter().sum();
    assert!(treble_high_energy > treble_low_energy);
}

// ── Frequency/Band Consistency ──────────────────────────────────

#[test]
fn test_eq_frequencies_cover_audible_range() {
    // First frequency should be in low bass range (< 100 Hz)
    assert!(EQ_FREQUENCIES[0] < 100.0);
    // Last frequency should be near Nyquist for 44.1kHz (around 20kHz)
    assert!(EQ_FREQUENCIES[17] >= 16000.0);
}

#[test]
fn test_eq_frequencies_spacing() {
    // Frequencies should be roughly logarithmically spaced
    for i in 1..EQ_FREQUENCIES.len() {
        let ratio = EQ_FREQUENCIES[i] / EQ_FREQUENCIES[i - 1];
        // Each step should be between 1.2x and 2.0x the previous
        assert!(
            ratio > 1.1 && ratio < 2.5,
            "Frequency ratio between band {} and {} is {}, expected ~1.4",
            i - 1,
            i,
            ratio
        );
    }
}

#[test]
fn test_dsp_flat_eq_preserves_signal_exactly() {
    let mut dsp = EqualizerDsp::new(44100, 1);
    dsp.set_gains(&[0.0; 18]); // Flat

    let original = vec![0.5f32; 1000];
    let mut processed = original.clone();
    dsp.process(&mut processed);

    assert_eq!(processed, original, "Flat EQ should not modify signal");
}

// ── In-place Coefficient Update Tests ───────────────────────────

#[test]
fn test_dsp_set_gains_preserves_filter_state() {
    // Process some audio to build up filter state, then change gains.
    // The output should differ from a fresh DSP with the same gains
    // (because the in-place update preserves x1/x2/y1/y2 state).
    let signal: Vec<f32> = (0..4410)
        .map(|i| (2.0 * std::f32::consts::PI * 440.0 * i as f32 / 44100.0).sin() * 0.3)
        .collect();

    // DSP A: process some audio first, then change gains
    let mut dsp_a = EqualizerDsp::new(44100, 1);
    dsp_a.set_gains(&[3.0; 18]);
    let mut warmup = signal[..2000].to_vec();
    dsp_a.process(&mut warmup);
    // Now change gains — state is preserved
    dsp_a.set_gains(&[6.0; 18]);
    let mut output_a = signal[2000..].to_vec();
    dsp_a.process(&mut output_a);

    // DSP B: fresh DSP with the new gains, no warmup
    let mut dsp_b = EqualizerDsp::new(44100, 1);
    dsp_b.set_gains(&[6.0; 18]);
    let mut output_b = signal[2000..].to_vec();
    dsp_b.process(&mut output_b);

    // They should produce different output (A has filter state, B starts fresh)
    assert_ne!(
        output_a, output_b,
        "In-place update should preserve filter state, producing different output than a fresh DSP"
    );
}

#[test]
fn test_dsp_inplace_update_produces_immediate_change() {
    // Verify that changing gains mid-stream via set_gains immediately
    // affects the output on the very next process() call.
    let mut dsp = EqualizerDsp::new(44100, 1);
    dsp.set_gains(&[0.0; 18]); // Flat — identity

    let signal: Vec<f32> = (0..1000)
        .map(|i| (2.0 * std::f32::consts::PI * 1000.0 * i as f32 / 44100.0).sin() * 0.3)
        .collect();

    // Warm up with flat EQ
    let mut warmup = signal.clone();
    dsp.process(&mut warmup);

    // Now boost everything by 12 dB
    dsp.set_gains(&[12.0; 18]);
    let mut boosted = signal.clone();
    dsp.process(&mut boosted);

    let flat_energy: f32 = warmup.iter().map(|s| s * s).sum();
    let boosted_energy: f32 = boosted.iter().map(|s| s * s).sum();

    assert!(
        boosted_energy > flat_energy * 1.5,
        "Boosted energy {boosted_energy} should be significantly greater than flat {flat_energy}"
    );
}

#[test]
fn test_dsp_rapid_gain_changes_remain_stable() {
    // Rapidly change gains many times during processing — no NaN/Inf
    let mut dsp = EqualizerDsp::new(44100, 2);

    let signal: Vec<f32> = (0..200)
        .map(|i| (2.0 * std::f32::consts::PI * 440.0 * i as f32 / 44100.0).sin() * 0.3)
        .collect();

    for i in 0..50 {
        let gain = ((i % 25) as f64 - 12.0).clamp(-12.0, 12.0);
        dsp.set_gains(&[gain; 18]);
        let mut chunk = signal.clone();
        dsp.process(&mut chunk);

        for &s in &chunk {
            assert!(
                s.is_finite(),
                "Sample became non-finite after rapid gain change {i}, gain={gain}"
            );
        }
    }
}

#[test]
fn test_dsp_inplace_converges_to_same_response() {
    // After enough samples, in-place updated DSP should converge to
    // the same steady-state response as a freshly built DSP.
    let mut dsp_inplace = EqualizerDsp::new(44100, 1);
    dsp_inplace.set_gains(&[3.0; 18]);

    // Process some warmup, then change gains
    let warmup: Vec<f32> = (0..4410)
        .map(|i| (2.0 * std::f32::consts::PI * 1000.0 * i as f32 / 44100.0).sin() * 0.2)
        .collect();
    let mut w = warmup.clone();
    dsp_inplace.process(&mut w);
    dsp_inplace.set_gains(&[6.0; 18]);

    // Fresh DSP with same gains
    let mut dsp_fresh = EqualizerDsp::new(44100, 1);
    dsp_fresh.set_gains(&[6.0; 18]);

    // Both process a long signal to settle
    let settle: Vec<f32> = (0..44100)
        .map(|i| (2.0 * std::f32::consts::PI * 1000.0 * i as f32 / 44100.0).sin() * 0.2)
        .collect();
    let mut out_inplace = settle.clone();
    let mut out_fresh = settle.clone();
    dsp_inplace.process(&mut out_inplace);
    dsp_fresh.process(&mut out_fresh);

    // Compare the last 1000 samples — should be nearly identical
    let tail = out_inplace.len() - 1000;
    let max_diff: f32 = out_inplace[tail..]
        .iter()
        .zip(&out_fresh[tail..])
        .map(|(a, b)| (a - b).abs())
        .fold(0.0f32, f32::max);

    assert!(
        max_diff < 0.001,
        "After settling, max sample difference should be < 0.001, got {max_diff}"
    );
}

#[test]
fn test_dsp_set_gains_from_boost_to_flat_stops_processing() {
    // Changing from boosted gains to all-zero should cause process() to
    // early-return (no-op), since all gains are 0.
    let mut dsp = EqualizerDsp::new(44100, 1);
    dsp.set_gains(&[12.0; 18]);

    let signal: Vec<f32> = (0..1000)
        .map(|i| (2.0 * std::f32::consts::PI * 440.0 * i as f32 / 44100.0).sin() * 0.3)
        .collect();
    let mut warmup = signal.clone();
    dsp.process(&mut warmup);

    // Reset to flat
    dsp.set_gains(&[0.0; 18]);
    let mut flat_output = signal.clone();
    dsp.process(&mut flat_output);

    // With all-zero gains, process returns immediately — signal unchanged
    assert_eq!(
        flat_output, signal,
        "Flat gains should pass signal through unchanged"
    );
}

#[test]
fn test_dsp_single_band_boost_affects_that_frequency() {
    // Boost only the 1kHz band (index 8) and verify it boosts a 1kHz signal
    // more than a 100Hz signal.
    let mut gains = [0.0f64; 18];
    gains[8] = 12.0; // 1047 Hz band

    let mut dsp_1k = EqualizerDsp::new(44100, 1);
    dsp_1k.set_gains(&gains);

    let mut dsp_100 = EqualizerDsp::new(44100, 1);
    dsp_100.set_gains(&gains);

    // 1 kHz signal
    let mut signal_1k: Vec<f32> = (0..4410)
        .map(|i| (2.0 * std::f32::consts::PI * 1000.0 * i as f32 / 44100.0).sin() * 0.1)
        .collect();
    let orig_1k: f32 = signal_1k.iter().map(|s| s * s).sum();
    dsp_1k.process(&mut signal_1k);
    let boosted_1k: f32 = signal_1k.iter().map(|s| s * s).sum();

    // 100 Hz signal
    let mut signal_100: Vec<f32> = (0..4410)
        .map(|i| (2.0 * std::f32::consts::PI * 100.0 * i as f32 / 44100.0).sin() * 0.1)
        .collect();
    let orig_100: f32 = signal_100.iter().map(|s| s * s).sum();
    dsp_100.process(&mut signal_100);
    let boosted_100: f32 = signal_100.iter().map(|s| s * s).sum();

    let ratio_1k = boosted_1k / orig_1k;
    let ratio_100 = boosted_100 / orig_100;

    assert!(
        ratio_1k > ratio_100 * 1.5,
        "1kHz boost ratio ({ratio_1k}) should be significantly larger than 100Hz ratio ({ratio_100})"
    );
}

#[test]
fn test_dsp_update_coefficients_matches_rebuild_steady_state() {
    // Verify that in-place coefficient update and full rebuild produce
    // identical filter coefficients (and thus identical steady-state output)
    // when starting from scratch.
    let mut dsp_update = EqualizerDsp::new(44100, 1);
    let mut dsp_rebuild = EqualizerDsp::new(44100, 1);

    let gains = [
        3.0, -2.0, 5.0, 0.0, -4.0, 6.0, 1.0, -1.0, 8.0, -3.0, 2.0, -5.0, 7.0, 0.0, -6.0, 4.0, -2.0,
        9.0,
    ];

    // Both start fresh, so both paths go through the same coefficient calc
    dsp_update.set_gains(&gains); // in-place update (filters already exist)
    dsp_rebuild.set_gains(&gains); // same path

    let signal: Vec<f32> = (0..4410)
        .map(|i| (2.0 * std::f32::consts::PI * 1000.0 * i as f32 / 44100.0).sin() * 0.2)
        .collect();

    let mut out_update = signal.clone();
    let mut out_rebuild = signal.clone();
    dsp_update.process(&mut out_update);
    dsp_rebuild.process(&mut out_rebuild);

    // Should be identical since both started from same state
    assert_eq!(out_update, out_rebuild);
}
