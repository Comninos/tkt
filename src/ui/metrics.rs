use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::App;
use crate::goals::{
    progress_ratio, ProgressLevel, DAY_GOAL_MINUTES, WEEK_GOAL_MINUTES,
};
use crate::theme::Theme;
use crate::ui::inner;
use crate::week::format_hhmm;

/// Fixed suffix: `" 00:00 / 00:00"` (14 chars) so day/week bars share width.
const BAR_SUFFIX_WIDTH: usize = 14;
const BAR_LABEL_WIDTH: usize = 5; // `"Day  "` / `"Week "`

pub fn render(frame: &mut Frame, area: Rect, app: &App, date_label: &str) {
    let theme = app.theme();
    let m = &app.metrics;

    let day_level = ProgressLevel::for_day(m.today_minutes);
    let week_level = ProgressLevel::for_week(m.week_minutes);
    let day_color = day_level.color(&theme);
    let week_color = week_level.color(&theme);

    let title = format!("tkt · {date_label}");
    let status = match &app.mode {
        crate::app::Mode::ConfirmDelete { .. } => None,
        _ if app.status.is_empty() => None,
        _ => Some(app.status.as_str()),
    };
    let block = theme.panel_with_status(title, true, status);
    let inner_area = inner(area);
    frame.render_widget(Paragraph::new("").block(block), area);

    let metrics_line = Line::from(vec![
        Span::styled("Today ", Style::default().fg(theme.muted)),
        Span::styled(
            m.today.clone(),
            Style::default()
                .fg(day_color)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled("Week ", Style::default().fg(theme.muted)),
        Span::styled(
            m.week.clone(),
            Style::default()
                .fg(week_color)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled("Month ", Style::default().fg(theme.muted)),
        Span::styled(
            m.month.clone(),
            Style::default()
                .fg(theme.metric)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled("Tasks ", Style::default().fg(theme.muted)),
        Span::styled(
            m.week_task_count.to_string(),
            Style::default()
                .fg(theme.metric)
                .add_modifier(Modifier::BOLD),
        ),
    ]);

    let day_bar = bar_line(
        &theme,
        "Day",
        m.today_minutes,
        DAY_GOAL_MINUTES,
        day_color,
        inner_area.width,
    );
    let week_bar = bar_line(
        &theme,
        "Week",
        m.week_minutes,
        WEEK_GOAL_MINUTES,
        week_color,
        inner_area.width,
    );

    let body = Paragraph::new(vec![metrics_line, day_bar, week_bar])
        .style(Style::default().bg(theme.surface).fg(theme.text));
    frame.render_widget(body, inner_area);
}

fn bar_line(
    theme: &Theme,
    label: &str,
    minutes: i64,
    goal: i64,
    fill: ratatui::style::Color,
    total_width: u16,
) -> Line<'static> {
    let suffix = format!(" {} / {}", format_hhmm(minutes), format_hhmm(goal));
    debug_assert_eq!(suffix.chars().count(), BAR_SUFFIX_WIDTH);

    let bar_width = (total_width as usize)
        .saturating_sub(BAR_LABEL_WIDTH + BAR_SUFFIX_WIDTH)
        .max(8);

    let ratio = progress_ratio(minutes, goal);
    let filled = ((ratio * bar_width as f64).round() as usize).min(bar_width);
    let empty = bar_width.saturating_sub(filled);

    let mut bar = String::new();
    bar.push_str(&"█".repeat(filled));
    bar.push_str(&"░".repeat(empty));

    Line::from(vec![
        Span::styled(
            format!("{label:<4} "),
            Style::default().fg(theme.muted),
        ),
        Span::styled(bar, Style::default().fg(fill)),
        Span::styled(suffix, Style::default().fg(theme.muted)),
    ])
}
