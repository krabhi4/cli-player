use std::f64::consts::PI;

/// 18-band EQ frequencies in Hz.
pub const EQ_FREQUENCIES: [f64; 18] = [
    65.0, 92.0, 131.0, 185.0, 262.0, 370.0, 523.0, 740.0, 1047.0, 1480.0, 2093.0, 2960.0, 4186.0,
    5920.0, 8372.0, 11840.0, 16744.0, 20000.0,
];

/// A second-order biquad filter using Direct Form I.
#[derive(Clone)]
struct BiquadFilter {
    b0: f64,
    b1: f64,
    b2: f64,
    a1: f64,
    a2: f64,
    // State
    x1: f64,
    x2: f64,
    y1: f64,
    y2: f64,
}

impl BiquadFilter {
    /// Create a peaking EQ filter (Robert Bristow-Johnson).
    fn peaking_eq(freq: f64, gain_db: f64, q: f64, sample_rate: f64) -> Self {
        let a = 10.0_f64.powf(gain_db / 40.0);
        let w0 = 2.0 * PI * freq / sample_rate;
        let cos_w0 = w0.cos();
        let sin_w0 = w0.sin();
        let alpha = sin_w0 / (2.0 * q);

        let b0 = 1.0 + alpha * a;
        let b1 = -2.0 * cos_w0;
        let b2 = 1.0 - alpha * a;
        let a0 = 1.0 + alpha / a;
        let a1 = -2.0 * cos_w0;
        let a2 = 1.0 - alpha / a;

        Self {
            b0: b0 / a0,
            b1: b1 / a0,
            b2: b2 / a0,
            a1: a1 / a0,
            a2: a2 / a0,
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
        }
    }

    /// Process a single sample through the filter.
    fn process(&mut self, input: f64) -> f64 {
        let output = self.b0 * input + self.b1 * self.x1 + self.b2 * self.x2
            - self.a1 * self.y1
            - self.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = input;
        self.y2 = self.y1;
        self.y1 = output;
        output
    }

    /// Update coefficients without resetting filter state (smooth transition).
    fn update_coefficients(&mut self, freq: f64, gain_db: f64, q: f64, sample_rate: f64) {
        let a = 10.0_f64.powf(gain_db / 40.0);
        let w0 = 2.0 * PI * freq / sample_rate;
        let cos_w0 = w0.cos();
        let sin_w0 = w0.sin();
        let alpha = sin_w0 / (2.0 * q);

        let b0 = 1.0 + alpha * a;
        let b1 = -2.0 * cos_w0;
        let b2 = 1.0 - alpha * a;
        let a0 = 1.0 + alpha / a;
        let a1 = -2.0 * cos_w0;
        let a2 = 1.0 - alpha / a;

        self.b0 = b0 / a0;
        self.b1 = b1 / a0;
        self.b2 = b2 / a0;
        self.a1 = a1 / a0;
        self.a2 = a2 / a0;
    }

    fn reset(&mut self) {
        self.x1 = 0.0;
        self.x2 = 0.0;
        self.y1 = 0.0;
        self.y2 = 0.0;
    }
}

/// 18-band parametric equalizer using cascaded biquad filters.
pub struct EqualizerDsp {
    /// filters[channel][band]
    filters: Vec<Vec<BiquadFilter>>,
    gains: [f64; 18],
    sample_rate: f64,
    channels: usize,
    enabled: bool,
}

impl EqualizerDsp {
    const Q: f64 = 1.41; // ~1 octave bandwidth

    pub fn new(sample_rate: u32, channels: usize) -> Self {
        let mut dsp = Self {
            filters: Vec::new(),
            gains: [0.0; 18],
            sample_rate: sample_rate as f64,
            channels,
            enabled: true,
        };
        dsp.rebuild_filters();
        dsp
    }

    pub fn set_gains(&mut self, gains: &[f64]) {
        for (i, &g) in gains.iter().take(18).enumerate() {
            self.gains[i] = g.clamp(-12.0, 12.0);
        }
        // Update coefficients in-place to preserve filter state (smooth transition)
        if self.filters.is_empty() {
            self.rebuild_filters();
        } else {
            for ch_filters in &mut self.filters {
                for (i, &freq) in EQ_FREQUENCIES.iter().enumerate() {
                    ch_filters[i].update_coefficients(
                        freq,
                        self.gains[i],
                        Self::Q,
                        self.sample_rate,
                    );
                }
            }
        }
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.reset_state();
        }
    }

    /// Process interleaved f32 samples in-place.
    pub fn process(&mut self, samples: &mut [f32]) {
        if !self.enabled || self.gains.iter().all(|&g| g == 0.0) {
            return;
        }

        for frame_start in (0..samples.len()).step_by(self.channels) {
            for ch in 0..self.channels {
                let idx = frame_start + ch;
                if idx >= samples.len() {
                    break;
                }
                let mut sample = samples[idx] as f64;
                for band in 0..18 {
                    sample = self.filters[ch][band].process(sample);
                }
                samples[idx] = sample as f32;
            }
        }
    }

    fn rebuild_filters(&mut self) {
        self.filters = (0..self.channels)
            .map(|_| {
                EQ_FREQUENCIES
                    .iter()
                    .enumerate()
                    .map(|(i, &freq)| {
                        BiquadFilter::peaking_eq(freq, self.gains[i], Self::Q, self.sample_rate)
                    })
                    .collect()
            })
            .collect();
    }

    fn reset_state(&mut self) {
        for ch_filters in &mut self.filters {
            for filter in ch_filters {
                filter.reset();
            }
        }
    }
}
