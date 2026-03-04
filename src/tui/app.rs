use std::collections::HashSet;
use std::io;
use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::event::{DisableMouseCapture, EnableMouseCapture};
use crossterm::event::{
    KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::Span;
use ratatui::widgets::Paragraph;

use crate::audio::pipeline::AudioEvent;
use crate::config::AppConfig;
use crate::equalizer::Equalizer;
use crate::player::Player;
use crate::queue::{QueueManager, RepeatMode};
use crate::subsonic::{Album, Artist, Genre, Playlist, Song, SubsonicClient};
use crate::tui::event::{AppEvent, EventHandler};
use crate::tui::theme;
use crate::tui::widgets::{
    browser, equalizer, help, lyrics, now_playing, queue_view, search, server_mgr,
};

const ALBUM_SORT_TYPES: &[&str] = &[
    "newest",
    "alphabeticalByName",
    "alphabeticalByArtist",
    "recent",
    "frequent",
    "random",
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum ModalKind {
    Search,
    Help,
    ServerManager,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Focus {
    Browser,
    Queue,
}

pub struct App {
    pub config: AppConfig,
    player: Player,
    queue_mgr: QueueManager,
    equalizer_state: Equalizer,
    client: Option<SubsonicClient>,
    rt: tokio::runtime::Runtime,

    // UI state
    should_quit: bool,
    active_tab: usize,
    focus: Focus,
    active_modal: Option<ModalKind>,
    eq_visible: bool,
    lyrics_visible: bool,

    // Tab selection indices
    tab_selected: [Option<usize>; 7],
    queue_selected: Option<usize>,

    // Navigation
    nav_history: Vec<(usize, Option<usize>)>,
    album_sort_index: usize,

    // Cached data
    albums: Vec<Album>,
    artists: Vec<Artist>,
    songs: Vec<Song>,
    playlists: Vec<Playlist>,
    genres: Vec<Genre>,
    starred_songs: Vec<Song>,
    play_history: Vec<Song>,

    // Starred IDs for quick lookup
    starred_ids: HashSet<String>,

    // Scrobbling
    scrobble_reported: bool,

    // Search state
    search_query: String,
    search_artists: Vec<Artist>,
    search_albums: Vec<Album>,
    search_songs: Vec<Song>,
    search_tab: usize,
    search_selected: Option<usize>,

    // Lyrics
    lyrics_text: String,
    lyrics_scroll: u16,

    // Server manager state
    server_selected: Option<usize>,
    server_form: [String; 4],
    server_active_field: usize,
    server_status: String,

    // EQ widget state
    eq_selected_band: usize,

    // Cached layout areas for mouse click handling
    layout_tabs_area: Rect,
    layout_browser_area: Rect,
    layout_queue_area: Rect,
    layout_now_playing_area: Rect,

    // Server manager focus mode
    server_focus_list: bool,

    // Double-click tracking
    last_click_time: Instant,
    last_click_row: u16,
    last_click_col: u16,
}

impl App {
    pub fn new(config: AppConfig) -> Self {
        let device = if config.audio_device == "auto" {
            None
        } else {
            Some(config.audio_device.clone())
        };
        let mut player = Player::new(device);
        player.set_volume(config.volume);

        let mut queue_mgr = QueueManager::new();
        if config.shuffle {
            queue_mgr.set_shuffle(true);
        }
        queue_mgr.set_repeat(RepeatMode::from_config_str(&config.repeat_mode));

        let eq_state = Equalizer::new(&config);
        let has_servers = !config.servers.is_empty();

        let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");

        Self {
            config,
            player,
            queue_mgr,
            equalizer_state: eq_state,
            client: None,
            rt,
            should_quit: false,
            active_tab: 0,
            focus: Focus::Browser,
            active_modal: None,
            eq_visible: false,
            lyrics_visible: false,
            tab_selected: [Some(0); 7],
            queue_selected: None,
            nav_history: Vec::new(),
            album_sort_index: 0,
            albums: Vec::new(),
            artists: Vec::new(),
            songs: Vec::new(),
            playlists: Vec::new(),
            genres: Vec::new(),
            starred_songs: Vec::new(),
            play_history: Vec::new(),
            starred_ids: HashSet::new(),
            scrobble_reported: false,
            search_query: String::new(),
            search_artists: Vec::new(),
            search_albums: Vec::new(),
            search_songs: Vec::new(),
            search_tab: 0,
            search_selected: None,
            lyrics_text: String::new(),
            lyrics_scroll: 0,
            server_selected: if has_servers { Some(0) } else { None },
            server_form: Default::default(),
            server_active_field: 0,
            server_status: String::new(),
            eq_selected_band: 0,
            layout_tabs_area: Rect::default(),
            layout_browser_area: Rect::default(),
            layout_queue_area: Rect::default(),
            layout_now_playing_area: Rect::default(),
            server_focus_list: true,
            last_click_time: Instant::now(),
            last_click_row: u16::MAX,
            last_click_col: u16::MAX,
        }
    }

    pub fn select_server_by_name(&mut self, name: &str) {
        if let Some(idx) = self.config.servers.iter().position(|s| s.name == name) {
            self.config.set_active_server(idx);
        }
    }

    pub fn run(&mut self) -> Result<()> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Connect to server
        self.connect_to_active_server();

        // Initial data load
        self.load_library_data();

        let events = EventHandler::new(Duration::from_millis(100));

        while !self.should_quit {
            terminal.draw(|f| self.draw(f))?;

            // Poll audio events
            let audio_events = self.player.poll_events();
            for event in audio_events {
                self.handle_audio_event(event);
            }

            match events.next()? {
                AppEvent::Key(key) if key.kind == KeyEventKind::Press => self.handle_key(key),
                AppEvent::Key(_) => {} // Ignore Release/Repeat events
                AppEvent::Mouse(mouse) => self.handle_mouse(mouse),
                AppEvent::Tick => {}
                AppEvent::Resize(_, _) => {}
            }
        }

        // Cleanup
        self.player.shutdown();
        self.config.volume = self.player.volume;
        self.config.shuffle = self.queue_mgr.shuffle_enabled();
        self.config.repeat_mode = self.queue_mgr.repeat_mode().as_str().to_string();
        self.config.save();

        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        Ok(())
    }

    fn connect_to_active_server(&mut self) {
        if let Some(server) = self.config.active_server() {
            let password = self.config.get_password(Some(server));
            if !password.is_empty() {
                self.client = Some(SubsonicClient::new(
                    &server.url,
                    &server.username,
                    &password,
                ));
            } else {
                self.client = None;
            }
        } else {
            self.client = None;
        }
    }

    fn load_library_data(&mut self) {
        if let Some(client) = &self.client {
            let sort_type = ALBUM_SORT_TYPES[self.album_sort_index];
            if let Ok(albums) = self.rt.block_on(client.get_album_list(sort_type, 50, 0)) {
                self.albums = albums;
            }
        }
    }

    fn draw(&mut self, f: &mut ratatui::Frame) {
        let area = f.area();

        let main_layout = Layout::vertical([
            Constraint::Length(1), // Header/tabs
            Constraint::Min(1),    // Content
            Constraint::Length(4), // Now playing
            Constraint::Length(1), // Status bar
        ])
        .split(area);

        // Header: tabs
        self.layout_tabs_area = main_layout[0];
        browser::render_tabs(f, main_layout[0], self.active_tab);

        // Content area: browser + optional queue sidebar
        let content_layout =
            Layout::horizontal([Constraint::Min(1), Constraint::Length(35)]).split(main_layout[1]);
        self.layout_queue_area = content_layout[1];

        // Browser content based on active tab
        let browser_area = if self.lyrics_visible {
            let lr = Layout::horizontal([Constraint::Percentage(60), Constraint::Percentage(40)])
                .split(content_layout[0]);

            // Lyrics panel
            let (title, artist) = self
                .player
                .current_song
                .as_ref()
                .map(|s| (s.title.as_str(), s.artist.as_str()))
                .unwrap_or(("", ""));
            lyrics::render(
                f,
                lr[1],
                title,
                artist,
                &self.lyrics_text,
                self.lyrics_scroll,
            );
            lr[0]
        } else {
            content_layout[0]
        };

        self.layout_browser_area = browser_area;
        self.render_browser_content(f, browser_area);

        // Queue sidebar
        queue_view::render(
            f,
            content_layout[1],
            self.queue_mgr.queue(),
            self.queue_mgr.current_index(),
            self.queue_selected,
        );

        // Now playing bar
        let server_name = self
            .config
            .active_server()
            .map(|s| s.name.as_str())
            .unwrap_or("No server");
        self.layout_now_playing_area = main_layout[2];
        now_playing::render(
            f,
            main_layout[2],
            &now_playing::NowPlayingState {
                current_song: self.player.current_song.as_ref(),
                state: self.player.state,
                position: self.player.position,
                duration: self.player.duration,
                volume: self.player.volume,
                muted: self.player.muted,
                shuffle: self.queue_mgr.shuffle_enabled(),
                repeat: self.queue_mgr.repeat_mode(),
                eq_enabled: self.equalizer_state.enabled,
                server_name,
            },
        );

        // Status bar
        let status = Paragraph::new(Span::styled(
            " cli-music-player v3.0.0 │ ? for help",
            Style::default().fg(theme::TEXT_MUTED),
        ))
        .style(Style::default().bg(theme::SURFACE_DARK));
        f.render_widget(status, main_layout[3]);

        // EQ overlay
        if self.eq_visible {
            let eq_area = centered_rect(80, 50, area);
            equalizer::render(
                f,
                eq_area,
                &self.equalizer_state.gains,
                self.equalizer_state.enabled,
                self.equalizer_state.current_preset_name(),
                self.eq_selected_band,
            );
        }

        // Modal overlays
        if let Some(modal) = self.active_modal {
            let modal_area = centered_rect(70, 80, area);
            match modal {
                ModalKind::Search => {
                    search::render(
                        f,
                        modal_area,
                        &search::SearchState {
                            query: &self.search_query,
                            artists: &self.search_artists,
                            albums: &self.search_albums,
                            songs: &self.search_songs,
                            active_tab: self.search_tab,
                            selected: self.search_selected,
                        },
                    );
                }
                ModalKind::Help => {
                    help::render(f, modal_area);
                }
                ModalKind::ServerManager => {
                    server_mgr::render(
                        f,
                        modal_area,
                        &server_mgr::ServerMgrState {
                            servers: &self.config.servers,
                            active_index: self.config.active_server_index,
                            selected: self.server_selected,
                            form_fields: &self.server_form,
                            active_field: self.server_active_field,
                            status_msg: &self.server_status,
                            focus_list: self.server_focus_list,
                        },
                    );
                }
            }
        }
    }

    fn render_browser_content(&mut self, f: &mut ratatui::Frame, area: Rect) {
        match self.active_tab {
            0 => browser::render_albums_table(f, area, &self.albums, self.tab_selected[0]),
            1 => browser::render_artists_table(f, area, &self.artists, self.tab_selected[1]),
            2 => browser::render_songs_table(f, area, &self.songs, self.tab_selected[2]),
            3 => browser::render_playlists_table(f, area, &self.playlists, self.tab_selected[3]),
            4 => browser::render_genres_table(f, area, &self.genres, self.tab_selected[4]),
            5 => browser::render_songs_table(f, area, &self.starred_songs, self.tab_selected[5]),
            6 => browser::render_songs_table(f, area, &self.play_history, self.tab_selected[6]),
            _ => {}
        }
    }

    fn handle_audio_event(&mut self, event: AudioEvent) {
        match event {
            AudioEvent::TrackEnd => {
                self.scrobble_reported = false;
                if let Some(song) = self.player.current_song.clone() {
                    self.play_history.push(song);
                    if self.play_history.len() > 100 {
                        self.play_history.remove(0);
                    }
                }
                // Auto-advance
                if let Some(song) = self.queue_mgr.next().cloned() {
                    self.play_song(&song);
                }
            }
            AudioEvent::PositionUpdate { position, duration } => {
                // Scrobble check
                if !self.scrobble_reported && duration > 0.0 {
                    let at_50_percent = position >= duration * 0.5;
                    let at_240s = position >= 240.0;
                    if at_50_percent || at_240s {
                        self.scrobble_reported = true;
                        if let (Some(client), Some(song)) =
                            (&self.client, &self.player.current_song)
                        {
                            let song_id = song.id.clone();
                            // Fire-and-forget: spawn async task to avoid blocking UI
                            let url = client.base_url().to_string();
                            let username = client.username().to_string();
                            let password = client.password().to_string();
                            self.rt.spawn(async move {
                                let c = SubsonicClient::new(&url, &username, &password);
                                let _ = c.scrobble(&song_id, true).await;
                            });
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn handle_key(&mut self, key: KeyEvent) {
        // Modal-specific key handling
        if let Some(modal) = self.active_modal {
            match modal {
                ModalKind::Search => {
                    self.handle_search_key(key);
                    return;
                }
                ModalKind::Help => {
                    if matches!(
                        key.code,
                        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('?') | KeyCode::Char('i')
                    ) {
                        self.active_modal = None;
                    }
                    return;
                }
                ModalKind::ServerManager => {
                    self.handle_server_mgr_key(key);
                    return;
                }
            }
        }

        // EQ-specific keys when visible
        if self.eq_visible {
            match key.code {
                KeyCode::Esc | KeyCode::Char('e') => {
                    self.eq_visible = false;
                    return;
                }
                KeyCode::Left => {
                    self.eq_selected_band = self.eq_selected_band.saturating_sub(1);
                    return;
                }
                KeyCode::Right => {
                    self.eq_selected_band = (self.eq_selected_band + 1).min(17);
                    return;
                }
                KeyCode::Up => {
                    let gain = self.equalizer_state.gains[self.eq_selected_band] + 1.0;
                    self.equalizer_state
                        .set_band(self.eq_selected_band, gain, &mut self.player);
                    return;
                }
                KeyCode::Down => {
                    let gain = self.equalizer_state.gains[self.eq_selected_band] - 1.0;
                    self.equalizer_state
                        .set_band(self.eq_selected_band, gain, &mut self.player);
                    return;
                }
                KeyCode::Char('r') => {
                    self.equalizer_state.reset(&mut self.player);
                    return;
                }
                KeyCode::Char('p') | KeyCode::Char('P') => {
                    // Cycle through presets
                    let presets = self.equalizer_state.get_presets(&self.config);
                    if !presets.is_empty() {
                        let current = self.equalizer_state.current_preset_name().to_string();
                        let idx = presets.iter().position(|p| p.name == current).unwrap_or(0);
                        let next_idx = if key.code == KeyCode::Char('P') {
                            // Shift+P = previous preset
                            if idx == 0 { presets.len() - 1 } else { idx - 1 }
                        } else {
                            (idx + 1) % presets.len()
                        };
                        let next_name = presets[next_idx].name.clone();
                        self.equalizer_state.load_preset(
                            &next_name,
                            &mut self.config,
                            &mut self.player,
                        );
                    }
                    return;
                }
                _ => {}
            }
        }

        // Global key bindings
        match key.code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char(' ') => {
                use crate::audio::pipeline::PlaybackState;
                if self.player.state == PlaybackState::Stopped {
                    // When stopped, play current queue song (matches Python behavior)
                    if let Some(song) = self.queue_mgr.current_song().cloned() {
                        self.play_song(&song);
                    }
                } else {
                    self.player.toggle_pause();
                }
            }
            KeyCode::Char('n') => self.next_track(),
            KeyCode::Char('p') => self.prev_track(),
            KeyCode::Char('s') => self.player.stop(),

            // Seek
            KeyCode::Right if key.modifiers.contains(KeyModifiers::SHIFT) => self.player.seek(30.0),
            KeyCode::Left if key.modifiers.contains(KeyModifiers::SHIFT) => self.player.seek(-30.0),
            KeyCode::Right => self.player.seek(5.0),
            KeyCode::Left => self.player.seek(-5.0),

            // Volume
            KeyCode::Char('+') | KeyCode::Char('=') => self.player.volume_up(5),
            KeyCode::Char('-') | KeyCode::Char('_') => self.player.volume_down(5),
            KeyCode::Char('m') => self.player.mute_toggle(),

            // Queue
            KeyCode::Char('z') => {
                self.queue_mgr.toggle_shuffle();
            }
            KeyCode::Char('r') => {
                self.queue_mgr.cycle_repeat();
            }
            KeyCode::Char('a') => self.add_selected_to_queue(),
            KeyCode::Char('d') | KeyCode::Delete => self.remove_from_queue(),
            KeyCode::Char('c') => self.queue_mgr.clear(),

            // Reorder queue
            KeyCode::Up if key.modifiers.contains(KeyModifiers::SHIFT) => {
                self.move_queue_item_up();
            }
            KeyCode::Down if key.modifiers.contains(KeyModifiers::SHIFT) => {
                self.move_queue_item_down();
            }

            // Navigation
            KeyCode::Up => self.move_selection(-1),
            KeyCode::Down => self.move_selection(1),
            KeyCode::Enter => self.select_current(),
            KeyCode::Esc | KeyCode::Backspace => self.go_back(),
            KeyCode::Tab => self.cycle_focus(),

            // Tab switching
            KeyCode::Char('1') => self.switch_tab(0),
            KeyCode::Char('2') => self.switch_tab(1),
            KeyCode::Char('3') => self.switch_tab(2),
            KeyCode::Char('4') => self.switch_tab(3),
            KeyCode::Char('5') => self.switch_tab(4),
            KeyCode::Char('6') => self.switch_tab(5),
            KeyCode::Char('7') => self.switch_tab(6),

            // Features
            KeyCode::Char('/') => {
                self.active_modal = Some(ModalKind::Search);
                self.search_query.clear();
            }
            KeyCode::Char('e') => {
                self.eq_visible = !self.eq_visible;
            }
            KeyCode::Char('l') => {
                self.lyrics_visible = !self.lyrics_visible;
                if self.lyrics_visible {
                    self.load_lyrics();
                }
            }
            KeyCode::Char('f') => self.toggle_star(),
            KeyCode::Char('R') => self.start_radio(),
            KeyCode::Char('o') => self.cycle_album_sort(),
            KeyCode::Char('S') => {
                self.active_modal = Some(ModalKind::ServerManager);
            }
            KeyCode::Char('?') | KeyCode::Char('i') => {
                self.active_modal = Some(ModalKind::Help);
            }

            _ => {}
        }
    }

    fn handle_mouse(&mut self, mouse: MouseEvent) {
        // Only handle left clicks
        if !matches!(mouse.kind, MouseEventKind::Down(MouseButton::Left)) {
            return;
        }

        // Ignore clicks when modals are open
        if self.active_modal.is_some() || self.eq_visible {
            return;
        }

        let col = mouse.column;
        let row = mouse.row;

        // Detect double-click (same position within 400ms)
        let now = Instant::now();
        let is_double_click = now.duration_since(self.last_click_time) < Duration::from_millis(400)
            && self.last_click_row == row
            && (col as i16 - self.last_click_col as i16).unsigned_abs() <= 2;
        self.last_click_time = now;
        self.last_click_row = row;
        self.last_click_col = col;

        // Click on tab bar
        if row >= self.layout_tabs_area.y
            && row < self.layout_tabs_area.y + self.layout_tabs_area.height
        {
            // Each tab renders as: [1 pad] title [1 pad] [divider " │ " = 3 chars]
            let tab_titles = browser::TAB_TITLES;
            let mut x = self.layout_tabs_area.x;
            for (i, title) in tab_titles.iter().enumerate() {
                let padding: u16 = 2;
                let divider: u16 = if i < tab_titles.len() - 1 { 3 } else { 0 };
                let tab_width = title.len() as u16 + padding + divider;
                if col >= x && col < x + tab_width {
                    self.switch_tab(i);
                    return;
                }
                x += tab_width;
            }
            return;
        }

        // Click in browser area
        if col >= self.layout_browser_area.x
            && col < self.layout_browser_area.x + self.layout_browser_area.width
            && row >= self.layout_browser_area.y
            && row < self.layout_browser_area.y + self.layout_browser_area.height
        {
            self.focus = Focus::Browser;
            let content_start = self.layout_browser_area.y + 2;
            if row >= content_start {
                let clicked_idx = (row - content_start) as usize;
                if clicked_idx < self.current_tab_len() {
                    self.tab_selected[self.active_tab] = Some(clicked_idx);
                    // Double-click triggers select (play/drill-down)
                    if is_double_click {
                        self.select_current();
                    }
                }
            }
            return;
        }

        // Click in queue area
        if col >= self.layout_queue_area.x
            && col < self.layout_queue_area.x + self.layout_queue_area.width
            && row >= self.layout_queue_area.y
            && row < self.layout_queue_area.y + self.layout_queue_area.height
        {
            self.focus = Focus::Queue;
            let content_start = self.layout_queue_area.y + 4;
            if row >= content_start {
                let clicked_idx = (row - content_start) as usize;
                if clicked_idx < self.queue_mgr.length() {
                    self.queue_selected = Some(clicked_idx);
                    // Double-click jumps to queue item and plays
                    if is_double_click {
                        self.select_current();
                    }
                }
            }
            return;
        }

        // Click in now-playing area
        if row >= self.layout_now_playing_area.y
            && row < self.layout_now_playing_area.y + self.layout_now_playing_area.height
        {
            let np = self.layout_now_playing_area;
            // Inner area starts 1 row below (TOP border)
            let inner_y = np.y + 1;
            // Row offsets within inner: 0=title, 1=seekbar, 2=controls, 3=info
            let inner_row = row.saturating_sub(inner_y);

            if inner_row == 1 && self.player.duration > 0.0 {
                // Seekbar click: seek proportionally
                // Layout: " M:SS " + bar + " M:SS "
                // Time labels take ~8 chars each side
                let bar_start = np.x + 8;
                let bar_end = np.x + np.width.saturating_sub(8);
                if col >= bar_start && col <= bar_end && bar_end > bar_start {
                    let proportion = (col - bar_start) as f64 / (bar_end - bar_start) as f64;
                    let seek_pos = proportion * self.player.duration;
                    self.player.seek_to(seek_pos);
                }
            } else if inner_row == 2 {
                // Merged row: controls (left) + repeat/server (right)
                // Left:  " ◁◁ ▶ ▷▷ │  ⇌ EQ │ ♪ xx%"
                // Right: "↻ Off │ ServerName "
                let rel = col.saturating_sub(np.x) as usize;
                let row_width = np.width as usize;

                let vol_str = if self.player.muted {
                    "♪ MUTED".to_string()
                } else {
                    format!("♪ {}%", self.player.volume)
                };
                let vol_end = 17 + 2 + vol_str.len();

                // Right side: compute positions from the right edge
                let repeat_icon = self.queue_mgr.repeat_mode().icon();
                let repeat_label = self.queue_mgr.repeat_mode().label();
                let server_name = self
                    .config
                    .active_server()
                    .map(|s| s.name.as_str())
                    .unwrap_or("No server");
                // Right text: "{icon} {label} │ {server} "
                let repeat_part_len = repeat_icon.len() + 1 + repeat_label.len();
                let server_part_len = 3 + server_name.len() + 1; // " │ {name} "
                let right_total = repeat_part_len + server_part_len;
                let right_start = row_width.saturating_sub(right_total);

                if rel <= 3 {
                    self.prev_track();
                } else if rel <= 5 {
                    use crate::audio::pipeline::PlaybackState;
                    if self.player.state == PlaybackState::Stopped {
                        if let Some(song) = self.queue_mgr.current_song().cloned() {
                            self.play_song(&song);
                        }
                    } else {
                        self.player.toggle_pause();
                    }
                } else if rel <= 8 {
                    self.next_track();
                } else if rel <= 13 {
                    self.queue_mgr.toggle_shuffle();
                } else if rel <= 16 {
                    self.eq_visible = !self.eq_visible;
                } else if rel < vol_end {
                    self.player.mute_toggle();
                } else if rel >= right_start && rel < right_start + repeat_part_len {
                    self.queue_mgr.cycle_repeat();
                } else if rel >= right_start + repeat_part_len + 3
                    && rel < right_start + right_total
                {
                    self.active_modal = Some(ModalKind::ServerManager);
                }
            }
        }
    }

    fn play_song(&mut self, song: &Song) {
        if let Some(client) = &self.client {
            let url = client.stream_url(&song.id);
            self.scrobble_reported = false;
            self.player.play(&url, song.clone());
            self.equalizer_state.apply(&mut self.player);

            // Report now playing (fire-and-forget to avoid blocking UI)
            let song_id = song.id.clone();
            let base_url = client.base_url().to_string();
            let username = client.username().to_string();
            let password = client.password().to_string();
            self.rt.spawn(async move {
                let c = SubsonicClient::new(&base_url, &username, &password);
                let _ = c.now_playing(&song_id).await;
            });
        }
    }

    fn next_track(&mut self) {
        if let Some(song) = self.queue_mgr.next().cloned() {
            self.play_song(&song);
        }
    }

    fn prev_track(&mut self) {
        // If >3 seconds into the song, restart it (standard music player behavior)
        if self.player.position > 3.0 && self.player.current_song.is_some() {
            self.player.seek_to(0.0);
            return;
        }
        if let Some(song) = self.queue_mgr.previous().cloned() {
            self.play_song(&song);
        }
    }

    fn add_selected_to_queue(&mut self) {
        let songs = match self.active_tab {
            2 => self.songs.clone(),
            5 => self.starred_songs.clone(),
            6 => self.play_history.clone(),
            _ => return,
        };
        if let Some(idx) = self.tab_selected[self.active_tab]
            && idx < songs.len()
        {
            self.queue_mgr.add(songs[idx].clone());
        }
    }

    fn remove_from_queue(&mut self) {
        if self.focus == Focus::Queue
            && let Some(idx) = self.queue_selected
        {
            self.queue_mgr.remove(idx);
            let len = self.queue_mgr.length();
            if len == 0 {
                self.queue_selected = None;
            } else if idx >= len {
                self.queue_selected = Some(len - 1);
            }
        }
    }

    fn move_queue_item_up(&mut self) {
        if self.focus == Focus::Queue
            && let Some(idx) = self.queue_selected
            && idx > 0
        {
            self.queue_mgr.move_item(idx, idx - 1);
            self.queue_selected = Some(idx - 1);
        }
    }

    fn move_queue_item_down(&mut self) {
        if self.focus == Focus::Queue
            && let Some(idx) = self.queue_selected
            && idx + 1 < self.queue_mgr.length()
        {
            self.queue_mgr.move_item(idx, idx + 1);
            self.queue_selected = Some(idx + 1);
        }
    }

    fn move_selection(&mut self, delta: i32) {
        match self.focus {
            Focus::Browser => {
                let len = self.current_tab_len();
                if len == 0 {
                    return;
                }
                let current = self.tab_selected[self.active_tab].unwrap_or(0) as i32;
                let new_idx = (current + delta).clamp(0, len as i32 - 1) as usize;
                self.tab_selected[self.active_tab] = Some(new_idx);
            }
            Focus::Queue => {
                let len = self.queue_mgr.length();
                if len == 0 {
                    return;
                }
                let current = self.queue_selected.unwrap_or(0) as i32;
                let new_idx = (current + delta).clamp(0, len as i32 - 1) as usize;
                self.queue_selected = Some(new_idx);
            }
        }
    }

    fn current_tab_len(&self) -> usize {
        match self.active_tab {
            0 => self.albums.len(),
            1 => self.artists.len(),
            2 => self.songs.len(),
            3 => self.playlists.len(),
            4 => self.genres.len(),
            5 => self.starred_songs.len(),
            6 => self.play_history.len(),
            _ => 0,
        }
    }

    fn select_current(&mut self) {
        match self.focus {
            Focus::Queue => {
                if let Some(idx) = self.queue_selected
                    && let Some(song) = self.queue_mgr.jump_to(idx).cloned()
                {
                    self.play_song(&song);
                }
            }
            Focus::Browser => {
                let idx = match self.tab_selected[self.active_tab] {
                    Some(i) => i,
                    None => return,
                };
                match self.active_tab {
                    0 => {
                        if idx < self.albums.len() {
                            let album_id = self.albums[idx].id.clone();
                            if let Some(client) = &self.client
                                && let Ok((_, songs)) =
                                    self.rt.block_on(client.get_album(&album_id))
                            {
                                self.push_nav();
                                self.songs = songs;
                                self.active_tab = 2;
                                self.tab_selected[2] = Some(0);
                            }
                        }
                    }
                    1 => {
                        if idx < self.artists.len() {
                            let artist_id = self.artists[idx].id.clone();
                            if let Some(client) = &self.client
                                && let Ok((_, albums)) =
                                    self.rt.block_on(client.get_artist(&artist_id))
                            {
                                self.push_nav();
                                self.albums = albums;
                                self.active_tab = 0;
                                self.tab_selected[0] = Some(0);
                            }
                        }
                    }
                    2 | 5 | 6 => {
                        let songs = match self.active_tab {
                            2 => &self.songs,
                            5 => &self.starred_songs,
                            6 => &self.play_history,
                            _ => return,
                        };
                        if idx < songs.len() {
                            let songs_clone = songs.to_vec();
                            self.queue_mgr.set_queue(songs_clone, idx);
                            if let Some(song) = self.queue_mgr.current_song().cloned() {
                                self.play_song(&song);
                            }
                        }
                    }
                    3 => {
                        if idx < self.playlists.len() {
                            let pl_id = self.playlists[idx].id.clone();
                            if let Some(client) = &self.client
                                && let Ok((_, songs)) =
                                    self.rt.block_on(client.get_playlist(&pl_id))
                            {
                                self.push_nav();
                                self.songs = songs;
                                self.active_tab = 2;
                                self.tab_selected[2] = Some(0);
                            }
                        }
                    }
                    4 => {
                        if idx < self.genres.len() {
                            let genre = self.genres[idx].name.clone();
                            if let Some(client) = &self.client
                                && let Ok(songs) =
                                    self.rt.block_on(client.get_songs_by_genre(&genre, 50, 0))
                            {
                                self.push_nav();
                                self.songs = songs;
                                self.active_tab = 2;
                                self.tab_selected[2] = Some(0);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    fn push_nav(&mut self) {
        self.nav_history
            .push((self.active_tab, self.tab_selected[self.active_tab]));
        if self.nav_history.len() > 50 {
            self.nav_history.remove(0);
        }
    }

    fn go_back(&mut self) {
        if let Some((tab, sel)) = self.nav_history.pop() {
            self.active_tab = tab;
            self.tab_selected[tab] = sel;
            self.reload_tab(tab);
        }
    }

    fn switch_tab(&mut self, tab: usize) {
        if tab == self.active_tab {
            return;
        }
        self.active_tab = tab;
        self.reload_tab(tab);
    }

    fn reload_tab(&mut self, tab: usize) {
        match tab {
            0 => self.load_library_data(),
            1 => self.load_artists(),
            2 => self.load_songs(),
            3 => self.load_playlists(),
            4 => self.load_genres(),
            5 => self.load_starred(),
            _ => {}
        }
    }

    fn load_songs(&mut self) {
        if let Some(client) = &self.client
            && let Ok(songs) = self.rt.block_on(client.get_random_songs(50, ""))
        {
            self.songs = songs;
        }
    }

    fn cycle_focus(&mut self) {
        self.focus = match self.focus {
            Focus::Browser => Focus::Queue,
            Focus::Queue => Focus::Browser,
        };
        if self.focus == Focus::Queue && self.queue_selected.is_none() && !self.queue_mgr.is_empty()
        {
            self.queue_selected = Some(0);
        }
    }

    fn cycle_album_sort(&mut self) {
        self.album_sort_index = (self.album_sort_index + 1) % ALBUM_SORT_TYPES.len();
        self.load_library_data();
    }

    fn load_artists(&mut self) {
        if let Some(client) = &self.client
            && let Ok(artists) = self.rt.block_on(client.get_artists())
        {
            self.artists = artists;
        }
    }

    fn load_playlists(&mut self) {
        if let Some(client) = &self.client
            && let Ok(playlists) = self.rt.block_on(client.get_playlists())
        {
            self.playlists = playlists;
        }
    }

    fn load_genres(&mut self) {
        if let Some(client) = &self.client
            && let Ok(genres) = self.rt.block_on(client.get_genres())
        {
            self.genres = genres;
        }
    }

    fn load_starred(&mut self) {
        if let Some(client) = &self.client
            && let Ok((_, _, songs)) = self.rt.block_on(client.get_starred())
        {
            self.starred_ids = songs.iter().map(|s| s.id.clone()).collect();
            self.starred_songs = songs;
        }
    }

    fn load_lyrics(&mut self) {
        if let Some(song) = &self.player.current_song
            && let Some(client) = &self.client
        {
            let artist = song.artist.clone();
            let title = song.title.clone();
            if let Ok(lyrics) = self.rt.block_on(client.get_lyrics(&artist, &title)) {
                self.lyrics_text = lyrics;
                self.lyrics_scroll = 0;
            }
        }
    }

    fn toggle_star(&mut self) {
        let song_id = match self.active_tab {
            2 | 5 | 6 => {
                let songs = match self.active_tab {
                    2 => &self.songs,
                    5 => &self.starred_songs,
                    6 => &self.play_history,
                    _ => return,
                };
                self.tab_selected[self.active_tab]
                    .and_then(|idx| songs.get(idx))
                    .map(|s| s.id.clone())
            }
            _ => None,
        };

        if let (Some(id), Some(client)) = (song_id, &self.client) {
            if self.starred_ids.contains(&id) {
                let _ = self.rt.block_on(client.unstar(&id));
                self.starred_ids.remove(&id);
            } else {
                let _ = self.rt.block_on(client.star(&id));
                self.starred_ids.insert(id);
            }
        }
    }

    fn start_radio(&mut self) {
        if let Some(song) = &self.player.current_song
            && let Some(client) = &self.client
        {
            let song_id = song.id.clone();
            if let Ok(songs) = self.rt.block_on(client.get_similar_songs(&song_id, 50))
                && !songs.is_empty()
            {
                self.queue_mgr.set_queue(songs, 0);
                if let Some(song) = self.queue_mgr.current_song().cloned() {
                    self.play_song(&song);
                }
            }
        }
    }

    fn handle_search_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.active_modal = None;
            }
            KeyCode::Char(c) => {
                self.search_query.push(c);
                if self.search_query.len() >= 2 {
                    self.do_search();
                }
            }
            KeyCode::Backspace => {
                self.search_query.pop();
                if self.search_query.len() >= 2 {
                    self.do_search();
                } else {
                    self.search_songs.clear();
                    self.search_albums.clear();
                    self.search_artists.clear();
                }
            }
            KeyCode::Tab => {
                self.search_tab = (self.search_tab + 1) % 3;
                self.search_selected = Some(0);
            }
            KeyCode::Up => {
                let current = self.search_selected.unwrap_or(0);
                if current > 0 {
                    self.search_selected = Some(current - 1);
                }
            }
            KeyCode::Down => {
                let len = match self.search_tab {
                    0 => self.search_songs.len(),
                    1 => self.search_albums.len(),
                    2 => self.search_artists.len(),
                    _ => 0,
                };
                let current = self.search_selected.unwrap_or(0);
                if current + 1 < len {
                    self.search_selected = Some(current + 1);
                }
            }
            KeyCode::Enter => {
                self.select_search_result();
                self.active_modal = None;
            }
            _ => {}
        }
    }

    fn do_search(&mut self) {
        if let Some(client) = &self.client
            && let Ok((artists, albums, songs)) =
                self.rt
                    .block_on(client.search(&self.search_query, 10, 10, 20))
        {
            self.search_artists = artists;
            self.search_albums = albums;
            self.search_songs = songs;
            self.search_selected = Some(0);
        }
    }

    fn select_search_result(&mut self) {
        let idx = self.search_selected.unwrap_or(0);
        match self.search_tab {
            0 => {
                if idx < self.search_songs.len() {
                    let songs = self.search_songs.clone();
                    self.queue_mgr.set_queue(songs, idx);
                    if let Some(song) = self.queue_mgr.current_song().cloned() {
                        self.play_song(&song);
                    }
                }
            }
            1 => {
                if idx < self.search_albums.len() {
                    let album_id = self.search_albums[idx].id.clone();
                    if let Some(client) = &self.client
                        && let Ok((_, songs)) = self.rt.block_on(client.get_album(&album_id))
                    {
                        self.songs = songs;
                        self.active_tab = 2;
                        self.tab_selected[2] = Some(0);
                    }
                }
            }
            2 => {
                if idx < self.search_artists.len() {
                    let artist_id = self.search_artists[idx].id.clone();
                    if let Some(client) = &self.client
                        && let Ok((_, albums)) = self.rt.block_on(client.get_artist(&artist_id))
                    {
                        self.albums = albums;
                        self.active_tab = 0;
                        self.tab_selected[0] = Some(0);
                    }
                }
            }
            _ => {}
        }
    }

    fn handle_server_mgr_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.active_modal = None;
            }
            KeyCode::Tab => {
                if self.server_focus_list {
                    // Switch from server list to form
                    self.server_focus_list = false;
                    self.server_active_field = 0;
                } else {
                    // Cycle through form fields, then back to list
                    if self.server_active_field < 3 {
                        self.server_active_field += 1;
                    } else {
                        self.server_focus_list = true;
                    }
                }
            }
            KeyCode::Up if self.server_focus_list => {
                if let Some(idx) = self.server_selected
                    && idx > 0
                {
                    self.server_selected = Some(idx - 1);
                }
            }
            KeyCode::Down if self.server_focus_list => {
                let len = self.config.servers.len();
                if let Some(idx) = self.server_selected {
                    if idx + 1 < len {
                        self.server_selected = Some(idx + 1);
                    }
                } else if len > 0 {
                    self.server_selected = Some(0);
                }
            }
            KeyCode::Enter if self.server_focus_list => {
                // Switch to selected server
                if let Some(idx) = self.server_selected
                    && idx < self.config.servers.len()
                {
                    self.config.set_active_server(idx);
                    self.config.save();
                    self.connect_to_active_server();
                    self.load_library_data();
                    let name = self.config.servers[idx].name.clone();
                    self.server_status = format!("Switched to '{name}'");
                }
            }
            KeyCode::Enter => {
                // In form mode: try to add server
                let name = self.server_form[0].clone();
                let url = self.server_form[1].clone();
                let username = self.server_form[2].clone();
                let password = self.server_form[3].clone();

                if name.is_empty() || url.is_empty() || username.is_empty() || password.is_empty() {
                    self.server_status = "All fields are required".to_string();
                    return;
                }

                // Test connection
                let test_client = SubsonicClient::new(&url, &username, &password);
                match self.rt.block_on(test_client.ping()) {
                    Ok(true) => {
                        self.config.add_server(&name, &url, &username, &password);
                        self.connect_to_active_server();
                        self.load_library_data();
                        self.server_form = Default::default();
                        self.server_status = format!("Server '{name}' added successfully");
                        self.server_selected = Some(self.config.servers.len().saturating_sub(1));
                    }
                    _ => {
                        self.server_status =
                            "Connection failed. Check URL and credentials.".to_string();
                    }
                }
            }
            KeyCode::Char(c) if !self.server_focus_list => {
                self.server_form[self.server_active_field].push(c);
            }
            KeyCode::Backspace if !self.server_focus_list => {
                self.server_form[self.server_active_field].pop();
            }
            KeyCode::Delete => {
                if let Some(idx) = self.server_selected {
                    self.config.remove_server(idx);
                    let len = self.config.servers.len();
                    if len == 0 {
                        self.server_selected = None;
                    } else if idx >= len {
                        self.server_selected = Some(len - 1);
                    }
                    self.connect_to_active_server();
                }
            }
            _ => {}
        }
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::vertical([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
    ])
    .split(area);

    Layout::horizontal([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
    ])
    .split(popup_layout[1])[1]
}
