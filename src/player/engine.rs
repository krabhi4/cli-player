use crate::audio::pipeline::{AudioCommand, AudioEvent, AudioPipeline, PlaybackState};
use crate::subsonic::Song;

pub struct Player {
    pipeline: AudioPipeline,
    pub state: PlaybackState,
    pub current_song: Option<Song>,
    pub position: f64,
    pub duration: f64,
    pub volume: u32,
    pub muted: bool,
}

impl Player {
    pub fn new(device_name: Option<String>) -> Self {
        Self {
            pipeline: AudioPipeline::new(device_name),
            state: PlaybackState::Stopped,
            current_song: None,
            position: 0.0,
            duration: 0.0,
            volume: 75,
            muted: false,
        }
    }

    pub fn play(&mut self, url: &str, song: Song) {
        self.current_song = Some(song);
        self.position = 0.0;
        self.duration = 0.0;
        self.state = PlaybackState::Playing; // Set immediately for responsive UI
        self.pipeline.send(AudioCommand::Play {
            url: url.to_string(),
        });
        self.pipeline.send(AudioCommand::SetVolume(self.volume));
        self.pipeline.send(AudioCommand::SetMuted(self.muted));
    }

    pub fn pause(&mut self) {
        if self.state == PlaybackState::Playing {
            self.state = PlaybackState::Paused; // Set immediately
            self.pipeline.send(AudioCommand::Pause);
        }
    }

    pub fn resume(&mut self) {
        if self.state == PlaybackState::Paused {
            self.state = PlaybackState::Playing; // Set immediately
            self.pipeline.send(AudioCommand::Resume);
        }
    }

    pub fn toggle_pause(&mut self) {
        match self.state {
            PlaybackState::Playing => self.pause(),
            PlaybackState::Paused => self.resume(),
            PlaybackState::Stopped => {}
        }
    }

    pub fn stop(&mut self) {
        self.state = PlaybackState::Stopped; // Set BEFORE pipeline stop (prevents false EOF)
        self.pipeline.send(AudioCommand::Stop);
        self.current_song = None;
        self.position = 0.0;
        self.duration = 0.0;
    }

    pub fn seek(&mut self, offset: f64) {
        let new_pos = (self.position + offset).max(0.0);
        let clamped = if self.duration > 0.0 {
            new_pos.min(self.duration)
        } else {
            new_pos
        };
        self.pipeline.send(AudioCommand::Seek(clamped));
        self.position = clamped;
    }

    pub fn seek_to(&mut self, position: f64) {
        let clamped = position.max(0.0);
        let clamped = if self.duration > 0.0 {
            clamped.min(self.duration)
        } else {
            clamped
        };
        self.pipeline.send(AudioCommand::Seek(clamped));
        self.position = clamped;
    }

    pub fn set_volume(&mut self, vol: u32) {
        self.volume = vol.min(100);
        self.pipeline.send(AudioCommand::SetVolume(self.volume));
    }

    pub fn volume_up(&mut self, step: u32) {
        self.set_volume(self.volume.saturating_add(step).min(100));
    }

    pub fn volume_down(&mut self, step: u32) {
        self.set_volume(self.volume.saturating_sub(step));
    }

    pub fn mute_toggle(&mut self) {
        self.muted = !self.muted;
        self.pipeline.send(AudioCommand::SetMuted(self.muted));
    }

    pub fn set_eq_gains(&mut self, gains: &[f64]) {
        self.pipeline.send(AudioCommand::SetEqGains(gains.to_vec()));
    }

    pub fn set_eq_enabled(&mut self, enabled: bool) {
        self.pipeline.send(AudioCommand::SetEqEnabled(enabled));
    }

    /// Poll audio events. Call this from the TUI tick.
    pub fn poll_events(&mut self) -> Vec<AudioEvent> {
        let events = self.pipeline.poll_events();
        for event in &events {
            match event {
                AudioEvent::PositionUpdate { position, duration } => {
                    self.position = *position;
                    self.duration = *duration;
                }
                AudioEvent::StateChange(new_state) => {
                    self.state = *new_state;
                }
                AudioEvent::TrackEnd => {
                    self.state = PlaybackState::Stopped;
                }
                AudioEvent::Error(_) => {}
            }
        }
        events
    }

    pub fn shutdown(&self) {
        self.pipeline.send(AudioCommand::Shutdown);
    }
}
