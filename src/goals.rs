use ratatui::style::Color;

use crate::theme::Theme;

pub const DAY_GOAL_MINUTES: i64 = 6 * 60;
pub const DAY_OK_MINUTES: i64 = 4 * 60;
pub const WEEK_GOAL_MINUTES: i64 = 30 * 60;
pub const WEEK_OK_MINUTES: i64 = 20 * 60;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgressLevel {
    /// Below the "okay" threshold.
    Low,
    /// At least okay, not yet at goal.
    Okay,
    /// At or above the goal.
    Goal,
}

impl ProgressLevel {
    pub fn for_day(minutes: i64) -> Self {
        level(minutes, DAY_OK_MINUTES, DAY_GOAL_MINUTES)
    }

    pub fn for_week(minutes: i64) -> Self {
        level(minutes, WEEK_OK_MINUTES, WEEK_GOAL_MINUTES)
    }

    pub fn color(self, theme: &Theme) -> Color {
        match self {
            ProgressLevel::Low => theme.danger,
            ProgressLevel::Okay => theme.warn,
            ProgressLevel::Goal => theme.good,
        }
    }
}

fn level(minutes: i64, okay: i64, goal: i64) -> ProgressLevel {
    if minutes >= goal {
        ProgressLevel::Goal
    } else if minutes >= okay {
        ProgressLevel::Okay
    } else {
        ProgressLevel::Low
    }
}

/// Fraction of goal, capped at 1.0 for bar fill.
pub fn progress_ratio(minutes: i64, goal: i64) -> f64 {
    if goal <= 0 {
        return 0.0;
    }
    (minutes as f64 / goal as f64).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn day_thresholds() {
        assert_eq!(ProgressLevel::for_day(0), ProgressLevel::Low);
        assert_eq!(ProgressLevel::for_day(239), ProgressLevel::Low); // 3h59
        assert_eq!(ProgressLevel::for_day(240), ProgressLevel::Okay); // 4h
        assert_eq!(ProgressLevel::for_day(359), ProgressLevel::Okay);
        assert_eq!(ProgressLevel::for_day(360), ProgressLevel::Goal); // 6h
        assert_eq!(ProgressLevel::for_day(500), ProgressLevel::Goal);
    }

    #[test]
    fn week_thresholds() {
        assert_eq!(ProgressLevel::for_week(19 * 60), ProgressLevel::Low);
        assert_eq!(ProgressLevel::for_week(20 * 60), ProgressLevel::Okay);
        assert_eq!(ProgressLevel::for_week(30 * 60), ProgressLevel::Goal);
    }
}
