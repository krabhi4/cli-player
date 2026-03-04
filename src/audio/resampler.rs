use anyhow::{Context, Result};
use rubato::{FftFixedInOut, Resampler as RubatoResampler};

pub struct Resampler {
    resampler: FftFixedInOut<f32>,
    channels: usize,
    input_frames_needed: usize,
    // Per-channel input buffers to accumulate frames until we have enough
    input_buffer: Vec<Vec<f32>>,
}

impl Resampler {
    pub fn new(from_rate: u32, to_rate: u32, channels: usize, chunk_size: usize) -> Result<Self> {
        let resampler =
            FftFixedInOut::new(from_rate as usize, to_rate as usize, chunk_size, channels)
                .context("Failed to create resampler")?;
        let input_frames_needed = resampler.input_frames_next();
        let input_buffer = vec![Vec::new(); channels];
        Ok(Self {
            resampler,
            channels,
            input_frames_needed,
            input_buffer,
        })
    }

    /// Resample interleaved samples with proper buffering.
    /// Accumulates input until enough frames are available, then processes
    /// complete chunks. Returns interleaved output (may be empty if still buffering).
    pub fn process(&mut self, interleaved_input: &[f32]) -> Result<Vec<f32>> {
        // De-interleave and append to per-channel buffers
        let frames = interleaved_input.len() / self.channels;
        for f in 0..frames {
            for ch in 0..self.channels {
                self.input_buffer[ch].push(interleaved_input[f * self.channels + ch]);
            }
        }

        // Process as many complete chunks as possible
        let mut output = Vec::new();
        while self.input_buffer[0].len() >= self.input_frames_needed {
            let chunk: Vec<Vec<f32>> = self
                .input_buffer
                .iter()
                .map(|ch| ch[..self.input_frames_needed].to_vec())
                .collect();

            // Remove processed frames from buffers
            for ch in &mut self.input_buffer {
                ch.drain(..self.input_frames_needed);
            }

            let output_channels = self
                .resampler
                .process(&chunk, None)
                .context("Resampling failed")?;

            // Re-interleave
            let out_frames = output_channels[0].len();
            for frame in 0..out_frames {
                for channel in &output_channels {
                    output.push(channel[frame]);
                }
            }
        }

        Ok(output)
    }
}
