use chrono::{Duration, NaiveDate};
use crate::aggregate::Summary;
use crate::model::Metric;

#[derive(Debug, Clone, PartialEq)]
pub struct Streaks {
    pub current: u32,
    pub current_range: Option<(NaiveDate, NaiveDate)>,
    pub longest: u32,
    pub longest_range: Option<(NaiveDate, NaiveDate)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Totals {
    pub total: f64,
    pub best_day: Option<(NaiveDate, f64)>,
    pub avg_per_active_day: f64,
    pub active_days: u32,
}

/// A day is "active" if it has any recorded usage (it is present in `days`).
pub fn streaks(summary: &Summary, today: NaiveDate) -> Streaks {
    let active: Vec<NaiveDate> = summary.days.keys().copied().collect(); // sorted (BTreeMap)

    // Current streak: consecutive active days ending exactly at `today`.
    let mut current = 0u32;
    let mut current_start = today;
    if active.binary_search(&today).is_ok() {
        let mut cursor = today;
        while summary.days.contains_key(&cursor) {
            current += 1;
            current_start = cursor;
            cursor -= Duration::days(1);
        }
    }
    let current_range = if current > 0 { Some((current_start, today)) } else { None };

    // Longest streak: scan the sorted active days for the longest consecutive run.
    let mut longest = 0u32;
    let mut longest_range = None;
    let mut run_start: Option<NaiveDate> = None;
    let mut prev: Option<NaiveDate> = None;
    let mut run_len = 0u32;
    for &day in &active {
        match prev {
            Some(p) if day == p + Duration::days(1) => run_len += 1,
            _ => {
                run_len = 1;
                run_start = Some(day);
            }
        }
        if run_len > longest {
            longest = run_len;
            longest_range = Some((run_start.unwrap(), day));
        }
        prev = Some(day);
    }

    Streaks { current, current_range, longest, longest_range }
}

/// Total / best-day / average for the selected metric, over active days only.
pub fn totals(summary: &Summary, metric: Metric) -> Totals {
    let mut total = 0.0;
    let mut best_day: Option<(NaiveDate, f64)> = None;
    for (date, day) in &summary.days {
        let v = day.metric(metric);
        total += v;
        if best_day.map_or(true, |(_, bv)| v > bv) {
            best_day = Some((*date, v));
        }
    }
    let active_days = summary.days.len() as u32;
    let avg_per_active_day = if active_days > 0 { total / active_days as f64 } else { 0.0 };
    Totals { total, best_day, avg_per_active_day, active_days }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::aggregate::DayStats;
    use std::collections::BTreeMap;

    fn summary_with_days(dates: &[(&str, f64)]) -> Summary {
        let mut days: BTreeMap<NaiveDate, DayStats> = BTreeMap::new();
        for (d, cost) in dates {
            let date = NaiveDate::parse_from_str(d, "%Y-%m-%d").unwrap();
            let mut ds = DayStats::default();
            ds.cost = *cost;
            days.insert(date, ds);
        }
        Summary { days, ..Default::default() }
    }

    fn d(s: &str) -> NaiveDate {
        NaiveDate::parse_from_str(s, "%Y-%m-%d").unwrap()
    }

    #[test]
    fn current_streak_counts_back_from_today() {
        let s = summary_with_days(&[("2026-06-11", 1.0), ("2026-06-12", 1.0), ("2026-06-13", 1.0)]);
        let st = streaks(&s, d("2026-06-13"));
        assert_eq!(st.current, 3);
        assert_eq!(st.current_range, Some((d("2026-06-11"), d("2026-06-13"))));
    }

    #[test]
    fn current_streak_zero_when_today_inactive() {
        let s = summary_with_days(&[("2026-06-10", 1.0), ("2026-06-11", 1.0)]);
        let st = streaks(&s, d("2026-06-13"));
        assert_eq!(st.current, 0);
        assert_eq!(st.current_range, None);
    }

    #[test]
    fn longest_streak_found_across_gaps() {
        let s = summary_with_days(&[
            ("2026-06-01", 1.0), ("2026-06-02", 1.0), ("2026-06-03", 1.0),
            ("2026-06-10", 1.0),
        ]);
        let st = streaks(&s, d("2026-06-13"));
        assert_eq!(st.longest, 3);
        assert_eq!(st.longest_range, Some((d("2026-06-01"), d("2026-06-03"))));
    }

    #[test]
    fn totals_computes_best_day_and_average() {
        let s = summary_with_days(&[("2026-06-11", 2.0), ("2026-06-12", 4.0), ("2026-06-13", 6.0)]);
        let t = totals(&s, Metric::Cost);
        assert_eq!(t.total, 12.0);
        assert_eq!(t.best_day, Some((d("2026-06-13"), 6.0)));
        assert_eq!(t.active_days, 3);
        assert!((t.avg_per_active_day - 4.0).abs() < 1e-9);
    }

    #[test]
    fn totals_empty_is_zero() {
        let s = Summary::default();
        let t = totals(&s, Metric::Cost);
        assert_eq!(t.total, 0.0);
        assert_eq!(t.best_day, None);
        assert_eq!(t.avg_per_active_day, 0.0);
    }
}
