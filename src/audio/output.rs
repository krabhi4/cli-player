use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{Device, Stream, StreamConfig};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::mpsc;
use std::sync::Arc;

pub struct AudioOutput {
    _stream: Stream,
    pub sample_sender: mpsc::SyncSender<Vec<f32>>,
    pub sample_rate: u32,
    pub channels: u16,
    /// When true, output callback fills silence immediately (instant pause).
    pub paused: Arc<AtomicBool>,
    /// Volume 0-100, applied in the output callback for instant response.
    pub volume: Arc<AtomicU32>,
    /// When true, output is silenced (instant mute).
    pub muted: Arc<AtomicBool>,
    /// When set, output callback discards buffered audio (for instant EQ changes).
    pub flush: Arc<AtomicBool>,
}

impl AudioOutput {
    pub fn new(device_name: Option<&str>) -> Result<Self> {
        let host = cpal::default_host();

        let device = if let Some(name) = device_name.filter(|n| *n != "auto") {
            host.output_devices()
                .context("Failed to enumerate output devices")?
                .find(|d| d.name().map(|n| n == name).unwrap_or(false))
                .context(format!("Audio device '{name}' not found"))?
        } else {
            host.default_output_device()
                .context("No default audio output device")?
        };

        let config = device.default_output_config()?;
        let sample_rate = config.sample_rate().0;
        let channels = config.channels();

        let (tx, rx) = mpsc::sync_channel::<Vec<f32>>(8);
        let paused = Arc::new(AtomicBool::new(false));
        let volume = Arc::new(AtomicU32::new(75));
        let muted = Arc::new(AtomicBool::new(false));
        let flush = Arc::new(AtomicBool::new(false));

        let stream = Self::build_stream(
            &device,
            &config.into(),
            rx,
            Arc::clone(&paused),
            Arc::clone(&volume),
            Arc::clone(&muted),
            Arc::clone(&flush),
        )?;
        stream.play().context("Failed to start audio stream")?;

        Ok(Self {
            _stream: stream,
            sample_sender: tx,
            sample_rate,
            channels,
            paused,
            volume,
            muted,
            flush,
        })
    }

    fn build_stream(
        device: &Device,
        config: &StreamConfig,
        rx: mpsc::Receiver<Vec<f32>>,
        paused: Arc<AtomicBool>,
        volume: Arc<AtomicU32>,
        muted: Arc<AtomicBool>,
        flush: Arc<AtomicBool>,
    ) -> Result<Stream> {
        let mut buffer: Vec<f32> = Vec::new();
        let mut buf_pos: usize = 0;

        let stream = device.build_output_stream(
            config,
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                // When paused, output silence immediately
                if paused.load(Ordering::Relaxed) {
                    data.fill(0.0);
                    return;
                }

                // Flush buffered audio when signaled (instant EQ/DSP changes)
                if flush.load(Ordering::Relaxed) {
                    buffer.clear();
                    buf_pos = 0;
                    // Drain all old data from the channel
                    while rx.try_recv().is_ok() {}
                    flush.store(false, Ordering::Relaxed);
                }

                let mut written = 0;
                while written < data.len() {
                    if buf_pos < buffer.len() {
                        let remaining = buffer.len() - buf_pos;
                        let to_copy = remaining.min(data.len() - written);
                        data[written..written + to_copy]
                            .copy_from_slice(&buffer[buf_pos..buf_pos + to_copy]);
                        buf_pos += to_copy;
                        written += to_copy;
                    } else {
                        match rx.try_recv() {
                            Ok(new_buf) => {
                                buffer = new_buf;
                                buf_pos = 0;
                            }
                            Err(_) => {
                                data[written..].fill(0.0);
                                return;
                            }
                        }
                    }
                }

                // Apply volume/mute at output stage for instant response
                let is_muted = muted.load(Ordering::Relaxed);
                let vol = volume.load(Ordering::Relaxed) as f32 / 100.0;
                if is_muted {
                    data.fill(0.0);
                } else if (vol - 1.0).abs() > f32::EPSILON {
                    for s in data.iter_mut() {
                        *s *= vol;
                    }
                }
            },
            |err| {
                eprintln!("Audio output error: {err}");
            },
            None,
        )?;

        Ok(stream)
    }

    pub fn enumerate_devices() -> Vec<String> {
        let host = cpal::default_host();
        host.output_devices()
            .map(|devices| {
                devices
                    .filter_map(|d| d.name().ok())
                    .collect()
            })
            .unwrap_or_default()
    }
}
