use ratatui::style::Color;

use crate::config::GoalsConfig;
use crate::theme::Theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgressLevel {
    /// Below the "okay" threshold.
    Low,
    /// At least okay, not yet at goal.
    Okay,
    /// At or above the goal (bar may still be incomplete vs bar max).
    Goal,
}

impl ProgressLevel {
    pub fn for_day(minutes: i64, goals: &GoalsConfig) -> Self {
        level(minutes, goals.day_ok_minutes, goals.day_goal_minutes)
    }

    pub fn for_week(minutes: i64, goals: &GoalsConfig) -> Self {
        level(minutes, goals.week_ok_minutes, goals.week_goal_minutes)
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

/// Fraction of bar max, capped at 1.0 for bar fill.
pub fn progress_ratio(minutes: i64, bar_max: i64) -> f64 {
    if bar_max <= 0 {
        return 0.0;
    }
    (minutes as f64 / bar_max as f64).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn defaults() -> GoalsConfig {
        GoalsConfig::default()
    }

    #[test]
    fn day_thresholds_use_goal_not_bar_max() {
        let g = defaults();
        assert_eq!(ProgressLevel::for_day(0, &g), ProgressLevel::Low);
        assert_eq!(ProgressLevel::for_day(239, &g), ProgressLevel::Low); // 3h59
        assert_eq!(ProgressLevel::for_day(240, &g), ProgressLevel::Okay); // 4h
        assert_eq!(ProgressLevel::for_day(359, &g), ProgressLevel::Okay);
        assert_eq!(ProgressLevel::for_day(360, &g), ProgressLevel::Goal); // 6h green
        assert_eq!(ProgressLevel::for_day(480, &g), ProgressLevel::Goal); // 8h still goal
        assert_eq!(ProgressLevel::for_day(500, &g), ProgressLevel::Goal);
    }

    #[test]
    fn bar_fills_against_max_not_goal() {
        assert!((progress_ratio(6 * 60, 8 * 60) - 0.75).abs() < f64::EPSILON);
        assert!((progress_ratio(8 * 60, 8 * 60) - 1.0).abs() < f64::EPSILON);
        assert!((progress_ratio(9 * 60, 8 * 60) - 1.0).abs() < f64::EPSILON);
        assert!((progress_ratio(30 * 60, 40 * 60) - 0.75).abs() < f64::EPSILON);
        assert!((progress_ratio(40 * 60, 40 * 60) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn week_thresholds() {
        let g = defaults();
        assert_eq!(ProgressLevel::for_week(19 * 60, &g), ProgressLevel::Low);
        assert_eq!(ProgressLevel::for_week(20 * 60, &g), ProgressLevel::Okay);
        assert_eq!(ProgressLevel::for_week(30 * 60, &g), ProgressLevel::Goal);
    }
}
