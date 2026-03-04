use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Paragraph, Row, Table};

use crate::subsonic::Song;
use crate::tui::theme;
use crate::utils;

pub fn render(
    f: &mut Frame,
    area: Rect,
    queue: &[Song],
    current_index: i32,
    selected: Option<usize>,
) {
    let block = Block::default()
        .title(" Play Queue ")
        .title_style(Style::default().fg(theme::PRIMARY))
        .borders(Borders::LEFT)
        .border_style(Style::default().fg(theme::BORDER));

    let inner = block.inner(area);
    f.render_widget(block, area);

    if inner.height < 3 {
        return;
    }

    let chunks = Layout::vertical([
        Constraint::Length(1), // Info
        Constraint::Min(1),    // Queue table
    ])
    .split(inner);

    // Info line
    let total_duration: u64 = queue.iter().map(|s| s.duration).sum();
    let info = Line::from(vec![
        Span::styled(
            format!(" {} songs", queue.len()),
            Style::default().fg(theme::TEXT_MUTED),
        ),
        Span::styled(" │ ", Style::default().fg(theme::BORDER)),
        Span::styled(
            utils::format_duration_long(total_duration),
            Style::default().fg(theme::TEXT_MUTED),
        ),
    ]);
    f.render_widget(Paragraph::new(info), chunks[0]);

    // Queue table
    let rows: Vec<Row> = queue
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let marker = if i as i32 == current_index { ">" } else { " " };
            let style = if i as i32 == current_index {
                Style::default()
                    .fg(theme::PRIMARY)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme::TEXT)
            };
            Row::new(vec![
                Cell::from(marker),
                Cell::from(utils::truncate(&s.title, 22)),
                Cell::from(utils::format_duration(s.duration)),
            ])
            .style(style)
        })
        .collect();

    let widths = [
        Constraint::Length(1),
        Constraint::Min(10),
        Constraint::Length(6),
    ];

    let table = Table::new(rows, widths)
        .block(Block::default().borders(Borders::NONE))
        .row_highlight_style(Style::default().bg(theme::CURSOR_BG).fg(theme::TEXT));

    let mut state = ratatui::widgets::TableState::default();
    state.select(selected);
    f.render_stateful_widget(table, chunks[1], &mut state);
}
