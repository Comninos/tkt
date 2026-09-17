use anyhow::Result;
use chrono::{Duration, Local, NaiveDate};
use crossterm::event::{
    Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
};
use ratatui::layout::Rect;
use ratatui::widgets::ListState;

use crate::config::Config;
use crate::db::{AgendaItem, Db, Metrics, Task};
use crate::theme::{Theme, ThemeId};
use crate::week::day_anchor;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    AgendaInput,
    TaskName,
    Hours,
    Minutes,
    TaskList,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Editing { task_id: i64 },
    EditingAgenda { item_id: i64 },
    ConfirmDeleteTask { task_id: i64, name: String },
    ConfirmDeleteAgenda { item_id: i64, name: String },
    ConfirmClearCompleted,
}

#[derive(Debug, Default, Clone)]
pub struct LayoutRects {
    pub agenda: Rect,
    pub agenda_rows: Vec<Rect>,
    /// Absolute agenda indices matching `agenda_rows`.
    pub agenda_row_indices: Vec<usize>,
    pub agenda_input: Rect,
    pub form: Rect,
    pub name: Rect,
    pub hours: Rect,
    pub minutes: Rect,
    pub tasks: Rect,
    pub task_rows: Vec<Rect>,
    /// Absolute task indices matching `task_rows`.
    pub task_row_indices: Vec<usize>,
}

pub struct App {
    pub db: Db,
    pub config: Config,
    pub should_quit: bool,
    pub focus: Focus,
    pub mode: Mode,
    pub theme_id: ThemeId,
    /// 0 = today, negative = days in the past. Never positive.
    pub day_offset: i64,
    pub agenda_input: String,
    pub task_name: String,
    pub hours_input: String,
    pub minutes_input: String,
    pub status: String,
    pub agenda: Vec<AgendaItem>,
    pub tasks: Vec<Task>,
    pub name_suggestions: Vec<String>,
    pub metrics: Metrics,
    pub task_list_state: ListState,
    pub agenda_selected: usize,
    pub rects: LayoutRects,
}

impl App {
    pub fn new(db: Db, config: Config) -> Result<Self> {
        let theme_id = load_theme(&db)?;
        let mut app = Self {
            db,
            config,
            should_quit: false,
            focus: Focus::TaskName,
            mode: Mode::Normal,
            theme_id,
            day_offset: 0,
            agenda_input: String::new(),
            task_name: String::new(),
            hours_input: String::new(),
            minutes_input: String::new(),
            status: String::new(),
            agenda: Vec::new(),
            tasks: Vec::new(),
            name_suggestions: Vec::new(),
            metrics: Metrics::default(),
            task_list_state: ListState::default(),
            agenda_selected: 0,
            rects: LayoutRects::default(),
        };
        app.reload()?;
        Ok(app)
    }

    pub fn theme(&self) -> Theme {
        self.theme_id.theme()
    }

    pub fn selected_date(&self) -> NaiveDate {
        (Local::now() + Duration::days(self.day_offset)).date_naive()
    }

    pub fn selected_day(&self) -> chrono::DateTime<Local> {
        if self.day_offset == 0 {
            Local::now()
        } else {
            day_anchor(self.selected_date())
        }
    }

    pub fn is_viewing_today(&self) -> bool {
        self.day_offset == 0
    }

    pub fn name_completion(&self) -> Option<&str> {
        if self.focus != Focus::TaskName || self.task_name.is_empty() {
            return None;
        }
        let needle = self.task_name.to_ascii_lowercase();
        self.name_suggestions.iter().find_map(|name| {
            if name.to_ascii_lowercase().starts_with(&needle) && name.len() > self.task_name.len()
            {
                Some(name.as_str())
            } else {
                None
            }
        })
    }

    pub fn reload(&mut self) -> Result<()> {
        self.agenda = self.db.list_agenda()?;
        self.tasks = self.db.tasks_for_day(self.selected_day())?;
        self.metrics = self
            .db
            .metrics(self.selected_day(), self.config.week_start())?;
        self.name_suggestions = self.db.recent_task_names(80)?;
        if self.agenda.is_empty() {
            self.agenda_selected = 0;
        } else if self.agenda_selected >= self.agenda.len() {
            self.agenda_selected = self.agenda.len() - 1;
        }
        match self.task_list_state.selected() {
            Some(i) if i >= self.tasks.len() => {
                if self.tasks.is_empty() {
                    self.task_list_state.select(None);
                } else {
                    self.task_list_state.select(Some(self.tasks.len() - 1));
                }
            }
            None if !self.tasks.is_empty() => self.task_list_state.select(Some(0)),
            _ => {}
        }
        Ok(())
    }

