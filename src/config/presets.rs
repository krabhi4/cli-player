use super::models::EQPreset;

pub fn default_eq_presets() -> Vec<EQPreset> {
    vec![
        EQPreset {
            name: "Flat".to_string(),
            gains: vec![0.0; 18],
        },
        EQPreset {
            name: "Bass Boost".to_string(),
            gains: vec![
                10.0, 8.0, 6.0, 4.0, 2.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0,
                0.0, 0.0,
            ],
        },
        EQPreset {
            name: "Treble Boost".to_string(),
            gains: vec![
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 2.0, 4.0, 6.0, 8.0, 10.0,
                10.0, 10.0,
            ],
        },
        EQPreset {
            name: "Vocal".to_string(),
            gains: vec![
                -2.0, -2.0, -1.0, 0.0, 2.0, 4.0, 5.0, 5.0, 4.0, 3.0, 2.0, 1.0, 0.0, -1.0, -2.0,
                -2.0, -3.0, -3.0,
            ],
        },
        EQPreset {
            name: "Rock".to_string(),
            gains: vec![
                5.0, 4.0, 3.0, 2.0, -1.0, -2.0, -1.0, 1.0, 3.0, 4.0, 5.0, 5.0, 4.0, 3.0, 2.0, 1.0,
                0.0, 0.0,
            ],
        },
        EQPreset {
            name: "Pop".to_string(),
            gains: vec![
                -2.0, -1.0, 0.0, 2.0, 4.0, 5.0, 4.0, 2.0, 0.0, -1.0, -2.0, -1.0, 0.0, 2.0, 3.0,
                4.0, 3.0, 2.0,
            ],
        },
        EQPreset {
            name: "Jazz".to_string(),
            gains: vec![
                3.0, 2.0, 1.0, 2.0, -1.0, -1.0, 0.0, 1.0, 2.0, 3.0, 3.0, 3.0, 2.0, 2.0, 3.0, 3.0,
                4.0, 4.0,
            ],
        },
        EQPreset {
            name: "Classical".to_string(),
            gains: vec![
                4.0, 3.0, 2.0, 1.0, -1.0, -1.0, 0.0, 0.0, 1.0, 2.0, 2.0, 3.0, 3.0, 2.0, 1.0, 2.0,
                3.0, 4.0,
            ],
        },
        EQPreset {
            name: "Electronic".to_string(),
            gains: vec![
                6.0, 5.0, 4.0, 2.0, 0.0, -2.0, -1.0, 0.0, 1.0, 2.0, 0.0, -1.0, 0.0, 2.0, 4.0, 5.0,
                6.0, 5.0,
            ],
        },
        EQPreset {
            name: "Loudness".to_string(),
            gains: vec![
                6.0, 5.0, 3.0, 0.0, -2.0, -3.0, -2.0, 0.0, 0.0, 1.0, 2.0, 4.0, 5.0, 3.0, 0.0, -1.0,
                2.0, 5.0,
            ],
        },
    ]
}

pub fn default_preset_names() -> std::collections::HashSet<String> {
    default_eq_presets()
        .iter()
        .map(|p| p.name.clone())
        .collect()
}
