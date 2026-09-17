use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use crate::app::{App, Focus, Mode};
use crate::theme::Theme;

pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let theme = app.theme();
    let line = match &app.mode {
        Mode::ConfirmDeleteTask { name, .. } | Mode::ConfirmDeleteAgenda { name, .. } => {
            Line::from(vec![
                Span::styled(" delete \"", theme.help_style()),
                Span::styled(name.clone(), Style::default().fg(theme.danger)),
                Span::styled("\"? ", theme.help_style()),
                key(&theme, "y"),
                Span::styled(" yes · ", theme.help_style()),
                key(&theme, "n"),
                Span::styled(" no", theme.help_style()),
            ])
        }
        Mode::ConfirmClearCompleted => Line::from(vec![
            Span::styled(" clear completed? ", theme.help_style()),
            key(&theme, "y"),
            Span::styled(" yes · ", theme.help_style()),
            key(&theme, "n"),
            Span::styled(" no", theme.help_style()),
        ]),
        Mode::Editing { .. } => hints(
            &theme,
            &[
                ("Tab", "next"),
                ("Enter", "save"),
                ("Esc", "cancel"),
                ("C^t", "theme"),
                ("C^p", "palette"),
                ("C^q", "quit"),
            ],
        ),
        Mode::EditingAgenda { .. } => hints(
            &theme,
            &[
                ("Enter", "save"),
                ("Esc", "cancel"),
                ("C^t", "theme"),
                ("C^p", "palette"),
                ("C^q", "quit"),
            ],
        ),
        Mode::Normal => match app.focus {
            Focus::AgendaInput => hints(
                &theme,
                &[
                    ("Enter", "add"),
                    ("Space", "toggle"),
                    ("e", "edit"),
                    ("d", "del"),
                    ("c", "clear"),
                    ("[/]", "day"),
                    ("C^t", "theme"),
                    ("C^p", "palette"),
                    ("C^q", "quit"),
                ],
            ),
            Focus::TaskName | Focus::Hours | Focus::Minutes => {
                let tab = if app.focus == Focus::TaskName && app.name_completion().is_some() {
                    ("Tab/→", "complete")
                } else {
                    ("Tab", "next")
                };
                hints(
                    &theme,
                    &[
                        tab,
                        ("Enter", "submit"),
                        ("[/]", "day"),
                        ("C^t", "theme"),
                        ("C^p", "palette"),
                        ("C^q", "quit"),
                    ],
                )
            }
            Focus::TaskList => hints(
                &theme,
                &[
                    ("e", "edit"),
                    ("d", "del"),
                    ("[/]", "day"),
                    ("C^t", "theme"),
                    ("C^p", "palette"),
                    ("C^q", "quit"),
                ],
            ),
        },
    };

    frame.render_widget(Paragraph::new(line).style(theme.help_style()), area);
}

fn key(theme: &Theme, label: &str) -> Span<'static> {
    Span::styled(label.to_string(), theme.key_style())
}

fn hints(theme: &Theme, pairs: &[(&str, &str)]) -> Line<'static> {
    let mut spans: Vec<Span> = vec![Span::raw(" ")];
    for (i, (k, label)) in pairs.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled(" · ", Style::default().fg(theme.muted)));
        }
        spans.push(Span::styled((*k).to_string(), theme.key_style()));
        spans.push(Span::styled(
            format!(" {label}"),
            Style::default().fg(theme.muted),
        ));
    }
    Line::from(spans)
}
