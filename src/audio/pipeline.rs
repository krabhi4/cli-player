use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use super::decoder::AudioDecoder;
use super::equalizer_dsp::EqualizerDsp;
use super::output::AudioOutput;
use super::resampler::Resampler;

#[derive(Debug)]
pub enum AudioCommand {
    Play { url: String },
    Pause,
    Resume,
    Stop,
    Seek(f64),
    SetVolume(u32),
    SetMuted(bool),
    SetEqGains(Vec<f64>),
    SetEqEnabled(bool),
    Shutdown,
}

#[derive(Debug, Clone)]
pub enum AudioEvent {
    PositionUpdate { position: f64, duration: f64 },
    StateChange(PlaybackState),
    TrackEnd,
    Error(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaybackState {
    Stopped,
    Playing,
    Paused,
}

pub struct AudioPipeline {
    cmd_tx: mpsc::Sender<AudioCommand>,
    event_rx: mpsc::Receiver<AudioEvent>,
    _thread: thread::JoinHandle<()>,
}

impl AudioPipeline {
    pub fn new(device_name: Option<String>) -> Self {
        let (cmd_tx, cmd_rx) = mpsc::channel::<AudioCommand>();
        let (event_tx, event_rx) = mpsc::channel::<AudioEvent>();

        let thread = thread::spawn(move || {
            audio_thread(cmd_rx, event_tx, device_name);
        });

        Self {
            cmd_tx,
            event_rx,
            _thread: thread,
        }
    }

    pub fn send(&self, cmd: AudioCommand) {
        let _ = self.cmd_tx.send(cmd);
    }

    pub fn poll_events(&self) -> Vec<AudioEvent> {
        let mut events = Vec::new();
        while let Ok(event) = self.event_rx.try_recv() {
            events.push(event);
        }
        events
    }
}

/// Convert interleaved samples between channel counts.
/// Mono → Stereo: duplicate each sample.
/// Stereo → Mono: average L+R.
/// N → M: take first min(N,M) channels, duplicate last if expanding.
pub fn convert_channels(samples: &[f32], from_ch: usize, to_ch: usize) -> Vec<f32> {
    if from_ch == to_ch {
        return samples.to_vec();
    }
    let frames = samples.len() / from_ch;
    let mut out = Vec::with_capacity(frames * to_ch);

    if from_ch == 1 && to_ch == 2 {
        // Mono → Stereo: duplicate
        for &s in samples {
            out.push(s);
            out.push(s);
        }
    } else if from_ch == 2 && to_ch == 1 {
        // Stereo → Mono: average
        for f in 0..frames {
            let l = samples[f * 2];
            let r = samples[f * 2 + 1];
            out.push((l + r) * 0.5);
        }
    } else {
        // General case
        for f in 0..frames {
            for c in 0..to_ch {
                if c < from_ch {
                    out.push(samples[f * from_ch + c]);
                } else {
                    // Duplicate last available channel
                    out.push(samples[f * from_ch + from_ch - 1]);
                }
            }
        }
    }
    out
}

fn audio_thread(
    cmd_rx: mpsc::Receiver<AudioCommand>,
    event_tx: mpsc::Sender<AudioEvent>,
    device_name: Option<String>,
) {
    let mut state = PlaybackState::Stopped;
    let mut volume: u32 = 75;
    let mut muted = false;
    let mut eq_gains: Vec<f64> = vec![0.0; 18];
    let mut eq_enabled = true;

    let mut decoder: Option<AudioDecoder> = None;
    let mut output: Option<AudioOutput> = None;
    let mut eq_dsp: Option<EqualizerDsp> = None;
    let mut resampler: Option<Resampler> = None;
    let mut last_position_update = Instant::now();
    let mut total_frames_decoded: u64 = 0;
    let mut dec_channels: usize = 2;
    let mut out_channels: usize = 2;

    loop {
        // Check for commands (non-blocking)
        while let Ok(cmd) = cmd_rx.try_recv() {
            match cmd {
                AudioCommand::Play { url } => {
                    // Stop current playback
                    decoder = None;
                    output = None;
                    eq_dsp = None;
                    resampler = None;
                    total_frames_decoded = 0;

                    match AudioDecoder::from_url(&url) {
                        Ok(dec) => {
                            let dec_rate = dec.sample_rate();
                            dec_channels = dec.channels();

                            match AudioOutput::new(device_name.as_deref()) {
                                Ok(out) => {
                                    let out_rate = out.sample_rate;
                                    out_channels = out.channels as usize;

                                    let mut dsp = EqualizerDsp::new(dec_rate, dec_channels);
                                    dsp.set_gains(&eq_gains);
                                    dsp.set_enabled(eq_enabled);

                                    // Set up resampler if rates differ
                                    // Resampler operates on output channel count
                                    // (channel conversion happens before resampling)
                                    if dec_rate != out_rate {
                                        match Resampler::new(dec_rate, out_rate, out_channels, 1024)
                                        {
                                            Ok(r) => resampler = Some(r),
                                            Err(e) => {
                                                let _ = event_tx.send(AudioEvent::Error(format!(
                                                    "Resampler init failed: {e}"
                                                )));
                                            }
                                        }
                                    }

                                    // Set initial volume/mute on the new output
                                    out.volume
                                        .store(volume, std::sync::atomic::Ordering::Relaxed);
                                    out.muted.store(muted, std::sync::atomic::Ordering::Relaxed);

                                    eq_dsp = Some(dsp);
                                    output = Some(out);
                                    decoder = Some(dec);
                                    state = PlaybackState::Playing;
                                    let _ = event_tx.send(AudioEvent::StateChange(state));
                                }
                                Err(e) => {
                                    let _ = event_tx.send(AudioEvent::Error(format!(
                                        "Audio output failed: {e}"
                                    )));
                                }
                            }
                        }
                        Err(e) => {
                            let _ = event_tx.send(AudioEvent::Error(format!("Decode failed: {e}")));
                        }
                    }
                }
                AudioCommand::Pause => {
                    if state == PlaybackState::Playing {
                        state = PlaybackState::Paused;
                        // Signal output to fill silence immediately
                        if let Some(out) = &output {
                            out.paused.store(true, std::sync::atomic::Ordering::Relaxed);
                        }
                        let _ = event_tx.send(AudioEvent::StateChange(state));
                    }
                }
                AudioCommand::Resume => {
                    if state == PlaybackState::Paused {
                        state = PlaybackState::Playing;
                        if let Some(out) = &output {
                            out.paused
                                .store(false, std::sync::atomic::Ordering::Relaxed);
                        }
                        let _ = event_tx.send(AudioEvent::StateChange(state));
                    }
                }
                AudioCommand::Stop => {
                    decoder = None;
                    output = None;
                    eq_dsp = None;
                    resampler = None;
                    state = PlaybackState::Stopped;
                    total_frames_decoded = 0;
                    let _ = event_tx.send(AudioEvent::StateChange(state));
                }
                AudioCommand::Seek(pos) => {
                    if let Some(dec) = &mut decoder {
                        let _ = dec.seek(pos);
                        let rate = dec.sample_rate() as u64;
                        total_frames_decoded = (pos * rate as f64) as u64;
                    }
                }
                AudioCommand::SetVolume(v) => {
                    volume = v;
                    if let Some(out) = &output {
                        out.volume.store(v, std::sync::atomic::Ordering::Relaxed);
                    }
                }
                AudioCommand::SetMuted(m) => {
                    muted = m;
                    if let Some(out) = &output {
                        out.muted.store(m, std::sync::atomic::Ordering::Relaxed);
                    }
                }
                AudioCommand::SetEqGains(gains) => {
                    eq_gains = gains;
                    if let Some(dsp) = &mut eq_dsp {
                        dsp.set_gains(&eq_gains);
                    }
                    // Flush output buffer so EQ change is heard immediately
                    if let Some(out) = &output {
                        out.flush.store(true, std::sync::atomic::Ordering::Relaxed);
                    }
                }
                AudioCommand::SetEqEnabled(enabled) => {
                    eq_enabled = enabled;
                    if let Some(dsp) = &mut eq_dsp {
                        dsp.set_enabled(enabled);
                    }
                    // Flush output buffer so EQ toggle is heard immediately
                    if let Some(out) = &output {
                        out.flush.store(true, std::sync::atomic::Ordering::Relaxed);
                    }
                }
                AudioCommand::Shutdown => {
                    return;
                }
            }
        }

        // Decode and send audio if playing
        if state == PlaybackState::Playing {
            if let (Some(dec), Some(out)) = (&mut decoder, &output) {
                match dec.next_packet() {
                    Some(mut samples) => {
                        let num_frames = samples.len() / dec_channels;
                        total_frames_decoded += num_frames as u64;

                        // Apply EQ (operates on decoder channel count)
                        if let Some(dsp) = &mut eq_dsp {
                            dsp.process(&mut samples);
                        }

                        // Volume/mute applied in output callback for instant response

                        // Convert channels if needed (mono→stereo, etc.)
                        let samples = if dec_channels != out_channels {
                            convert_channels(&samples, dec_channels, out_channels)
                        } else {
                            samples
                        };

                        // Resample if needed
                        let final_samples = if let Some(rs) = &mut resampler {
                            match rs.process(&samples) {
                                Ok(resampled) => resampled,
                                Err(e) => {
                                    let _ = event_tx
                                        .send(AudioEvent::Error(format!("Resample error: {e}")));
                                    continue;
                                }
                            }
                        } else {
                            samples
                        };

                        // Send to output (skip empty - resampler may be buffering)
                        if !final_samples.is_empty() {
                            let _ = out.sample_sender.send(final_samples);
                        }

                        // Periodic position update (~10 Hz)
                        if last_position_update.elapsed() >= Duration::from_millis(100) {
                            let position = total_frames_decoded as f64 / dec.sample_rate() as f64;
                            let duration = dec.duration_secs().unwrap_or(0.0);
                            let _ =
                                event_tx.send(AudioEvent::PositionUpdate { position, duration });
                            last_position_update = Instant::now();
                        }
                    }
                    None => {
                        // Track ended
                        let _ = event_tx.send(AudioEvent::TrackEnd);
                        decoder = None;
                        output = None;
                        eq_dsp = None;
                        resampler = None;
                        state = PlaybackState::Stopped;
                        let _ = event_tx.send(AudioEvent::StateChange(state));
                    }
                }
            }
        } else {
            // Sleep briefly when not playing to avoid busy-waiting
            thread::sleep(Duration::from_millis(10));
        }
    }
}