    pub fn handle_event(&mut self, event: Event) -> Result<()> {
        match event {
            Event::Key(key) if key.kind == KeyEventKind::Press => self.handle_key(key)?,
            Event::Mouse(mouse) => self.handle_mouse(mouse)?,
            _ => {}
        }
        Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent) -> Result<()> {
        if self.handle_confirm_key(key)? {
            return Ok(());
        }

        if key.modifiers.contains(KeyModifiers::CONTROL) {
            match key.code {
                KeyCode::Char('c') | KeyCode::Char('q') => {
                    self.should_quit = true;
                    return Ok(());
                }
                KeyCode::Char('t') => {
                    self.toggle_light_dark()?;
                    return Ok(());
                }
                KeyCode::Char('p') => {
                    self.cycle_palette()?;
                    return Ok(());
                }
                _ => {}
            }
        }

        match key.code {
            KeyCode::Char('[') => {
                self.shift_day(-1)?;
                return Ok(());
            }
            KeyCode::Char(']') => {
                self.shift_day(1)?;
                return Ok(());
            }
            KeyCode::Esc => {
                if matches!(
                    self.mode,
                    Mode::Editing { .. } | Mode::EditingAgenda { .. }
                ) {
                    self.cancel_edit();
                } else if self.focus == Focus::TaskList {
                    self.focus = Focus::TaskName;
                }
            }
            KeyCode::Tab => {
                if self.try_accept_completion() {
                    // accepted
                } else {
                    self.focus_next();
                }
            }
            KeyCode::BackTab => self.focus_prev(),
            KeyCode::Enter => self.on_enter()?,
            KeyCode::Backspace => self.on_backspace(),
            KeyCode::Right if self.focus == Focus::TaskName => {
                let _ = self.try_accept_completion();
            }
            KeyCode::Up => self.on_up(),
            KeyCode::Down => self.on_down(),
            KeyCode::Char(' ')
                if self.focus == Focus::AgendaInput && self.agenda_input.is_empty() =>
            {
                self.toggle_selected_agenda()?;
            }
            KeyCode::Char(c)
                if self.focus == Focus::AgendaInput
                    && self.agenda_input.is_empty()
                    && matches!(self.mode, Mode::Normal) =>
            {
                match c {
                    'e' => self.begin_edit_agenda()?,
                    'd' => self.begin_delete_agenda(),
                    'c' => self.begin_clear_completed(),
                    _ => self.insert_char(c),
                }
            }
            KeyCode::Char(c) if self.focus == Focus::TaskList => match c {
                'e' => self.begin_edit()?,
                'd' => self.begin_delete(),
                't' => self.toggle_light_dark()?,
                'p' => self.cycle_palette()?,
                'q' => self.should_quit = true,
                _ => {}
            },
            KeyCode::Char(c) => self.insert_char(c),
            _ => {}
        }
        Ok(())
    }

