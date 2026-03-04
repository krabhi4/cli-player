use rand::seq::SliceRandom;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::subsonic::Song;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RepeatMode {
    Off,
    All,
    One,
}

impl RepeatMode {
    pub fn cycle(self) -> Self {
        match self {
            Self::Off => Self::All,
            Self::All => Self::One,
            Self::One => Self::Off,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Off => "Off",
            Self::All => "All",
            Self::One => "One",
        }
    }

    pub fn icon(self) -> &'static str {
        match self {
            Self::Off => "↻",
            Self::All => "↻",
            Self::One => "↻1",
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Off => "off",
            Self::All => "all",
            Self::One => "one",
        }
    }

    pub fn from_config_str(s: &str) -> Self {
        match s {
            "all" => Self::All,
            "one" => Self::One,
            _ => Self::Off,
        }
    }
}

pub struct QueueManager {
    original_queue: Vec<Song>,
    queue: Vec<Song>,
    current_index: i32,
    shuffle: bool,
    repeat: RepeatMode,
    history: Vec<i32>,
}

impl QueueManager {
    pub fn new() -> Self {
        Self {
            original_queue: Vec::new(),
            queue: Vec::new(),
            current_index: -1,
            shuffle: false,
            repeat: RepeatMode::Off,
            history: Vec::new(),
        }
    }

    pub fn queue(&self) -> &[Song] {
        &self.queue
    }

    pub fn current_index(&self) -> i32 {
        self.current_index
    }

