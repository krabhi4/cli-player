use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Wrap};
use ratatui::Frame;

use crate::tui::theme;

pub fn render(f: &mut Frame, area: Rect) {
    let block = Block::default()
        .title(" Help ")
        .title_style(Style::default().fg(theme::PRIMARY))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::BORDER))
        .style(Style::default().bg(theme::SURFACE_DARK));

    f.render_widget(Clear, area);

    let help_text = vec![
        Line::from(Span::styled(
            "Playback",
            Style::default().fg(theme::PRIMARY),
        )),
        Line::from("  Space     Toggle play/pause"),
        Line::from("  n         Next track"),
        Line::from("  p         Previous track"),
        Line::from("  s         Stop"),
        Line::from("  →/←       Seek ±5s"),
        Line::from("  Shift+→/← Seek ±30s"),
        Line::from(""),
        Line::from(Span::styled(
            "Volume",
            Style::default().fg(theme::PRIMARY),
        )),
        Line::from("  +/=       Volume up"),
        Line::from("  -/_       Volume down"),
        Line::from("  m         Mute toggle"),
        Line::from(""),
        Line::from(Span::styled(
            "Queue",
            Style::default().fg(theme::PRIMARY),
        )),
        Line::from("  a         Add to queue"),
        Line::from("  d/Delete  Remove from queue"),
        Line::from("  c         Clear queue"),
        Line::from("  Shift+↑/↓ Reorder queue"),
        Line::from("  z         Toggle shuffle"),
        Line::from("  r         Cycle repeat"),
        Line::from(""),
        Line::from(Span::styled(
            "Navigation",
            Style::default().fg(theme::PRIMARY),
        )),
        Line::from("  1-7       Switch tab"),
        Line::from("  Tab       Cycle focus"),
        Line::from("  Enter     Select/play"),
        Line::from("  Esc/Bksp  Go back"),
        Line::from("  /         Search"),
        Line::from(""),
        Line::from(Span::styled(
            "Other",
            Style::default().fg(theme::PRIMARY),
        )),
        Line::from("  e         Toggle equalizer"),
        Line::from("  l         Toggle lyrics"),
        Line::from("  f         Star/unstar"),
        Line::from("  R         Artist radio"),
        Line::from("  P         Save as playlist"),
        Line::from("  o         Cycle album sort"),
        Line::from("  S         Server manager"),
        Line::from("  ?/i       This help"),
        Line::from("  q         Quit"),
    ];

    let paragraph = Paragraph::new(help_text)
        .block(block)
        .style(Style::default().fg(theme::TEXT))
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}
