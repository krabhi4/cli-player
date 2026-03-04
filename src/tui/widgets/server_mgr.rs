use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table};

use crate::config::models::ServerConfig;
use crate::tui::theme;

pub struct ServerMgrState<'a> {
    pub servers: &'a [ServerConfig],
    pub active_index: i32,
    pub selected: Option<usize>,
    pub form_fields: &'a [String; 4],
    pub active_field: usize,
    pub status_msg: &'a str,
    pub focus_list: bool,
}

pub fn render(f: &mut Frame, area: Rect, state: &ServerMgrState) {
    let block = Block::default()
        .title(" Server Manager ")
        .title_style(Style::default().fg(theme::PRIMARY))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme::BORDER))
        .style(Style::default().bg(theme::SURFACE_DARK));

    f.render_widget(Clear, area);
    let inner = block.inner(area);
    f.render_widget(block, area);

    let chunks = Layout::vertical([
        Constraint::Length(1), // Help text
        Constraint::Min(5),    // Server list
        Constraint::Length(1), // Separator
        Constraint::Length(6), // Add form
        Constraint::Length(1), // Status
    ])
    .split(inner);

    // Help text
    let help = if state.focus_list {
        " ↑↓ select │ Enter switch │ Del remove │ Tab add form │ Esc close"
    } else {
        " Tab next field │ Enter add │ Esc close"
    };
    f.render_widget(
        Paragraph::new(Span::styled(help, Style::default().fg(theme::TEXT_MUTED))),
        chunks[0],
    );

    // Server list
    let list_border_style = if state.focus_list {
        Style::default().fg(theme::PRIMARY)
    } else {
        Style::default().fg(theme::BORDER)
    };

    let rows: Vec<Row> = state
        .servers
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let active_mark = if i as i32 == state.active_index {
                "✓"
            } else {
                " "
            };
            Row::new(vec![
                Cell::from(active_mark),
                Cell::from(s.name.as_str()),
                Cell::from(s.url.as_str()),
                Cell::from(s.username.as_str()),
            ])
        })
        .collect();

    let widths = [
        Constraint::Length(1),
        Constraint::Percentage(25),
        Constraint::Percentage(40),
        Constraint::Percentage(20),
    ];

    let header = Row::new(vec![" ", "Name", "URL", "User"])
        .style(Style::default().fg(theme::SECONDARY))
        .bottom_margin(1);

    let table = Table::new(rows, widths)
        .header(header)
        .block(
            Block::default()
                .borders(Borders::TOP)
                .border_style(list_border_style)
                .title(" Servers ")
                .title_style(list_border_style),
        )
        .row_highlight_style(Style::default().bg(theme::CURSOR_BG));

    let mut tstate = ratatui::widgets::TableState::default();
    tstate.select(state.selected);
    f.render_stateful_widget(table, chunks[1], &mut tstate);

    // Add form
    let form_border_style = if !state.focus_list {
        Style::default().fg(theme::PRIMARY)
    } else {
        Style::default().fg(theme::BORDER)
    };

    let field_labels = ["Name:", "URL:", "Username:", "Password:"];
    let form_chunks = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .split(chunks[3]);

    f.render_widget(
        Paragraph::new(Span::styled(
            " Add Server",
            form_border_style.add_modifier(Modifier::BOLD),
        )),
        form_chunks[0],
    );

    for (i, label) in field_labels.iter().enumerate() {
        if i + 1 >= form_chunks.len() {
            break;
        }
        let is_active = !state.focus_list && state.active_field == i;
        let display_val = if i == 3 {
            "*".repeat(state.form_fields[i].len())
        } else {
            state.form_fields[i].clone()
        };

        let style = if is_active {
            Style::default().fg(theme::PRIMARY)
        } else {
            Style::default().fg(theme::TEXT)
        };

        let line = Line::from(vec![
            Span::styled(
                format!(" {label:<10}"),
                Style::default().fg(theme::TEXT_MUTED),
            ),
            Span::styled(&display_val, style),
            if is_active {
                Span::styled("_", Style::default().fg(theme::PRIMARY))
            } else {
                Span::raw("")
            },
        ]);
        f.render_widget(Paragraph::new(line), form_chunks[i + 1]);
    }

    // Status
    if !state.status_msg.is_empty() {
        f.render_widget(
            Paragraph::new(Span::styled(
                format!(" {}", state.status_msg),
                Style::default().fg(theme::WARNING),
            )),
            chunks[4],
        );
    }
}
