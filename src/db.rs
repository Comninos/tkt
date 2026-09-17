use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::{DateTime, Local, Weekday};
use directories::ProjectDirs;
use rusqlite::{params, Connection};

use crate::week::{day_bounds, format_duration_minutes, month_bounds, week_bounds};

#[derive(Debug, Clone)]
pub struct AgendaItem {
    pub id: i64,
    pub name: String,
    pub completed: bool,
}

#[derive(Debug, Clone)]
pub struct Task {
    pub id: i64,
    pub name: String,
    pub hours: i32,
    pub minutes: i32,
    pub logged_at: DateTime<Local>,
}

#[derive(Debug, Clone, Default)]
pub struct Metrics {
    pub today: String,
    pub week: String,
    pub month: String,
    pub week_task_count: i64,
    pub today_minutes: i64,
    pub week_minutes: i64,
}

pub struct Db {
    conn: Connection,
}

impl Db {
    pub fn open_default() -> Result<Self> {
        let path = db_path()?;
        Self::open(&path)
    }

    pub fn open(path: &Path) -> Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("create db dir {}", parent.display()))?;
        }
        let conn = Connection::open(path)
            .with_context(|| format!("open db {}", path.display()))?;
        let db = Self { conn };
        db.migrate()?;
        Ok(db)
    }

    fn migrate(&self) -> Result<()> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS agenda_items (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                created_at TEXT NOT NULL,
                completed_at TEXT
            );

            CREATE TABLE IF NOT EXISTS tasks (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                hours INTEGER NOT NULL CHECK (hours >= 0),
                minutes INTEGER NOT NULL CHECK (minutes >= 0 AND minutes < 60),
                logged_at TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_tasks_logged_at ON tasks(logged_at);
            CREATE INDEX IF NOT EXISTS idx_agenda_active ON agenda_items(completed_at);

            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            "#,
        )?;
        Ok(())
    }

    pub fn get_setting(&self, key: &str) -> Result<Option<String>> {
        use rusqlite::OptionalExtension;
        self.conn
            .query_row(
                "SELECT value FROM settings WHERE key = ?1",
                params![key],
                |row| row.get(0),
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        self.conn.execute(
            "INSERT INTO settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }

    /// All agenda items (open + completed), oldest first.
    pub fn list_agenda(&self) -> Result<Vec<AgendaItem>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, completed_at IS NOT NULL FROM agenda_items ORDER BY id ASC",
        )?;
        let rows = stmt
            .query_map([], |row| {
                Ok(AgendaItem {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    completed: row.get(2)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn add_agenda(&self, name: &str) -> Result<()> {
        let now = Local::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO agenda_items (name, created_at) VALUES (?1, ?2)",
            params![name, now],
        )?;
        Ok(())
    }

    pub fn update_agenda(&self, id: i64, name: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE agenda_items SET name = ?1 WHERE id = ?2",
            params![name, id],
        )?;
        Ok(())
    }

    pub fn delete_agenda(&self, id: i64) -> Result<()> {
        self.conn
            .execute("DELETE FROM agenda_items WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn complete_agenda(&self, id: i64) -> Result<()> {
        let now = Local::now().to_rfc3339();
        self.conn.execute(
            "UPDATE agenda_items SET completed_at = ?1 WHERE id = ?2 AND completed_at IS NULL",
            params![now, id],
        )?;
        Ok(())
    }

    pub fn uncomplete_agenda(&self, id: i64) -> Result<()> {
        self.conn.execute(
            "UPDATE agenda_items SET completed_at = NULL WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }

    pub fn clear_completed_agenda(&self) -> Result<usize> {
        let n = self
            .conn
            .execute("DELETE FROM agenda_items WHERE completed_at IS NOT NULL", [])?;
        Ok(n)
    }

    pub fn tasks_for_day(&self, day: DateTime<Local>) -> Result<Vec<Task>> {
        let (start, end) = day_bounds(day);
        self.tasks_between(start, end)
    }

    fn tasks_between(&self, start: DateTime<Local>, end: DateTime<Local>) -> Result<Vec<Task>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, name, hours, minutes, logged_at FROM tasks
             WHERE logged_at >= ?1 AND logged_at < ?2
             ORDER BY logged_at DESC, id DESC",
        )?;
        let rows = stmt
            .query_map(params![start.to_rfc3339(), end.to_rfc3339()], |row| {
                let logged_at: String = row.get(4)?;
                Ok(Task {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    hours: row.get(2)?,
                    minutes: row.get(3)?,
                    logged_at: parse_dt(&logged_at)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn add_task(
        &self,
        name: &str,
        hours: i32,
        minutes: i32,
        logged_at: DateTime<Local>,
    ) -> Result<()> {
        self.conn.execute(
            "INSERT INTO tasks (name, hours, minutes, logged_at) VALUES (?1, ?2, ?3, ?4)",
            params![name, hours, minutes, logged_at.to_rfc3339()],
        )?;
        Ok(())
    }

    pub fn update_task(&self, id: i64, name: &str, hours: i32, minutes: i32) -> Result<()> {
        self.conn.execute(
            "UPDATE tasks SET name = ?1, hours = ?2, minutes = ?3 WHERE id = ?4",
            params![name, hours, minutes, id],
        )?;
        Ok(())
    }

    pub fn delete_task(&self, id: i64) -> Result<()> {
        self.conn
            .execute("DELETE FROM tasks WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// Distinct task names, most recently used first.
    pub fn recent_task_names(&self, limit: usize) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT name FROM tasks
             GROUP BY name
             ORDER BY MAX(logged_at) DESC
             LIMIT ?1",
        )?;
        let rows = stmt
            .query_map(params![limit as i64], |row| row.get(0))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(rows)
    }

    pub fn metrics(&self, day: DateTime<Local>, week_start: Weekday) -> Result<Metrics> {
        let (day_start, day_end) = day_bounds(day);
        let (week_start_dt, week_end) = week_bounds(day, week_start);
        let (month_start, month_end) = month_bounds(day);

        let today_minutes = self.sum_minutes(day_start, day_end)?;
        let week_minutes = self.sum_minutes(week_start_dt, week_end)?;

        Ok(Metrics {
            today: format_duration_minutes(today_minutes),
            week: format_duration_minutes(week_minutes),
            month: format_duration_minutes(self.sum_minutes(month_start, month_end)?),
            week_task_count: self.count_tasks(week_start_dt, week_end)?,
            today_minutes,
            week_minutes,
        })
    }

    fn sum_minutes(&self, start: DateTime<Local>, end: DateTime<Local>) -> Result<i64> {
        let minutes: i64 = self.conn.query_row(
            "SELECT COALESCE(SUM(hours * 60 + minutes), 0) FROM tasks
             WHERE logged_at >= ?1 AND logged_at < ?2",
            params![start.to_rfc3339(), end.to_rfc3339()],
            |row| row.get(0),
        )?;
        Ok(minutes)
    }

    fn count_tasks(&self, start: DateTime<Local>, end: DateTime<Local>) -> Result<i64> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM tasks WHERE logged_at >= ?1 AND logged_at < ?2",
            params![start.to_rfc3339(), end.to_rfc3339()],
            |row| row.get(0),
        )?;
        Ok(count)
    }
}

pub fn db_path() -> Result<PathBuf> {
    if let Ok(override_path) = std::env::var("TKT_DB") {
        return Ok(PathBuf::from(override_path));
    }
    let dirs = ProjectDirs::from("dev", "tkt", "tkt")
        .context("could not resolve local data directory")?;
    Ok(dirs.data_local_dir().join("tkt.db"))
}

fn parse_dt(s: &str) -> std::result::Result<DateTime<Local>, rusqlite::Error> {
    DateTime::parse_from_rfc3339(s)
        .map(|dt| dt.with_timezone(&Local))
        .map_err(|e| {
            rusqlite::Error::FromSqlConversionFailure(
                4,
                rusqlite::types::Type::Text,
                Box::new(e),
            )
        })
}
