mod agenda;
mod form;
mod help;
mod metrics;
mod task_list;

use chrono::Local;
use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::widgets::Block;
use ratatui::Frame;

use crate::app::App;

/// Portrait-friendly content width; wider terminals letterbox around this.
const MAX_UI_WIDTH: u16 = 88;

pub fn draw(frame: &mut Frame, app: &mut App) {
    let theme = app.theme();
    let area = frame.area();

    // Full-bleed background chrome
    frame.render_widget(Block::default().style(theme.root_style()), area);

    // 1u margin, then cap/center so maximized windows stay portrait-shaped
    let content = center_max_width(inset(area, 1), MAX_UI_WIDTH);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5), // metrics + progress bars
            Constraint::Length(7), // agenda
            Constraint::Length(3), // form
            Constraint::Min(5),    // tasks
            Constraint::Length(1), // help
        ])
        .split(content);

    let date_label = Local::now().format("%d %b %Y").to_string();

    metrics::render(frame, chunks[0], app, &date_label);
    agenda::render(frame, chunks[1], app);
    form::render(frame, chunks[2], app);
    task_list::render(frame, chunks[3], app);
    help::render(frame, chunks[4], app);
}

pub fn inner(area: Rect) -> Rect {
    inset(area, 1)
}

fn inset(area: Rect, pad: u16) -> Rect {
    Rect {
        x: area.x.saturating_add(pad),
        y: area.y.saturating_add(pad),
        width: area.width.saturating_sub(pad.saturating_mul(2)),
        height: area.height.saturating_sub(pad.saturating_mul(2)),
    }
}

fn center_max_width(area: Rect, max_width: u16) -> Rect {
    if area.width <= max_width {
        return area;
    }
    let x = area.x + (area.width - max_width) / 2;
    Rect {
        x,
        y: area.y,
        width: max_width,
        height: area.height,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn center_max_width_letterboxes() {
        let area = Rect::new(0, 0, 200, 40);
        let content = center_max_width(area, 88);
        assert_eq!(content.width, 88);
        assert_eq!(content.x, 56); // (200 - 88) / 2
        assert_eq!(content.height, 40);
    }

    #[test]
    fn center_max_width_passthrough_when_narrow() {
        let area = Rect::new(1, 1, 60, 20);
        assert_eq!(center_max_width(area, 88), area);
    }
}
