use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::Weekday;
use serde::{Deserialize, Serialize};

use crate::db::db_path;

/// Runtime configuration loaded from a TOML file next to the DB (or `TKT_CONFIG`).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub goals: GoalsConfig,
    #[serde(default)]
    pub week: WeekConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalsConfig {
    /// Minutes below which the day bar is "low" (danger).
    #[serde(default = "default_day_ok")]
    pub day_ok_minutes: i64,
    /// Minutes at which the day bar turns green ("goal").
    #[serde(default = "default_day_goal")]
    pub day_goal_minutes: i64,
    /// Minutes at which the day bar is full (1.0). Display denominator.
    #[serde(default = "default_day_bar_max")]
    pub day_bar_max_minutes: i64,
    #[serde(default = "default_week_ok")]
    pub week_ok_minutes: i64,
    #[serde(default = "default_week_goal")]
    pub week_goal_minutes: i64,
    #[serde(default = "default_week_bar_max")]
    pub week_bar_max_minutes: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeekConfig {
    /// Week start weekday: mon, tue, wed, thu, fri, sat, sun.
    #[serde(default = "default_week_start")]
    pub start: String,
}

impl Default for GoalsConfig {
    fn default() -> Self {
        Self {
            day_ok_minutes: default_day_ok(),
            day_goal_minutes: default_day_goal(),
            day_bar_max_minutes: default_day_bar_max(),
            week_ok_minutes: default_week_ok(),
            week_goal_minutes: default_week_goal(),
            week_bar_max_minutes: default_week_bar_max(),
        }
    }
}

impl Default for WeekConfig {
    fn default() -> Self {
        Self {
            start: default_week_start(),
        }
    }
}

fn default_day_ok() -> i64 {
    4 * 60
}
fn default_day_goal() -> i64 {
    6 * 60
}
fn default_day_bar_max() -> i64 {
    8 * 60
}
fn default_week_ok() -> i64 {
    20 * 60
}
fn default_week_goal() -> i64 {
    30 * 60
}
fn default_week_bar_max() -> i64 {
    40 * 60
}
fn default_week_start() -> String {
    "thu".into()
}

impl Config {
    pub fn load_default() -> Result<Self> {
        let path = config_path()?;
        Self::load_or_create(&path)
    }

    pub fn load_or_create(path: &Path) -> Result<Self> {
        if path.exists() {
            let text = std::fs::read_to_string(path)
                .with_context(|| format!("read config {}", path.display()))?;
            let cfg: Config = toml::from_str(&text)
                .with_context(|| format!("parse config {}", path.display()))?;
            return Ok(cfg.sanitized());
        }
        let cfg = Config::default();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("create config dir {}", parent.display()))?;
        }
        let text = toml::to_string_pretty(&cfg).context("serialize default config")?;
        std::fs::write(path, text)
            .with_context(|| format!("write default config {}", path.display()))?;
        Ok(cfg)
    }

    fn sanitized(mut self) -> Self {
        let g = &mut self.goals;
        if g.day_ok_minutes < 0 {
            g.day_ok_minutes = default_day_ok();
        }
        if g.day_goal_minutes < g.day_ok_minutes {
            g.day_goal_minutes = g.day_ok_minutes;
        }
        if g.day_bar_max_minutes < g.day_goal_minutes {
            g.day_bar_max_minutes = g.day_goal_minutes;
        }
        if g.week_ok_minutes < 0 {
            g.week_ok_minutes = default_week_ok();
        }
        if g.week_goal_minutes < g.week_ok_minutes {
            g.week_goal_minutes = g.week_ok_minutes;
        }
        if g.week_bar_max_minutes < g.week_goal_minutes {
            g.week_bar_max_minutes = g.week_goal_minutes;
        }
        self
    }

    pub fn week_start(&self) -> Weekday {
        parse_weekday(&self.week.start).unwrap_or(Weekday::Thu)
    }
}

pub fn config_path() -> Result<PathBuf> {
    if let Ok(override_path) = std::env::var("TKT_CONFIG") {
        return Ok(PathBuf::from(override_path));
    }
    // Prefer alongside the DB so one folder holds state + config.
    let db = db_path()?;
    Ok(db
        .parent()
        .map(|p| p.join("config.toml"))
        .unwrap_or_else(|| PathBuf::from("config.toml")))
}

fn parse_weekday(s: &str) -> Option<Weekday> {
    match s.trim().to_ascii_lowercase().as_str() {
        "mon" | "monday" => Some(Weekday::Mon),
        "tue" | "tues" | "tuesday" => Some(Weekday::Tue),
        "wed" | "wednesday" => Some(Weekday::Wed),
        "thu" | "thur" | "thurs" | "thursday" => Some(Weekday::Thu),
        "fri" | "friday" => Some(Weekday::Fri),
        "sat" | "saturday" => Some(Weekday::Sat),
        "sun" | "sunday" => Some(Weekday::Sun),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_product_targets() {
        let cfg = Config::default();
        assert_eq!(cfg.goals.day_goal_minutes, 6 * 60);
        assert_eq!(cfg.goals.day_bar_max_minutes, 8 * 60);
        assert_eq!(cfg.goals.week_goal_minutes, 30 * 60);
        assert_eq!(cfg.goals.week_bar_max_minutes, 40 * 60);
        assert_eq!(cfg.week_start(), Weekday::Thu);
    }

    #[test]
    fn sanitizes_inverted_thresholds() {
        let raw = Config {
            goals: GoalsConfig {
                day_ok_minutes: 500,
                day_goal_minutes: 100,
                day_bar_max_minutes: 50,
                ..GoalsConfig::default()
            },
            ..Config::default()
        };
        let cfg = raw.sanitized();
        assert!(cfg.goals.day_goal_minutes >= cfg.goals.day_ok_minutes);
        assert!(cfg.goals.day_bar_max_minutes >= cfg.goals.day_goal_minutes);
    }
}