    fn handle_confirm_key(&mut self, key: KeyEvent) -> Result<bool> {
        match &self.mode {
            Mode::ConfirmDeleteTask { task_id, .. } => {
                let id = *task_id;
                match key.code {
                    KeyCode::Char('y') | KeyCode::Char('Y') => {
                        self.db.delete_task(id)?;
                        self.mode = Mode::Normal;
                        self.status = "task deleted".into();
                        self.reload()?;
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                        self.mode = Mode::Normal;
                        self.status.clear();
                    }
                    _ => {}
                }
                Ok(true)
            }
            Mode::ConfirmDeleteAgenda { item_id, .. } => {
                let id = *item_id;
                match key.code {
                    KeyCode::Char('y') | KeyCode::Char('Y') => {
                        self.db.delete_agenda(id)?;
                        self.mode = Mode::Normal;
                        self.status = "agenda item deleted".into();
                        self.reload()?;
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                        self.mode = Mode::Normal;
                        self.status.clear();
                    }
                    _ => {}
                }
                Ok(true)
            }
            Mode::ConfirmClearCompleted => {
                match key.code {
                    KeyCode::Char('y') | KeyCode::Char('Y') => {
                        let n = self.db.clear_completed_agenda()?;
                        self.mode = Mode::Normal;
                        self.status = format!("cleared {n} completed");
                        self.reload()?;
                    }
                    KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                        self.mode = Mode::Normal;
                        self.status.clear();
                    }
                    _ => {}
                }
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn shift_day(&mut self, delta: i64) -> Result<()> {
        let next = self.day_offset + delta;
        if next > 0 {
            self.status = "already at today".into();
            return Ok(());
        }
        // Soft floor: ~10 years
        if next < -3650 {
            self.status = "day limit reached".into();
            return Ok(());
        }
        self.day_offset = next;
        if matches!(self.mode, Mode::Editing { .. }) {
            self.cancel_edit();
        }
        self.reload()?;
        if self.is_viewing_today() {
            self.status = "today".into();
        } else {
            self.status = self.selected_date().format("%d %b %Y").to_string();
        }
        Ok(())
    }

    fn try_accept_completion(&mut self) -> bool {
        if self.focus != Focus::TaskName {
            return false;
        }
        if let Some(full) = self.name_completion().map(str::to_string) {
            self.task_name = full;
            true
        } else {
            false
        }
    }

    fn toggle_light_dark(&mut self) -> Result<()> {
        self.theme_id = self.theme_id.toggle_light_dark();
        self.persist_theme()?;
        self.status = format!("theme: {}", self.theme_id.label());
        Ok(())
    }

    fn cycle_palette(&mut self) -> Result<()> {
        self.theme_id = self.theme_id.cycle_palette();
        self.persist_theme()?;
        self.status = format!("theme: {}", self.theme_id.label());
        Ok(())
    }

    fn persist_theme(&self) -> Result<()> {
        let (palette, variant) = self.theme_id.to_storage();
        self.db.set_setting("theme.palette", &palette)?;
        self.db.set_setting("theme.variant", &variant)?;
        Ok(())
    }

    fn handle_mouse(&mut self, mouse: MouseEvent) -> Result<()> {
        let col = mouse.column;
        let row = mouse.row;

        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) => {
                if point_in(self.rects.agenda_input, col, row) {
                    self.focus = Focus::AgendaInput;
                    return Ok(());
                }
                if point_in(self.rects.name, col, row) {
                    self.focus = Focus::TaskName;
                    return Ok(());
                }
                if point_in(self.rects.hours, col, row) {
                    self.focus = Focus::Hours;
                    return Ok(());
                }
                if point_in(self.rects.minutes, col, row) {
                    self.focus = Focus::Minutes;
                    return Ok(());
                }
                for (i, rect) in self.rects.agenda_rows.iter().enumerate() {
                    if point_in(*rect, col, row) {
                        let abs = self
                            .rects
                            .agenda_row_indices
                            .get(i)
                            .copied()
                            .unwrap_or(i);
                        self.agenda_selected = abs;
                        if col <= rect.x.saturating_add(3)
                            && let Some(item) = self.agenda.get(self.agenda_selected).cloned()
                        {
                            self.toggle_agenda_id(item.id, item.completed)?;
                        }
                        self.focus = Focus::AgendaInput;
                        return Ok(());
                    }
                }
                for (i, rect) in self.rects.task_rows.iter().enumerate() {
                    if point_in(*rect, col, row) {
                        let abs = self
                            .rects
                            .task_row_indices
                            .get(i)
                            .copied()
                            .unwrap_or(i);
                        self.task_list_state.select(Some(abs));
                        self.focus = Focus::TaskList;
                        return Ok(());
                    }
                }
                if point_in(self.rects.tasks, col, row) {
                    self.focus = Focus::TaskList;
                } else if point_in(self.rects.agenda, col, row) {
                    self.focus = Focus::AgendaInput;
                } else if point_in(self.rects.form, col, row) {
                    self.focus = Focus::TaskName;
                }
            }
            MouseEventKind::ScrollUp if self.focus == Focus::TaskList => self.on_up(),
            MouseEventKind::ScrollDown if self.focus == Focus::TaskList => self.on_down(),
            MouseEventKind::ScrollUp if self.focus == Focus::AgendaInput => self.on_up(),
            MouseEventKind::ScrollDown if self.focus == Focus::AgendaInput => self.on_down(),
            _ => {}
        }
        Ok(())
    }

    fn focus_next(&mut self) {
        self.focus = match self.focus {
            Focus::AgendaInput => Focus::TaskName,
            Focus::TaskName => Focus::Hours,
            Focus::Hours => Focus::Minutes,
            Focus::Minutes => Focus::TaskList,
            Focus::TaskList => Focus::AgendaInput,
        };
    }

    fn focus_prev(&mut self) {
        self.focus = match self.focus {
            Focus::AgendaInput => Focus::TaskList,
            Focus::TaskName => Focus::AgendaInput,
            Focus::Hours => Focus::TaskName,
            Focus::Minutes => Focus::Hours,
            Focus::TaskList => Focus::Minutes,
        };
    }

