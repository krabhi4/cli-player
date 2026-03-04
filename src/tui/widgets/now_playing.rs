use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::player::PlaybackState;
use crate::queue::RepeatMode;
use crate::subsonic::Song;
use crate::tui::theme;
use crate::utils;

pub struct NowPlayingState<'a> {
    pub current_song: Option<&'a Song>,
    pub state: PlaybackState,
    pub position: f64,
    pub duration: f64,
    pub volume: u32,
    pub muted: bool,
    pub shuffle: bool,
    pub repeat: RepeatMode,
    pub eq_enabled: bool,
    pub server_name: &'a str,
}

pub fn render(f: &mut Frame, area: Rect, np: &NowPlayingState) {
    let block = Block::default()
        .borders(Borders::TOP)
        .border_style(Style::default().fg(theme::BORDER));

    let inner = block.inner(area);
    f.render_widget(block, area);

    if inner.height < 3 {
        return;
    }

    let chunks = Layout::vertical([
        Constraint::Length(1), // Title line
        Constraint::Length(1), // Seekbar
        Constraint::Length(1), // Controls + info
    ])
    .split(inner);

    // Row 1: State icon + title + artist [album]
    let state_icon = match np.state {
        PlaybackState::Playing => "▶",
        PlaybackState::Paused => "⏸",
        PlaybackState::Stopped => "⏹",
    };

    let title_line = if let Some(song) = np.current_song {
        Line::from(vec![
            Span::styled(
                format!(" {state_icon} "),
                Style::default().fg(theme::PRIMARY),
            ),
            Span::styled(
                &song.title,
                Style::default()
                    .fg(theme::TEXT)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" - ", Style::default().fg(theme::TEXT_MUTED)),
            Span::styled(&song.artist, Style::default().fg(theme::SECONDARY)),
            Span::styled(
                format!(" [{}]", song.album),
                Style::default().fg(theme::TEXT_MUTED),
            ),
            if song.bitrate > 0 {
                Span::styled(
                    format!(" {}kbps", song.bitrate),
                    Style::default().fg(theme::TEXT_MUTED),
                )
            } else {
                Span::raw("")
            },
        ])
    } else {
        Line::from(Span::styled(
            format!(" {state_icon} No track playing"),
            Style::default().fg(theme::TEXT_MUTED),
        ))
    };
    f.render_widget(Paragraph::new(title_line), chunks[0]);

    // Row 2: Seekbar
    let pos_secs = np.position as u64;
    let dur_secs = np.duration as u64;
    let bar_width = chunks[1].width.saturating_sub(16) as usize;
    let filled = if np.duration > 0.0 {
        ((np.position / np.duration) * bar_width as f64) as usize
    } else {
        0
    };
    let empty = bar_width.saturating_sub(filled);

    let seekbar = Line::from(vec![
        Span::styled(
            format!(" {} ", utils::format_duration(pos_secs)),
            Style::default().fg(theme::TEXT),
        ),
        Span::styled("━".repeat(filled), Style::default().fg(theme::PRIMARY)),
        Span::styled("╌".repeat(empty), Style::default().fg(theme::BORDER)),
        Span::styled(
            format!(" {} ", utils::format_duration(dur_secs)),
            Style::default().fg(theme::TEXT_MUTED),
        ),
    ]);
    f.render_widget(Paragraph::new(seekbar), chunks[1]);

    // Row 3: Controls (left) + repeat/server (right)
    let vol_str = if np.muted {
        "♪ MUTED".to_string()
    } else {
        format!("♪ {}%", np.volume)
    };

    let shuffle_span = if np.shuffle {
        Span::styled(" ⇌ ", Style::default().fg(theme::SUCCESS))
    } else {
        Span::styled(" ⇌ ", Style::default().fg(theme::TEXT_MUTED))
    };

    let eq_span = if np.eq_enabled {
        Span::styled("EQ ", Style::default().fg(theme::SUCCESS))
    } else {
        Span::styled("EQ ", Style::default().fg(theme::TEXT_MUTED))
    };

    let repeat_style = if np.repeat != RepeatMode::Off {
        Style::default().fg(theme::SUCCESS)
    } else {
        Style::default().fg(theme::TEXT_MUTED)
    };

    // Build left side: " ◁◁ ▶ ▷▷ │  ⇌ EQ │ ♪ xx%"
    let left_text = format!(" ◁◁ {} ▷▷ │ ", state_icon,);
    let shuffle_str = " ⇌ ";
    let eq_str = "EQ ";
    let vol_part = format!("│ {vol_str}");
    let left_width = left_text.len() + shuffle_str.len() + eq_str.len() + vol_part.len();

    // Build right side: " ↻ Off │ ServerName "
    let repeat_text = format!("{} {}", np.repeat.icon(), np.repeat.label());
    let right_text = format!("{repeat_text} │ {} ", np.server_name);
    let right_width = right_text.len();

    // Fill gap between left and right
    let row_width = chunks[2].width as usize;
    let gap = row_width.saturating_sub(left_width + right_width);

    let controls = Line::from(vec![
        Span::raw(" "),
        Span::styled("◁◁", Style::default().fg(theme::TEXT_MUTED)),
        Span::raw(" "),
        Span::styled(state_icon, Style::default().fg(theme::PRIMARY)),
        Span::raw(" "),
        Span::styled("▷▷", Style::default().fg(theme::TEXT_MUTED)),
        Span::raw(" │ "),
        shuffle_span,
        eq_span,
        Span::styled(vol_part, Style::default().fg(theme::SECONDARY)),
        Span::raw(" ".repeat(gap)),
        Span::styled(repeat_text, repeat_style),
        Span::styled(
            format!(" │ {} ", np.server_name),
            Style::default().fg(theme::TEXT_MUTED),
        ),
    ]);
    f.render_widget(Paragraph::new(controls), chunks[2]);
}
