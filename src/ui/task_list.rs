use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{List, ListItem};
use ratatui::Frame;

use crate::app::{App, Focus};
use crate::ui::inner;
use crate::week::format_clock;

pub fn render(frame: &mut Frame, area: Rect, app: &mut App) {
    let theme = app.theme();
    let focused = app.focus == Focus::TaskList;
    let block = theme.panel("today", focused);

    app.rects.tasks = area;
    app.rects.task_rows.clear();

    let inner_area = inner(area);
    let max_rows = inner_area.height as usize;
    let width = inner_area.width as usize;
    let selected = app.task_list_state.selected();

    let items: Vec<ListItem> = app
        .tasks
        .iter()
        .enumerate()
        .take(max_rows)
        .map(|(i, task)| {
            let row_rect = Rect {
                x: inner_area.x,
                y: inner_area.y.saturating_add(i as u16),
                width: inner_area.width,
                height: 1,
            };
            app.rects.task_rows.push(row_rect);

            // Stable marker width so the duration column stays flush right.
            let marker = if selected == Some(i) { "› " } else { "  " };
            let clock = format_clock(&task.logged_at); // HH:MM
            let dur = format!("{:>2}h {:02}m", task.hours, task.minutes);
            let fixed = marker.chars().count()
                + clock.chars().count()
                + 2 // gap after clock
                + dur.chars().count();
            let name_width = width.saturating_sub(fixed);
            let name = pad_truncate(&task.name, name_width);

            let line = Line::from(vec![
                Span::styled(marker.to_string(), Style::default().fg(theme.accent)),
                Span::styled(clock, Style::default().fg(theme.muted)),
                Span::raw("  "),
                Span::styled(name, Style::default().fg(theme.text)),
                Span::styled(dur, Style::default().fg(theme.metric)),
            ]);
            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .fg(theme.highlight_fg)
                .bg(theme.highlight_bg)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(""); // marker drawn in-row for stable columns

    frame.render_stateful_widget(list, area, &mut app.task_list_state);
}

fn pad_truncate(s: &str, width: usize) -> String {
    if width == 0 {
        return String::new();
    }
    let chars: Vec<char> = s.chars().collect();
    if chars.len() <= width {
        let mut out: String = chars.into_iter().collect();
        while out.chars().count() < width {
            out.push(' ');
        }
        return out;
    }
    if width == 1 {
        return "…".into();
    }
    let mut out: String = chars.into_iter().take(width - 1).collect();
    out.push('…');
    out
}