    fn insert_char(&mut self, c: char) {
        match self.focus {
            Focus::AgendaInput => self.agenda_input.push(c),
            Focus::TaskName => self.task_name.push(c),
            Focus::Hours if c.is_ascii_digit() && self.hours_input.len() < 2 => {
                self.hours_input.push(c);
            }
            Focus::Minutes if c.is_ascii_digit() && self.minutes_input.len() < 2 => {
                self.minutes_input.push(c);
            }
            _ => {}
        }
    }

    fn on_backspace(&mut self) {
        match self.focus {
            Focus::AgendaInput => {
                self.agenda_input.pop();
            }
            Focus::TaskName => {
                self.task_name.pop();
            }
            Focus::Hours => {
                self.hours_input.pop();
            }
            Focus::Minutes => {
                self.minutes_input.pop();
            }
            Focus::TaskList => {}
        }
    }

    fn on_enter(&mut self) -> Result<()> {
        match self.focus {
            Focus::AgendaInput => {
                let name = self.agenda_input.trim().to_string();
                if name.is_empty() {
                    return Ok(());
                }
                match self.mode {
                    Mode::EditingAgenda { item_id } => {
                        self.db.update_agenda(item_id, &name)?;
                        self.mode = Mode::Normal;
                        self.status = "agenda item updated".into();
                    }
                    _ => {
                        self.db.add_agenda(&name)?;
                        self.status = "agenda item added".into();
                    }
                }
                self.agenda_input.clear();
                self.reload()?;
            }
            Focus::TaskName => self.focus = Focus::Hours,
            Focus::Hours => self.focus = Focus::Minutes,
            Focus::Minutes => self.submit_task()?,
            Focus::TaskList => {}
        }
        Ok(())
    }

    fn submit_task(&mut self) -> Result<()> {
        let name = self.task_name.trim().to_string();
        if name.is_empty() {
            self.status = "task required".into();
            self.focus = Focus::TaskName;
            return Ok(());
        }
        let hours: i32 = if self.hours_input.is_empty() {
            0
        } else {
            match self.hours_input.parse() {
                Ok(h) if h >= 0 => h,
                _ => {
                    self.status = "invalid hours".into();
                    self.focus = Focus::Hours;
                    return Ok(());
                }
            }
        };
        let minutes: i32 = if self.minutes_input.is_empty() {
            0
        } else {
            match self.minutes_input.parse() {
                Ok(m) if (0..60).contains(&m) => m,
                _ => {
                    self.status = "minutes must be 0-59".into();
                    self.focus = Focus::Minutes;
                    return Ok(());
                }
            }
        };
        if hours == 0 && minutes == 0 {
            self.status = "duration must be > 0".into();
            return Ok(());
        }

        match self.mode {
            Mode::Editing { task_id } => {
                self.db.update_task(task_id, &name, hours, minutes)?;
                self.mode = Mode::Normal;
                self.status = "task updated".into();
            }
            Mode::Normal | Mode::EditingAgenda { .. } => {
                self.db
                    .add_task(&name, hours, minutes, self.selected_day())?;
                self.status = "task logged".into();
                self.mode = Mode::Normal;
            }
            Mode::ConfirmDeleteTask { .. }
            | Mode::ConfirmDeleteAgenda { .. }
            | Mode::ConfirmClearCompleted => {}
        }

        self.clear_form();
        self.focus = Focus::TaskName;
        self.reload()?;
        Ok(())
    }

    fn clear_form(&mut self) {
        self.task_name.clear();
        self.hours_input.clear();
        self.minutes_input.clear();
    }

    fn cancel_edit(&mut self) {
        match self.mode {
            Mode::Editing { .. } => {
                self.mode = Mode::Normal;
                self.clear_form();
                self.status = "edit cancelled".into();
                self.focus = Focus::TaskList;
            }
            Mode::EditingAgenda { .. } => {
                self.mode = Mode::Normal;
                self.agenda_input.clear();
                self.status = "edit cancelled".into();
                self.focus = Focus::AgendaInput;
            }
            _ => {}
        }
    }

    fn begin_edit(&mut self) -> Result<()> {
        let Some(idx) = self.task_list_state.selected() else {
            return Ok(());
        };
        let Some(task) = self.tasks.get(idx).cloned() else {
            return Ok(());
        };
        self.task_name = task.name;
        self.hours_input = task.hours.to_string();
        self.minutes_input = format!("{:02}", task.minutes);
        self.mode = Mode::Editing { task_id: task.id };
        self.focus = Focus::TaskName;
        self.status = format!("editing task #{}", task.id);
        Ok(())
    }

