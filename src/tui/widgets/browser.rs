use ratatui::layout::{Constraint, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{Block, Borders, Cell, Row, Table, Tabs};
use ratatui::Frame;

use crate::subsonic::{Album, Artist, Genre, Playlist, Song};
use crate::tui::theme;
use crate::utils;

pub const TAB_TITLES: &[&str] = &[
    "Albums",
    "Artists",
    "Songs",
    "Playlists",
    "Genres",
    "Starred",
    "History",
];

pub fn render_tabs(f: &mut Frame, area: Rect, active_tab: usize) {
    let titles: Vec<Span> = TAB_TITLES
        .iter()
        .enumerate()
        .map(|(i, t)| {
            if i == active_tab {
                Span::styled(*t, Style::default().fg(theme::PRIMARY))
            } else {
                Span::styled(*t, Style::default().fg(theme::TEXT_MUTED))
            }
        })
        .collect();

    let tabs = Tabs::new(titles)
        .select(active_tab)
        .divider(Span::raw(" │ "))
        .highlight_style(
            Style::default()
                .fg(theme::PRIMARY)
                .add_modifier(Modifier::BOLD),
        );

    f.render_widget(tabs, area);
}

pub fn render_albums_table(
    f: &mut Frame,
    area: Rect,
    albums: &[Album],
    selected: Option<usize>,
) {
    let rows: Vec<Row> = albums
        .iter()
        .map(|a| {
            Row::new(vec![
                Cell::from(a.name.as_str()),
                Cell::from(a.artist.as_str()),
                Cell::from(if a.year > 0 {
                    a.year.to_string()
                } else {
                    String::new()
                }),
                Cell::from(a.song_count.to_string()),
            ])
        })
        .collect();

    let widths = [
        Constraint::Percentage(40),
        Constraint::Percentage(30),
        Constraint::Percentage(10),
        Constraint::Percentage(10),
    ];

    let header = Row::new(vec!["Album", "Artist", "Year", "Tracks"])
        .style(Style::default().fg(theme::SECONDARY))
        .bottom_margin(1);

    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default().borders(Borders::NONE))
        .row_highlight_style(
            Style::default()
                .bg(theme::CURSOR_BG)
                .fg(theme::TEXT),
        );

    let mut state = ratatui::widgets::TableState::default();
    state.select(selected);
    f.render_stateful_widget(table, area, &mut state);
}

pub fn render_artists_table(
    f: &mut Frame,
    area: Rect,
    artists: &[Artist],
    selected: Option<usize>,
) {
    let rows: Vec<Row> = artists
        .iter()
        .map(|a| {
            Row::new(vec![
                Cell::from(a.name.as_str()),
                Cell::from(a.album_count.to_string()),
            ])
        })
        .collect();

    let widths = [Constraint::Percentage(70), Constraint::Percentage(20)];

    let header = Row::new(vec!["Artist", "Albums"])
        .style(Style::default().fg(theme::SECONDARY))
        .bottom_margin(1);

    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default().borders(Borders::NONE))
        .row_highlight_style(
            Style::default()
                .bg(theme::CURSOR_BG)
                .fg(theme::TEXT),
        );

    let mut state = ratatui::widgets::TableState::default();
    state.select(selected);
    f.render_stateful_widget(table, area, &mut state);
}

pub fn render_songs_table(
    f: &mut Frame,
    area: Rect,
    songs: &[Song],
    selected: Option<usize>,
) {
    let rows: Vec<Row> = songs
        .iter()
        .enumerate()
        .map(|(i, s)| {
            Row::new(vec![
                Cell::from(format!("{}", i + 1)),
                Cell::from(s.title.as_str()),
                Cell::from(s.artist.as_str()),
                Cell::from(s.album.as_str()),
                Cell::from(utils::format_duration(s.duration)),
            ])
        })
        .collect();

    let widths = [
        Constraint::Length(4),
        Constraint::Percentage(30),
        Constraint::Percentage(25),
        Constraint::Percentage(25),
        Constraint::Length(6),
    ];

    let header = Row::new(vec!["#", "Title", "Artist", "Album", "Time"])
        .style(Style::default().fg(theme::SECONDARY))
        .bottom_margin(1);

    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default().borders(Borders::NONE))
        .row_highlight_style(
            Style::default()
                .bg(theme::CURSOR_BG)
                .fg(theme::TEXT),
        );

    let mut state = ratatui::widgets::TableState::default();
    state.select(selected);
    f.render_stateful_widget(table, area, &mut state);
}

pub fn render_playlists_table(
    f: &mut Frame,
    area: Rect,
    playlists: &[Playlist],
    selected: Option<usize>,
) {
    let rows: Vec<Row> = playlists
        .iter()
        .map(|p| {
            Row::new(vec![
                Cell::from(p.name.as_str()),
                Cell::from(p.song_count.to_string()),
                Cell::from(utils::format_duration_long(p.duration)),
            ])
        })
        .collect();

    let widths = [
        Constraint::Percentage(50),
        Constraint::Percentage(20),
        Constraint::Percentage(20),
    ];

    let header = Row::new(vec!["Name", "Songs", "Duration"])
        .style(Style::default().fg(theme::SECONDARY))
        .bottom_margin(1);

    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default().borders(Borders::NONE))
        .row_highlight_style(
            Style::default()
                .bg(theme::CURSOR_BG)
                .fg(theme::TEXT),
        );

    let mut state = ratatui::widgets::TableState::default();
    state.select(selected);
    f.render_stateful_widget(table, area, &mut state);
}

pub fn render_genres_table(
    f: &mut Frame,
    area: Rect,
    genres: &[Genre],
    selected: Option<usize>,
) {
    let rows: Vec<Row> = genres
        .iter()
        .map(|g| {
            Row::new(vec![
                Cell::from(g.name.as_str()),
                Cell::from(g.song_count.to_string()),
                Cell::from(g.album_count.to_string()),
            ])
        })
        .collect();

    let widths = [
        Constraint::Percentage(50),
        Constraint::Percentage(20),
        Constraint::Percentage(20),
    ];

    let header = Row::new(vec!["Genre", "Songs", "Albums"])
        .style(Style::default().fg(theme::SECONDARY))
        .bottom_margin(1);

    let table = Table::new(rows, widths)
        .header(header)
        .block(Block::default().borders(Borders::NONE))
        .row_highlight_style(
            Style::default()
                .bg(theme::CURSOR_BG)
                .fg(theme::TEXT),
        );

    let mut state = ratatui::widgets::TableState::default();
    state.select(selected);
    f.render_stateful_widget(table, area, &mut state);
}
