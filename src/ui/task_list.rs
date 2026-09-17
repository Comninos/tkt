use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{List, ListItem, ListState};
use ratatui::Frame;

use crate::app::{App, Focus};
use crate::ui::inner;
use crate::week::{format_clock, window_start};

pub fn render(frame: &mut Frame, area: Rect, app: &mut App) {
    let theme = app.theme();
    let focused = app.focus == Focus::TaskList;
    let block = theme.panel(app.tasks_title(), focused);

    app.rects.tasks = area;
    app.rects.task_rows.clear();
    app.rects.task_row_indices.clear();

    let inner_area = inner(area);
    let max_rows = inner_area.height as usize;
    let width = inner_area.width as usize;
    let selected = app.task_list_state.selected();
    let start = window_start(selected.unwrap_or(0), app.tasks.len(), max_rows);

    let items: Vec<ListItem> = app
        .tasks
        .iter()
        .enumerate()
        .skip(start)
        .take(max_rows)
        .enumerate()
        .map(|(vis, (abs, task))| {
            let row_rect = Rect {
                x: inner_area.x,
                y: inner_area.y.saturating_add(vis as u16),
                width: inner_area.width,
                height: 1,
            };
            app.rects.task_rows.push(row_rect);
            app.rects.task_row_indices.push(abs);

            let marker = if selected == Some(abs) { "› " } else { "  " };
            let clock = format_clock(&task.logged_at);
            let dur = format!("{:>2}h {:02}m", task.hours, task.minutes);
            let fixed = marker.chars().count()
                + clock.chars().count()
                + 2
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

    // Highlight relative to the visible window.
    let mut state = ListState::default();
    if let Some(sel) = selected
        && sel >= start
        && sel < start + items.len()
    {
        state.select(Some(sel - start));
    }

    let list = List::new(items)
        .block(block)
        .highlight_style(
            Style::default()
                .fg(theme.highlight_fg)
                .bg(theme.highlight_bg)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("");

    frame.render_stateful_widget(list, area, &mut state);
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