    fn begin_delete(&mut self) {
        let Some(idx) = self.task_list_state.selected() else {
            return;
        };
        let Some(task) = self.tasks.get(idx) else {
            return;
        };
        self.mode = Mode::ConfirmDeleteTask {
            task_id: task.id,
            name: task.name.clone(),
        };
        self.status = format!("delete \"{}\"? [y/N]", task.name);
    }

    fn begin_edit_agenda(&mut self) -> Result<()> {
        let Some(item) = self.agenda.get(self.agenda_selected).cloned() else {
            return Ok(());
        };
        self.agenda_input = item.name;
        self.mode = Mode::EditingAgenda {
            item_id: item.id,
        };
        self.status = format!("editing agenda #{}", item.id);
        Ok(())
    }

    fn begin_delete_agenda(&mut self) {
        let Some(item) = self.agenda.get(self.agenda_selected) else {
            return;
        };
        self.mode = Mode::ConfirmDeleteAgenda {
            item_id: item.id,
            name: item.name.clone(),
        };
        self.status = format!("delete \"{}\"? [y/N]", item.name);
    }

    fn begin_clear_completed(&mut self) {
        let n = self.agenda.iter().filter(|a| a.completed).count();
        if n == 0 {
            self.status = "no completed items".into();
            return;
        }
        self.mode = Mode::ConfirmClearCompleted;
        self.status = format!("clear {n} completed? [y/N]");
    }

    fn on_up(&mut self) {
        match self.focus {
            Focus::TaskList => {
                if self.tasks.is_empty() {
                    return;
                }
                let i = self.task_list_state.selected().unwrap_or(0);
                let next = if i == 0 { self.tasks.len() - 1 } else { i - 1 };
                self.task_list_state.select(Some(next));
            }
            Focus::AgendaInput => {
                if self.agenda.is_empty() {
                    return;
                }
                if self.agenda_selected == 0 {
                    self.agenda_selected = self.agenda.len() - 1;
                } else {
                    self.agenda_selected -= 1;
                }
            }
            _ => {}
        }
    }

    fn on_down(&mut self) {
        match self.focus {
            Focus::TaskList => {
                if self.tasks.is_empty() {
                    return;
                }
                let i = self.task_list_state.selected().unwrap_or(0);
                let next = (i + 1) % self.tasks.len();
                self.task_list_state.select(Some(next));
            }
            Focus::AgendaInput => {
                if self.agenda.is_empty() {
                    return;
                }
                self.agenda_selected = (self.agenda_selected + 1) % self.agenda.len();
            }
            _ => {}
        }
    }

    fn toggle_selected_agenda(&mut self) -> Result<()> {
        if let Some(item) = self.agenda.get(self.agenda_selected).cloned() {
            self.toggle_agenda_id(item.id, item.completed)?;
        }
        Ok(())
    }

    fn toggle_agenda_id(&mut self, id: i64, completed: bool) -> Result<()> {
        if completed {
            self.db.uncomplete_agenda(id)?;
            self.status = "agenda item reopened".into();
        } else {
            self.db.complete_agenda(id)?;
            self.status = "agenda item done".into();
        }
        self.reload()?;
        Ok(())
    }

    pub fn form_title(&self) -> &'static str {
        match self.mode {
            Mode::Editing { .. } => "log (editing · Esc cancel)",
            _ => "log",
        }
    }

    pub fn agenda_title(&self) -> &'static str {
        match self.mode {
            Mode::EditingAgenda { .. } => "agenda (editing · Esc cancel)",
            _ => "agenda",
        }
    }

    pub fn tasks_title(&self) -> String {
        if self.is_viewing_today() {
            "today".into()
        } else {
            self.selected_date().format("%d %b").to_string()
        }
    }
}

fn load_theme(db: &Db) -> Result<ThemeId> {
    let palette = db
        .get_setting("theme.palette")?
        .unwrap_or_else(|| "catppuccin".into());
    let variant = db
        .get_setting("theme.variant")?
        .unwrap_or_else(|| "dark".into());
    Ok(ThemeId::from_storage(&palette, &variant))
}

fn point_in(rect: Rect, col: u16, row: u16) -> bool {
    col >= rect.x
        && col < rect.x.saturating_add(rect.width)
        && row >= rect.y
        && row < rect.y.saturating_add(rect.height)
}
