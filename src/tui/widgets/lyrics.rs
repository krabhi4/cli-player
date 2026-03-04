use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use ratatui::Frame;

use crate::tui::theme;

pub fn render(f: &mut Frame, area: Rect, title: &str, artist: &str, lyrics: &str, scroll: u16) {
    let block = Block::default()
        .title(" Lyrics ")
        .title_style(Style::default().fg(theme::PRIMARY))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::BORDER));

    let mut lines = vec![
        Line::from(Span::styled(
            title,
            Style::default()
                .fg(theme::TEXT)
                .add_modifier(Modifier::BOLD),
        )),
        Line::from(Span::styled(artist, Style::default().fg(theme::SECONDARY))),
        Line::from(""),
    ];

    if lyrics.is_empty() {
        lines.push(Line::from(Span::styled(
            "No lyrics available",
            Style::default().fg(theme::TEXT_MUTED),
        )));
    } else {
        for line in lyrics.lines() {
            lines.push(Line::from(Span::styled(
                line,
                Style::default().fg(theme::TEXT),
            )));
        }
    }

    let paragraph = Paragraph::new(lines)
        .block(block)
        .wrap(Wrap { trim: false })
        .scroll((scroll, 0));

    f.render_widget(paragraph, area);
}
