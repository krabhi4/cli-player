use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::equalizer::{EQ_BAND_LABELS, GAIN_MAX, GAIN_MIN};
use crate::tui::theme;

pub fn render(
    f: &mut Frame,
    area: Rect,
    gains: &[f64],
    enabled: bool,
    preset_name: &str,
    selected_band: usize,
) {
    let block = Block::default()
        .title(" Equalizer ")
        .title_style(Style::default().fg(theme::PRIMARY))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::BORDER))
        .style(Style::default().bg(theme::SURFACE_DARK));

    f.render_widget(Clear, area);
    let inner = block.inner(area);
    f.render_widget(block, area);

    if inner.height < 6 || inner.width < 40 {
        return;
    }

    let chunks = Layout::vertical([
        Constraint::Length(1), // Controls row
        Constraint::Length(1), // Spacer
        Constraint::Min(1),   // Band sliders
        Constraint::Length(1), // Gain labels
        Constraint::Length(1), // Freq labels
    ])
    .split(inner);

    // Controls row
    let enabled_str = if enabled { "ON" } else { "OFF" };
    let enabled_style = if enabled {
        Style::default().fg(theme::SUCCESS)
    } else {
        Style::default().fg(theme::ERROR)
    };

    let controls = Line::from(vec![
        Span::styled(" Preset: ", Style::default().fg(theme::TEXT_MUTED)),
        Span::styled(
            preset_name,
            Style::default()
                .fg(theme::PRIMARY)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" │ ", Style::default().fg(theme::BORDER)),
        Span::styled(enabled_str, enabled_style),
        Span::styled(
            " │ ←→ band, ↑↓ adjust, p preset, r reset",
            Style::default().fg(theme::TEXT_MUTED),
        ),
    ]);
    f.render_widget(Paragraph::new(controls), chunks[0]);

    // Band sliders
    let slider_area = chunks[2];
    let band_width = (slider_area.width as usize) / 18;
    if band_width < 2 {
        return;
    }

    let slider_height = slider_area.height as usize;
    let db_range = GAIN_MAX - GAIN_MIN;

    for (i, &gain) in gains.iter().take(18).enumerate() {
        let x = slider_area.x + (i * band_width) as u16;
        let is_selected = i == selected_band;

        // Draw vertical bar for this band
        let normalized = ((gain - GAIN_MIN) / db_range * slider_height as f64) as usize;
        let normalized = normalized.min(slider_height);

        for row in 0..slider_height {
            let y = slider_area.y + (slider_height - 1 - row) as u16;
            let ch = if row < normalized { "█" } else { "░" };
            let style = if is_selected {
                if row < normalized {
                    Style::default().fg(theme::PRIMARY)
                } else {
                    Style::default().fg(theme::BORDER)
                }
            } else if row < normalized {
                Style::default().fg(theme::SECONDARY)
            } else {
                Style::default().fg(theme::BORDER)
            };
            let span = Span::styled(ch, style);
            f.render_widget(
                Paragraph::new(Line::from(span)),
                Rect::new(x + 1, y, 1, 1),
            );
        }

        // Gain value label
        let gain_str = format!("{:+.0}", gain);
        let label_style = if is_selected {
            Style::default()
                .fg(theme::PRIMARY)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(theme::TEXT_MUTED)
        };
        let gain_label_area = chunks[3];
        if band_width >= gain_str.len() {
            f.render_widget(
                Paragraph::new(Span::styled(&gain_str, label_style)),
                Rect::new(x, gain_label_area.y, gain_str.len() as u16, 1),
            );
        }
    }

    // Frequency labels
    let freq_area = chunks[4];
    for (i, label) in EQ_BAND_LABELS.iter().enumerate() {
        let x = freq_area.x + (i * band_width) as u16;
        let style = if i == selected_band {
            Style::default().fg(theme::PRIMARY)
        } else {
            Style::default().fg(theme::TEXT_MUTED)
        };
        let w = label.len().min(band_width) as u16;
        f.render_widget(
            Paragraph::new(Span::styled(*label, style)),
            Rect::new(x, freq_area.y, w, 1),
        );
    }
}
