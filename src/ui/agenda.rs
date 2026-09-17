use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::{App, Focus};
use crate::ui::inner;
use crate::week::window_start;

pub fn render(frame: &mut Frame, area: Rect, app: &mut App) {
    let theme = app.theme();
    let focused = app.focus == Focus::AgendaInput;
    let block = theme.panel(app.agenda_title(), focused);
    let inner_area = inner(area);
    frame.render_widget(block, area);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(inner_area);

    let list_area = chunks[0];
    let input_area = chunks[1];

    app.rects.agenda = area;
    app.rects.agenda_input = input_area;
    app.rects.agenda_rows.clear();
    app.rects.agenda_row_indices.clear();

    let max_rows = list_area.height as usize;
    let start = window_start(app.agenda_selected, app.agenda.len(), max_rows);
    let mut lines: Vec<Line> = Vec::new();

    for (vis, (abs, item)) in app
        .agenda
        .iter()
        .enumerate()
        .skip(start)
        .take(max_rows)
        .enumerate()
    {
        let selected = abs == app.agenda_selected;
        let row_rect = Rect {
            x: list_area.x,
            y: list_area.y.saturating_add(vis as u16),
            width: list_area.width,
            height: 1,
        };
        app.rects.agenda_rows.push(row_rect);
        app.rects.agenda_row_indices.push(abs);

        let style = if selected {
            Style::default()
                .fg(theme.highlight_fg)
                .bg(theme.highlight_bg)
                .add_modifier(Modifier::BOLD)
        } else if item.completed {
            Style::default().fg(theme.muted).bg(theme.surface)
        } else {
            Style::default().fg(theme.text).bg(theme.surface)
        };
        let mark = if item.completed { "[x] " } else { "[ ] " };
        let name_style = if item.completed {
            style.add_modifier(Modifier::CROSSED_OUT)
        } else {
            style
        };
        lines.push(Line::from(vec![
            Span::styled(mark, style),
            Span::styled(item.name.clone(), name_style),
        ]));
    }

    if app.agenda.is_empty() {
        lines.push(Line::from(Span::styled(
            "(empty)",
            Style::default().fg(theme.muted).bg(theme.surface),
        )));
    }

    frame.render_widget(
        Paragraph::new(lines).style(Style::default().bg(theme.surface)),
        list_area,
    );

    let cursor = if focused { "▌" } else { "" };
    let input_line = Line::from(vec![
        Span::styled("> ", Style::default().fg(theme.accent)),
        Span::styled(app.agenda_input.clone(), Style::default().fg(theme.text)),
        Span::styled(cursor, Style::default().fg(theme.accent)),
    ]);
    frame.render_widget(
        Paragraph::new(input_line).style(Style::default().bg(theme.surface)),
        input_area,
    );
}
