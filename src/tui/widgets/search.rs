use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table, Tabs};
use ratatui::Frame;

use crate::subsonic::{Album, Artist, Song};
use crate::tui::theme;
use crate::utils;

pub struct SearchState<'a> {
    pub query: &'a str,
    pub artists: &'a [Artist],
    pub albums: &'a [Album],
    pub songs: &'a [Song],
    pub active_tab: usize,
    pub selected: Option<usize>,
}

pub fn render(f: &mut Frame, area: Rect, state: &SearchState) {
    let block = Block::default()
        .title(" Search ")
        .title_style(Style::default().fg(theme::PRIMARY))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::BORDER))
        .style(Style::default().bg(theme::SURFACE_DARK));

    f.render_widget(Clear, area);
    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::vertical([
        Constraint::Length(1), // Input
        Constraint::Length(1), // Tabs
        Constraint::Min(1),   // Results
    ])
    .split(inner);

    // Search input
    let input = Line::from(vec![
        Span::styled(" / ", Style::default().fg(theme::PRIMARY)),
        Span::styled(state.query, Style::default().fg(theme::TEXT)),
        Span::styled("_", Style::default().fg(theme::PRIMARY)),
    ]);
    f.render_widget(Paragraph::new(input), chunks[0]);

    // Result tabs
    let tab_titles = [
        format!("Songs ({})", state.songs.len()),
        format!("Albums ({})", state.albums.len()),
        format!("Artists ({})", state.artists.len()),
    ];
    let titles: Vec<Span> = tab_titles
        .iter()
        .enumerate()
        .map(|(i, t)| {
            if i == state.active_tab {
                Span::styled(t.as_str(), Style::default().fg(theme::PRIMARY))
            } else {
                Span::styled(t.as_str(), Style::default().fg(theme::TEXT_MUTED))
            }
        })
        .collect();

    let tabs = Tabs::new(titles)
        .select(state.active_tab)
        .divider(" │ ");
    f.render_widget(tabs, chunks[1]);

    // Results
    match state.active_tab {
        0 => {
            let rows: Vec<Row> = state
                .songs
                .iter()
                .map(|s| {
                    Row::new(vec![
                        Cell::from(s.title.as_str()),
                        Cell::from(s.artist.as_str()),
                        Cell::from(utils::format_duration(s.duration)),
                    ])
                })
                .collect();
            let widths = [
                Constraint::Percentage(40),
                Constraint::Percentage(35),
                Constraint::Length(6),
            ];
            let table = Table::new(rows, widths)
                .row_highlight_style(Style::default().bg(theme::CURSOR_BG));
            let mut tstate = ratatui::widgets::TableState::default();
            tstate.select(state.selected);
            f.render_stateful_widget(table, chunks[2], &mut tstate);
        }
        1 => {
            let rows: Vec<Row> = state
                .albums
                .iter()
                .map(|a| {
                    Row::new(vec![
                        Cell::from(a.name.as_str()),
                        Cell::from(a.artist.as_str()),
                    ])
                })
                .collect();
            let widths = [Constraint::Percentage(50), Constraint::Percentage(40)];
            let table = Table::new(rows, widths)
                .row_highlight_style(Style::default().bg(theme::CURSOR_BG));
            let mut tstate = ratatui::widgets::TableState::default();
            tstate.select(state.selected);
            f.render_stateful_widget(table, chunks[2], &mut tstate);
        }
        2 => {
            let rows: Vec<Row> = state
                .artists
                .iter()
                .map(|a| Row::new(vec![Cell::from(a.name.as_str())]))
                .collect();
            let widths = [Constraint::Percentage(80)];
            let table = Table::new(rows, widths)
                .row_highlight_style(Style::default().bg(theme::CURSOR_BG));
            let mut tstate = ratatui::widgets::TableState::default();
            tstate.select(state.selected);
            f.render_stateful_widget(table, chunks[2], &mut tstate);
        }
        _ => {}
    }
}
