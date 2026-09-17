use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::{App, Focus};
use crate::theme::Theme;
use crate::ui::inner;

pub fn render(frame: &mut Frame, area: Rect, app: &mut App) {
    let theme = app.theme();
    let form_focused = matches!(
        app.focus,
        Focus::TaskName | Focus::Hours | Focus::Minutes
    );
    let block = theme.panel(app.form_title(), form_focused);
    let inner_area = inner(area);
    frame.render_widget(block, area);
    app.rects.form = area;

    // Name fills the left; h/m sit flush on the right edge.
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(8),
            Constraint::Length(8),
            Constraint::Length(8),
        ])
        .split(inner_area);

    app.rects.name = chunks[0];
    app.rects.hours = chunks[1];
    app.rects.minutes = chunks[2];

    frame.render_widget(
        field(&theme, "name", &app.task_name, app.focus == Focus::TaskName, false),
        chunks[0],
    );
    frame.render_widget(
        field(&theme, "h", &app.hours_input, app.focus == Focus::Hours, true),
        chunks[1],
    );
    frame.render_widget(
        field(&theme, "m", &app.minutes_input, app.focus == Focus::Minutes, true),
        chunks[2],
    );
}

fn field(
    theme: &Theme,
    label: &str,
    value: &str,
    focused: bool,
    right_align: bool,
) -> Paragraph<'static> {
    let cursor = if focused { "▌" } else { "" };
    let label_style = if focused {
        Style::default().fg(theme.accent)
    } else {
        Style::default().fg(theme.muted)
    };
    let paragraph = Paragraph::new(Line::from(vec![
        Span::styled(format!("{label}: "), label_style),
        Span::styled(value.to_string(), Style::default().fg(theme.text)),
        Span::styled(cursor.to_string(), Style::default().fg(theme.accent)),
    ]))
    .style(Style::default().bg(theme.surface).fg(theme.text));

    if right_align {
        paragraph.alignment(Alignment::Right)
    } else {
        paragraph
    }
}
