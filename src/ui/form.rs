use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::{App, Focus};
use crate::theme::Theme;
use crate::ui::inner;

/// Shared width for h: and m: value slots (0–99 / 0–59).
const DURATION_WIDTH: usize = 2;

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

    // Task flexes; duration fields are fixed-width "inputs" with a trailing inset
    // so m: never sits on the panel edge. Gaps keep the groups from colliding.
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(10),
            Constraint::Length(2),  // gap before hours
            Constraint::Length(6),  // h: ##▌
            Constraint::Length(1),  // gap
            Constraint::Length(6),  // m: ##▌
            Constraint::Length(1),  // right inset
        ])
        .split(inner_area);

    app.rects.name = chunks[0];
    app.rects.hours = chunks[2];
    app.rects.minutes = chunks[4];

    frame.render_widget(
        task_field(&theme, app, app.focus == Focus::TaskName),
        chunks[0],
    );
    frame.render_widget(
        duration_field(
            &theme,
            "h",
            &app.hours_input,
            DURATION_WIDTH,
            app.focus == Focus::Hours,
        ),
        chunks[2],
    );
    frame.render_widget(
        duration_field(
            &theme,
            "m",
            &app.minutes_input,
            DURATION_WIDTH,
            app.focus == Focus::Minutes,
        ),
        chunks[4],
    );
}

fn task_field(theme: &Theme, app: &App, focused: bool) -> Paragraph<'static> {
    let label_style = label_style(theme, focused);
    let mut spans = vec![
        Span::styled("task: ".to_string(), label_style),
        Span::styled(app.task_name.clone(), value_style(theme, focused)),
    ];

    if focused
        && let Some(full) = app.name_completion()
    {
        let rest: String = full.chars().skip(app.task_name.chars().count()).collect();
        if !rest.is_empty() {
            spans.push(Span::styled(rest, Style::default().fg(theme.muted)));
        }
    }

    // Reserved cursor cell — same width focused or not, so the line doesn't jump.
    spans.push(cursor_span(theme, focused));

    Paragraph::new(Line::from(spans)).style(Style::default().bg(theme.surface).fg(theme.text))
}

fn duration_field(
    theme: &Theme,
    label: &str,
    value: &str,
    width: usize,
    focused: bool,
) -> Paragraph<'static> {
    let padded = pad_left(value, width);
    let value_style = if focused {
        Style::default()
            .fg(theme.highlight_fg)
            .bg(theme.highlight_bg)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.text).bg(theme.surface)
    };

    // Fixed-width value slot; highlight fills it when focused.
    let spans = vec![
        Span::styled(format!("{label}: "), label_style(theme, focused)),
        Span::styled(padded, value_style),
        cursor_span(theme, focused),
    ];

    Paragraph::new(Line::from(spans)).style(Style::default().bg(theme.surface).fg(theme.text))
}

fn label_style(theme: &Theme, focused: bool) -> Style {
    if focused {
        Style::default().fg(theme.accent).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.muted)
    }
}

fn value_style(theme: &Theme, focused: bool) -> Style {
    if focused {
        Style::default().fg(theme.text).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.text)
    }
}

fn cursor_span(theme: &Theme, focused: bool) -> Span<'static> {
    if focused {
        Span::styled("▌", Style::default().fg(theme.accent).bg(theme.highlight_bg))
    } else {
        // Keep width stable when unfocused.
        Span::styled(" ", Style::default().bg(theme.surface))
    }
}

fn pad_left(value: &str, width: usize) -> String {
    let chars: Vec<char> = value.chars().collect();
    if chars.len() >= width {
        return chars[chars.len() - width..].iter().collect();
    }
    format!("{:>width$}", value, width = width)
}
