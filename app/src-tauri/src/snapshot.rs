use serde::Serialize;
use pulsegraph_core::{summarize, streaks, totals, Filter, Metric, Pricing, UsageEvent};

#[derive(Serialize, Clone)]
pub struct DayValue {
    pub date: String,
    pub value: f64,
}

#[derive(Serialize, Clone)]
pub struct Snapshot {
    pub days: Vec<DayValue>,
    pub total: f64,
    pub best_day: Option<DayValue>,
    pub avg_per_active_day: f64,
    pub active_days: u32,
    pub current_streak: u32,
    pub longest_streak: u32,
    pub projects: Vec<String>,
    pub models: Vec<String>,
    pub generated_at: i64,
    pub unreadable_lines: u64,
}

fn parse_metric(s: &str) -> Metric {
    match s {
        "billable" => Metric::Billable,
        "output" => Metric::Output,
        "raw" => Metric::Raw,
        _ => Metric::Cost,
    }
}

/// Aggregate in-memory events into a serializable Snapshot for the given filter+metric.
pub fn build_snapshot(
    events: &[UsageEvent],
    pricing: &Pricing,
    project: Option<String>,
    model: Option<String>,
    metric_str: &str,
    generated_at: i64,
) -> Snapshot {
    let metric = parse_metric(metric_str);
    let filter = Filter { project, model };
    let summary = summarize(events.iter(), pricing, &filter);

    let days: Vec<DayValue> = summary
        .days
        .iter()
        .map(|(date, day)| DayValue { date: date.to_string(), value: day.metric(metric) })
        .collect();

    // Filter dropdowns should list ALL projects/models, unfiltered.
    let all = summarize(events.iter(), pricing, &Filter::default());
    let mut projects: Vec<String> = all.projects.keys().cloned().collect();
    projects.sort();
    let mut models: Vec<String> = all.models.iter().cloned().collect();
    models.sort();

    let today = chrono::Local::now().date_naive();
    let st = streaks(&summary, today);
    let tot = totals(&summary, metric);

    Snapshot {
        days,
        total: tot.total,
        best_day: tot.best_day.map(|(d, v)| DayValue { date: d.to_string(), value: v }),
        avg_per_active_day: tot.avg_per_active_day,
        active_days: tot.active_days,
        current_streak: st.current,
        longest_streak: st.longest,
        projects,
        models,
        generated_at,
        unreadable_lines: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Utc};
    use pulsegraph_core::TokenCounts;

    fn ev(ts: &str, project: &str, model: &str, input: u64) -> UsageEvent {
        UsageEvent {
            source: "claude-code".into(),
            timestamp: DateTime::parse_from_rfc3339(ts).unwrap().with_timezone(&Utc),
            model: model.into(),
            project: project.into(),
            session_id: "s".into(),
            tokens: TokenCounts { input, output: 0, cache_write_5m: 0, cache_write_1h: 0, cache_read: 0 },
        }
    }

    fn pricing() -> Pricing {
        let mut t = std::collections::HashMap::new();
        t.insert("claude-opus-4-8".to_string(), pulsegraph_core::pricing::ModelPrice { input: 5.0, output: 25.0 });
        Pricing::from_map(t)
    }

    #[test]
    fn build_snapshot_returns_wellformed_data() {
        let events = vec![
            ev("2026-06-13T10:00:00Z", "API", "claude-opus-4-8", 1_000_000),
            ev("2026-06-12T10:00:00Z", "Vault", "claude-opus-4-8", 2_000_000),
        ];
        let snap = build_snapshot(&events, &pricing(), None, None, "cost", 1234);
        assert_eq!(snap.generated_at, 1234);
        assert_eq!(snap.active_days, 2);
        assert!(snap.models.contains(&"claude-opus-4-8".to_string()));
        assert!(snap.projects.contains(&"API".to_string()));
        assert!(snap.projects.contains(&"Vault".to_string()));
        // 1M + 2M input * $5/M = $15 total
        assert!((snap.total - 15.0).abs() < 1e-9);
        assert_eq!(snap.days.len(), 2);
    }

    #[test]
    fn project_filter_narrows_results() {
        let events = vec![
            ev("2026-06-13T10:00:00Z", "API", "claude-opus-4-8", 1_000_000),
            ev("2026-06-13T10:00:00Z", "Vault", "claude-opus-4-8", 1_000_000),
        ];
        let snap = build_snapshot(&events, &pricing(), Some("API".into()), None, "cost", 0);
        assert!((snap.total - 5.0).abs() < 1e-9);
    }
}