    pub fn current_song(&self) -> Option<&Song> {
        if self.current_index >= 0 && (self.current_index as usize) < self.queue.len() {
            Some(&self.queue[self.current_index as usize])
        } else {
            None
        }
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    pub fn length(&self) -> usize {
        self.queue.len()
    }

    pub fn shuffle_enabled(&self) -> bool {
        self.shuffle
    }

    pub fn repeat_mode(&self) -> RepeatMode {
        self.repeat
    }

    pub fn has_next(&self) -> bool {
        if self.repeat == RepeatMode::All || self.repeat == RepeatMode::One {
            return !self.queue.is_empty();
        }
        (self.current_index as usize) < self.queue.len().saturating_sub(1)
    }

    pub fn has_prev(&self) -> bool {
        if self.shuffle {
            return !self.history.is_empty();
        }
        self.current_index > 0
    }

    pub fn set_queue(&mut self, songs: Vec<Song>, start_index: usize) {
        self.original_queue = songs.clone();
        self.queue = songs;
        self.current_index = start_index as i32;
        self.history.clear();
        if self.shuffle {
            self.apply_shuffle();
        }
    }

    pub fn add(&mut self, song: Song) {
        self.original_queue.push(song.clone());
        self.queue.push(song);
    }

    pub fn add_songs(&mut self, songs: Vec<Song>) {
        self.original_queue.extend(songs.clone());
        self.queue.extend(songs);
    }

    pub fn add_next(&mut self, song: Song) {
        let insert_pos = (self.current_index + 1) as usize;
        self.original_queue.insert(insert_pos, song.clone());
        self.queue.insert(insert_pos, song);
    }

    pub fn remove(&mut self, index: usize) {
        if index >= self.queue.len() {
            return;
        }
        // Find the song by ID before removing, so we can remove it from original_queue too
        let song_id = self.queue[index].id.clone();
        self.queue.remove(index);
        if let Some(orig_pos) = self.original_queue.iter().position(|s| s.id == song_id) {
            self.original_queue.remove(orig_pos);
        }

        // Clean up history entries that reference removed indices
        self.history.retain(|&h| h != index as i32);
        for h in &mut self.history {
            if *h > index as i32 {
                *h -= 1;
            }
        }

        let idx = index as i32;
        if idx < self.current_index {
            self.current_index -= 1;
        } else if idx == self.current_index {
            if self.queue.is_empty() {
                self.current_index = -1;
            } else if self.current_index >= self.queue.len() as i32 {
                self.current_index = self.queue.len() as i32 - 1;
            }
            // Otherwise current_index stays the same (points to next song)
        }
    }

    pub fn clear(&mut self) {
        self.queue.clear();
        self.original_queue.clear();
        self.current_index = -1;
        self.history.clear();
    }

    pub fn move_item(&mut self, from_idx: usize, to_idx: usize) {
        if from_idx >= self.queue.len() || to_idx >= self.queue.len() {
            return;
        }
        let song = self.queue.remove(from_idx);
        self.queue.insert(to_idx, song);

        let from_i = from_idx as i32;
        let to_i = to_idx as i32;
        let cur = self.current_index;
        if from_i == cur {
            self.current_index = to_i;
        } else if from_i < cur && to_i >= cur {
            self.current_index -= 1;
        } else if to_i <= cur && from_i > cur {
            self.current_index += 1;
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<&Song> {
        if self.is_empty() {
            return None;
        }

        if self.repeat == RepeatMode::One {
            return self.current_song();
        }

        if self.shuffle {
            self.history.push(self.current_index);
            // Cap history to prevent unbounded growth
            if self.history.len() > 200 {
                self.history.drain(..self.history.len() - 200);
            }
            let remaining: Vec<i32> = (0..self.queue.len() as i32)
                .filter(|&i| i != self.current_index)
                .collect();
            if !remaining.is_empty() {
                let mut rng = rand::thread_rng();
                self.current_index = remaining[rng.gen_range(0..remaining.len())];
            } else if self.repeat == RepeatMode::All {
                let mut rng = rand::thread_rng();
                self.current_index = rng.gen_range(0..self.queue.len() as i32);
            } else {
                return None;
            }
        } else if (self.current_index as usize) < self.queue.len() - 1 {
            self.current_index += 1;
        } else if self.repeat == RepeatMode::All {
            self.current_index = 0;
        } else {
            return None;
        }

        self.current_song()
    }

    pub fn previous(&mut self) -> Option<&Song> {
        if self.is_empty() {
            return None;
        }

        if self.shuffle && !self.history.is_empty() {
            self.current_index = self.history.pop().unwrap();
        } else if self.current_index > 0 {
            self.current_index -= 1;
        } else if self.repeat == RepeatMode::All {
            self.current_index = self.queue.len() as i32 - 1;
        } else {
            return None;
        }

        self.current_song()
    }

    pub fn jump_to(&mut self, index: usize) -> Option<&Song> {
        if index < self.queue.len() {
            self.current_index = index as i32;
            if self.shuffle {
                self.history.clear();
            }
            self.current_song()
        } else {
            None
        }
    }

    pub fn toggle_shuffle(&mut self) {
        self.shuffle = !self.shuffle;
        if self.shuffle {
            self.apply_shuffle();
        } else {
            self.restore_order();
        }
    }

    pub fn set_shuffle(&mut self, enabled: bool) {
        if enabled != self.shuffle {
            self.toggle_shuffle();
        }
    }

    pub fn cycle_repeat(&mut self) -> RepeatMode {
        self.repeat = self.repeat.cycle();
        self.repeat
    }

    pub fn set_repeat(&mut self, mode: RepeatMode) {
        self.repeat = mode;
    }

    fn apply_shuffle(&mut self) {
        if self.queue.len() <= 1 {
            return;
        }
        let current = self.current_song().cloned();
        let mut others: Vec<Song> = self
            .queue
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != self.current_index as usize)
            .map(|(_, s)| s.clone())
            .collect();
        let mut rng = rand::thread_rng();
        others.shuffle(&mut rng);
        if let Some(cur) = current {
            self.queue = std::iter::once(cur).chain(others).collect();
        } else {
            self.queue = others;
        }
        self.current_index = 0;
        self.history.clear();
    }

    fn restore_order(&mut self) {
        let current = self.current_song().cloned();
        self.queue = self.original_queue.clone();
        if let Some(cur) = current {
            self.current_index = self
                .queue
                .iter()
                .position(|s| s.id == cur.id)
                .map(|i| i as i32)
                .unwrap_or(0);
        }
        self.history.clear();
    }

    pub fn get_upcoming(&self, count: usize) -> &[Song] {
        if self.is_empty() || self.current_index < 0 {
            return &[];
        }
        let start = (self.current_index + 1) as usize;
        let end = (start + count).min(self.queue.len());
        if start >= self.queue.len() {
            return &[];
        }
        &self.queue[start..end]
    }

    pub fn total_duration(&self) -> u64 {
        self.queue.iter().map(|s| s.duration).sum()
    }

    pub fn history(&self) -> &[i32] {
        &self.history
    }
}

impl Default for QueueManager {
    fn default() -> Self {
        Self::new()
    }
}
