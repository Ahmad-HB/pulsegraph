use chrono::{Datelike, Duration, NaiveDate};
use owo_colors::OwoColorize;
use tokenpulse_core::stats::{Streaks, Totals};
use tokenpulse_core::{Metric, Summary};

/// Map a value to a 0..=4 intensity bucket given the period max.
pub fn level(value: f64, max: f64) -> u8 {
    if max <= 0.0 || value <= 0.0 {
        return 0;
    }
    let frac = value / max;
    if frac > 0.75 {
        4
    } else if frac > 0.5 {
        3
    } else if frac > 0.25 {
        2
    } else {
        1
    }
}

fn cell(lvl: u8) -> String {
    // 5-step GitHub-style green ramp via truecolor; level 0 is a dim block.
    let (r, g, b) = match lvl {
        0 => (45, 51, 59),
        1 => (14, 68, 41),
        2 => (0, 109, 50),
        3 => (38, 166, 65),
        4 => (57, 211, 83),
        _ => (45, 51, 59),
    };
    "  ".on_truecolor(r, g, b).to_string()
}

fn metric_label(m: Metric) -> &'static str {
    match m {
        Metric::Cost => "cost ($)",
        Metric::Billable => "billable tokens",
        Metric::Output => "output tokens",
        Metric::Raw => "all tokens",
    }
}

fn fmt_value(m: Metric, v: f64) -> String {
    match m {
        Metric::Cost => format!("${v:.2}"),
        _ => format!("{}", v as u64),
    }
}

/// Render a full-year heatmap (today back 52 weeks) plus stat cards.
pub fn print_heatmap(
    summary: &Summary,
    st: &Streaks,
    tot: &Totals,
    metric: Metric,
    today: NaiveDate,
    unreadable: u64,
) {
    println!("\nTokenPulse — {}\n", metric_label(metric));

    // Build the grid: 53 columns (weeks) x 7 rows (Mon..Sun), ending at `today`.
    let weeks = 53;
    let start = today - Duration::days((weeks * 7 - 1) as i64);
    // Align start to Monday.
    let start = start - Duration::days(start.weekday().num_days_from_monday() as i64);

    let max = summary
        .days
        .values()
        .map(|d| d.metric(metric))
        .fold(0.0_f64, f64::max);

    for row in 0..7u32 {
        let mut line = String::new();
        for col in 0..weeks {
            let date = start + Duration::days((col * 7 + row) as i64);
            if date > today {
                line.push_str("  ");
                continue;
            }
            let v = summary.days.get(&date).map(|d| d.metric(metric)).unwrap_or(0.0);
            line.push_str(&cell(level(v, max)));
        }
        println!("{line}");
    }

    println!(
        "\n  Total {}   Best day {}   Avg/active-day {}   Active days {}",
        fmt_value(metric, tot.total),
        tot.best_day.map(|(_, v)| fmt_value(metric, v)).unwrap_or_else(|| "—".into()),
        fmt_value(metric, tot.avg_per_active_day),
        tot.active_days,
    );
    println!(
        "  Current streak {} days   Longest {} days",
        st.current, st.longest
    );
    if unreadable > 0 {
        println!("  ({unreadable} unreadable lines skipped)");
    }
    println!();
}

pub fn print_json(summary: &Summary, st: &Streaks, tot: &Totals, metric: Metric) {
    // Minimal, stable JSON for scripting. Days as date→metric value.
    let days: std::collections::BTreeMap<String, f64> = summary
        .days
        .iter()
        .map(|(d, s)| (d.to_string(), s.metric(metric)))
        .collect();
    let out = serde_json::json!({
        "metric": metric_label(metric),
        "total": tot.total,
        "best_day": tot.best_day.map(|(d, v)| serde_json::json!({"date": d.to_string(), "value": v})),
        "avg_per_active_day": tot.avg_per_active_day,
        "active_days": tot.active_days,
        "current_streak": st.current,
        "longest_streak": st.longest,
        "days": days,
    });
    println!("{}", serde_json::to_string_pretty(&out).unwrap());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn level_zero_for_zero_value() {
        assert_eq!(level(0.0, 100.0), 0);
    }

    #[test]
    fn level_scales_into_four_buckets() {
        // max=100 → quartile thresholds at 25/50/75.
        assert_eq!(level(10.0, 100.0), 1);
        assert_eq!(level(40.0, 100.0), 2);
        assert_eq!(level(60.0, 100.0), 3);
        assert_eq!(level(100.0, 100.0), 4);
    }

    #[test]
    fn level_zero_when_max_is_zero() {
        assert_eq!(level(0.0, 0.0), 0);
    }
}
