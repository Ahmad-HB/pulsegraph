use std::collections::{BTreeMap, HashMap, HashSet};
use chrono::{DateTime, Local, NaiveDate, Utc};
use crate::model::{Metric, TokenCounts, UsageEvent};
use crate::pricing::Pricing;

/// Per-day rollup of usage.
#[derive(Debug, Clone, Default)]
pub struct DayStats {
    pub tokens: TokenCounts,
    pub cost: f64,
    pub unpriced_events: u64,
    pub sessions: HashSet<String>,
}

impl DayStats {
    /// Value used for heatmap intensity / stat cards under the selected metric.
    pub fn metric(&self, m: Metric) -> f64 {
        match m {
            Metric::Cost => self.cost,
            _ => m.value_tokens(&self.tokens) as f64,
        }
    }
}

/// Filter applied while aggregating.
#[derive(Debug, Default, Clone)]
pub struct Filter {
    pub project: Option<String>,
    pub model: Option<String>,
}

/// Result of aggregating a set of events.
#[derive(Debug, Clone, Default)]
pub struct Summary {
    pub days: BTreeMap<NaiveDate, DayStats>,
    /// Per-project totals (overall, after filtering) → (tokens, cost).
    pub projects: HashMap<String, (TokenCounts, f64)>,
    /// Distinct model ids seen (after filtering), for the model dropdown.
    pub models: HashSet<String>,
}

/// Local calendar day for a UTC timestamp.
pub fn local_date(ts: DateTime<Utc>) -> NaiveDate {
    ts.with_timezone(&Local).date_naive()
}

/// Aggregate events into a Summary, applying the filter. `cost` per day sums the
/// per-event cost from the pricing table; events with an unknown model add to
/// `unpriced_events` instead.
pub fn summarize<'a>(
    events: impl Iterator<Item = &'a UsageEvent>,
    pricing: &Pricing,
    filter: &Filter,
) -> Summary {
    let mut s = Summary::default();
    for e in events {
        if let Some(p) = &filter.project {
            if &e.project != p {
                continue;
            }
        }
        if let Some(m) = &filter.model {
            if &e.model != m {
                continue;
            }
        }

        s.models.insert(e.model.clone());

        let date = local_date(e.timestamp);
        let day = s.days.entry(date).or_default();
        day.tokens += e.tokens;
        day.sessions.insert(e.session_id.clone());

        let cost = pricing.cost(&e.model, &e.tokens);
        match cost {
            Some(c) => day.cost += c,
            None => day.unpriced_events += 1,
        }

        let entry = s.projects.entry(e.project.clone()).or_insert((TokenCounts::default(), 0.0));
        entry.0 += e.tokens;
        if let Some(c) = cost {
            entry.1 += c;
        }
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn ev(day_utc: &str, project: &str, model: &str, session: &str, input: u64) -> UsageEvent {
        UsageEvent {
            source: "claude-code".into(),
            timestamp: DateTime::parse_from_rfc3339(day_utc).unwrap().with_timezone(&Utc),
            model: model.into(),
            project: project.into(),
            session_id: session.into(),
            tokens: TokenCounts { input, output: 0, cache_write_5m: 0, cache_write_1h: 0, cache_read: 0 },
        }
    }

    fn pricing() -> Pricing {
        let mut t = std::collections::HashMap::new();
        t.insert("claude-opus-4-8".to_string(), crate::pricing::ModelPrice { input: 5.0, output: 25.0 });
        Pricing::from_map(t)
    }

    #[test]
    fn buckets_by_local_day_and_counts_sessions() {
        let evs = vec![
            ev("2026-06-13T10:00:00Z", "API", "claude-opus-4-8", "s1", 1_000_000),
            ev("2026-06-13T12:00:00Z", "API", "claude-opus-4-8", "s1", 1_000_000),
            ev("2026-06-13T12:00:00Z", "API", "claude-opus-4-8", "s2", 1_000_000),
        ];
        let s = summarize(evs.iter(), &pricing(), &Filter::default());
        // All same UTC day; assume test host is not 12h+ off UTC such that these split.
        let day = local_date(evs[0].timestamp);
        let d = &s.days[&day];
        assert_eq!(d.tokens.input, 3_000_000);
        assert_eq!(d.sessions.len(), 2);
        assert!((d.cost - 15.0).abs() < 1e-9); // 3M input * $5/M
        assert_eq!(d.unpriced_events, 0);
    }

    #[test]
    fn unknown_model_counts_as_unpriced() {
        let evs = vec![ev("2026-06-13T10:00:00Z", "API", "mystery", "s1", 1_000_000)];
        let s = summarize(evs.iter(), &pricing(), &Filter::default());
        let day = local_date(evs[0].timestamp);
        assert_eq!(s.days[&day].unpriced_events, 1);
        assert_eq!(s.days[&day].cost, 0.0);
    }

    #[test]
    fn project_filter_excludes_others() {
        let evs = vec![
            ev("2026-06-13T10:00:00Z", "API", "claude-opus-4-8", "s1", 1_000_000),
            ev("2026-06-13T10:00:00Z", "Vault", "claude-opus-4-8", "s2", 1_000_000),
        ];
        let filter = Filter { project: Some("API".into()), model: None };
        let s = summarize(evs.iter(), &pricing(), &filter);
        let day = local_date(evs[0].timestamp);
        assert_eq!(s.days[&day].tokens.input, 1_000_000);
        assert!(s.projects.contains_key("API"));
        assert!(!s.projects.contains_key("Vault"));
    }

    #[test]
    fn collects_models_and_project_totals() {
        let evs = vec![
            ev("2026-06-13T10:00:00Z", "API", "claude-opus-4-8", "s1", 1_000_000),
            ev("2026-06-12T10:00:00Z", "API", "claude-haiku-4-5", "s2", 2_000_000),
        ];
        let s = summarize(evs.iter(), &pricing(), &Filter::default());
        assert!(s.models.contains("claude-opus-4-8"));
        assert!(s.models.contains("claude-haiku-4-5"));
        assert_eq!(s.projects["API"].0.input, 3_000_000);
    }
}
