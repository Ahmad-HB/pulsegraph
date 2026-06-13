//! TUI application state and pure transitions. No rendering, no I/O here.

use std::collections::HashMap;

use chrono::{Datelike, Duration, NaiveDate};
use pulsegraph_core::aggregate::local_date;
use pulsegraph_core::{summarize, Filter, Metric, Pricing, Summary, TokenCounts, UsageEvent};

/// Time window shown by the heatmap.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Range {
    TwelveWeeks,
    ThirtyDays,
    Year,
}

impl Range {
    pub fn next(self) -> Range {
        match self {
            Range::TwelveWeeks => Range::ThirtyDays,
            Range::ThirtyDays => Range::Year,
            Range::Year => Range::TwelveWeeks,
        }
    }

    /// How many days back the window spans (grid is then Monday-aligned).
    pub fn days_back(self) -> i64 {
        match self {
            Range::TwelveWeeks => 12 * 7,
            Range::ThirtyDays => 30,
            Range::Year => 53 * 7,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Range::TwelveWeeks => "12 weeks",
            Range::ThirtyDays => "30 days",
            Range::Year => "full year",
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PickerKind {
    Project,
    Model,
}

/// An open selection overlay (project or model). Item 0 is always "All".
pub struct Picker {
    pub kind: PickerKind,
    pub items: Vec<String>,
    pub selected: usize,
}

/// One row of a day's per-project or per-model breakdown.
#[derive(Debug, Clone, PartialEq)]
pub struct Breakdown {
    pub label: String,
    pub value: f64,
}

pub struct App {
    events: Vec<UsageEvent>,
    pricing: Pricing,
    pub metric: Metric,
    pub filter: Filter,
    pub range: Range,
    pub cursor: NaiveDate,
    pub today: NaiveDate,
    pub view: Summary,
    pub projects: Vec<String>,
    pub models: Vec<String>,
    pub picker: Option<Picker>,
    pub should_quit: bool,
}

impl App {
    pub fn new(
        events: Vec<UsageEvent>,
        pricing: Pricing,
        metric: Metric,
        filter: Filter,
        today: NaiveDate,
    ) -> App {
        // First pass with no filter: the full project/model universe for the pickers.
        let universe = summarize(events.iter(), &pricing, &Filter::default());
        let mut projects: Vec<String> = universe.projects.keys().cloned().collect();
        projects.sort();
        let mut models: Vec<String> = universe.models.iter().cloned().collect();
        models.sort();
        let view = summarize(events.iter(), &pricing, &filter);
        App {
            events,
            pricing,
            metric,
            filter,
            // Default to the widest window so the heatmap fills the terminal;
            // `r` zooms in to 12 weeks / 30 days.
            range: Range::Year,
            cursor: today,
            today,
            view,
            projects,
            models,
            picker: None,
            should_quit: false,
        }
    }

    pub fn cycle_metric(&mut self) {
        self.metric = match self.metric {
            Metric::Cost => Metric::Billable,
            Metric::Billable => Metric::Output,
            Metric::Output => Metric::Raw,
            Metric::Raw => Metric::Cost,
        };
    }

    pub fn cycle_range(&mut self) {
        self.range = self.range.next();
        self.clamp_cursor();
    }

    /// Monday-aligned first day of the full (width-unclamped) grid.
    pub fn grid_start(&self) -> NaiveDate {
        let start = self.today - Duration::days(self.range.days_back() - 1);
        start - Duration::days(start.weekday().num_days_from_monday() as i64)
    }

    fn clamp_cursor(&mut self) {
        let gs = self.grid_start();
        self.cursor = self.cursor.clamp(gs, self.today);
    }

    /// Move the cursor by whole weeks and/or days, clamped to the grid and today.
    pub fn move_cursor(&mut self, dx_weeks: i64, dy_days: i64) {
        let cand = self.cursor + Duration::days(dx_weeks * 7 + dy_days);
        self.cursor = cand.clamp(self.grid_start(), self.today);
    }

    pub fn jump_today(&mut self) {
        self.cursor = self.today;
    }

    pub fn set_project(&mut self, project: Option<String>) {
        self.filter.project = project;
        self.recompute();
    }

    pub fn set_model(&mut self, model: Option<String>) {
        self.filter.model = model;
        self.recompute();
    }

    fn recompute(&mut self) {
        self.view = summarize(self.events.iter(), &self.pricing, &self.filter);
    }

    pub fn day_value(&self, date: NaiveDate) -> f64 {
        self.view.days.get(&date).map(|d| d.metric(self.metric)).unwrap_or(0.0)
    }

    pub fn max_value(&self) -> f64 {
        self.view.days.values().map(|d| d.metric(self.metric)).fold(0.0_f64, f64::max)
    }

    pub fn day_sessions(&self, date: NaiveDate) -> usize {
        self.view.days.get(&date).map(|d| d.sessions.len()).unwrap_or(0)
    }

    fn day_breakdown_by<F>(&self, date: NaiveDate, key: F) -> Vec<Breakdown>
    where
        F: Fn(&UsageEvent) -> &str,
    {
        let mut acc: HashMap<String, (TokenCounts, f64)> = HashMap::new();
        // Mirrors the project/model filtering in core's `summarize`: we re-derive the
        // breakdown straight from events for this day so it always matches the active filter.
        for e in &self.events {
            if local_date(e.timestamp) != date {
                continue;
            }
            if let Some(p) = &self.filter.project {
                if &e.project != p {
                    continue;
                }
            }
            if let Some(m) = &self.filter.model {
                if &e.model != m {
                    continue;
                }
            }
            let entry = acc.entry(key(e).to_string()).or_default();
            entry.0 += e.tokens;
            if let Some(c) = self.pricing.cost(&e.model, &e.tokens) {
                entry.1 += c;
            }
        }
        let mut out: Vec<Breakdown> = acc
            .into_iter()
            .map(|(label, (tokens, cost))| {
                let value = match self.metric {
                    Metric::Cost => cost,
                    _ => self.metric.value_tokens(&tokens) as f64,
                };
                Breakdown { label, value }
            })
            .collect();
        out.sort_by(|a, b| {
            b.value.partial_cmp(&a.value).unwrap_or(std::cmp::Ordering::Equal)
        });
        out
    }

    pub fn day_by_project(&self, date: NaiveDate) -> Vec<Breakdown> {
        self.day_breakdown_by(date, |e| &e.project)
    }

    pub fn day_by_model(&self, date: NaiveDate) -> Vec<Breakdown> {
        self.day_breakdown_by(date, |e| &e.model)
    }

    pub fn open_picker(&mut self, kind: PickerKind) {
        let mut items = vec!["All".to_string()];
        match kind {
            PickerKind::Project => items.extend(self.projects.iter().cloned()),
            PickerKind::Model => items.extend(self.models.iter().cloned()),
        }
        let current = match kind {
            PickerKind::Project => &self.filter.project,
            PickerKind::Model => &self.filter.model,
        };
        let selected = current
            .as_ref()
            .and_then(|c| items.iter().position(|i| i == c))
            .unwrap_or(0);
        self.picker = Some(Picker { kind, items, selected });
    }

    /// Apply the open picker's selection (item 0 = "All" = clear filter) and close it.
    pub fn apply_picker(&mut self) {
        if let Some(p) = self.picker.take() {
            let choice = if p.selected == 0 {
                None
            } else {
                Some(p.items[p.selected].clone())
            };
            match p.kind {
                PickerKind::Project => self.set_project(choice),
                PickerKind::Model => self.set_model(choice),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, TimeZone, Utc};
    use std::collections::HashMap;

    fn ev(day: &str, project: &str, model: &str, session: &str, output: u64) -> UsageEvent {
        let ts: DateTime<Utc> = Utc.with_ymd_and_hms(
            day[0..4].parse().unwrap(),
            day[5..7].parse().unwrap(),
            day[8..10].parse().unwrap(),
            12, 0, 0,
        ).unwrap();
        UsageEvent {
            source: "test".into(),
            timestamp: ts,
            model: model.into(),
            project: project.into(),
            session_id: session.into(),
            tokens: TokenCounts { input: 0, output, cache_write_5m: 0, cache_write_1h: 0, cache_read: 0 },
        }
    }

    fn pricing() -> Pricing {
        let mut t = HashMap::new();
        t.insert("m1".to_string(), pulsegraph_core::pricing::ModelPrice { input: 1.0, output: 1.0 });
        Pricing::from_map(t)
    }

    fn app_with(today: NaiveDate, events: Vec<UsageEvent>) -> App {
        App::new(events, pricing(), Metric::Output, Filter::default(), today)
    }

    #[test]
    fn metric_cycles_through_all_four() {
        let mut a = app_with(NaiveDate::from_ymd_opt(2026, 6, 14).unwrap(), vec![]);
        assert_eq!(a.metric, Metric::Output);
        a.cycle_metric();
        assert_eq!(a.metric, Metric::Raw);
        a.cycle_metric();
        assert_eq!(a.metric, Metric::Cost);
        a.cycle_metric();
        assert_eq!(a.metric, Metric::Billable);
        a.cycle_metric();
        assert_eq!(a.metric, Metric::Output);
    }

    #[test]
    fn range_cycles_and_wraps() {
        let mut a = app_with(NaiveDate::from_ymd_opt(2026, 6, 14).unwrap(), vec![]);
        assert_eq!(a.range, Range::Year); // widest by default
        a.cycle_range();
        assert_eq!(a.range, Range::TwelveWeeks);
        a.cycle_range();
        assert_eq!(a.range, Range::ThirtyDays);
        a.cycle_range();
        assert_eq!(a.range, Range::Year);
    }

    #[test]
    fn cursor_never_moves_past_today_or_before_grid() {
        let today = NaiveDate::from_ymd_opt(2026, 6, 14).unwrap();
        let mut a = app_with(today, vec![]);
        a.move_cursor(1, 0); // try to go a week into the future
        assert_eq!(a.cursor, today);
        // go far into the past; must clamp at the grid start
        a.move_cursor(-1000, 0);
        assert_eq!(a.cursor, a.grid_start());
    }

    #[test]
    fn filter_changes_recompute_the_view() {
        let today = NaiveDate::from_ymd_opt(2026, 6, 14).unwrap();
        let day = "2026-06-13";
        let events = vec![
            ev(day, "alpha", "m1", "s1", 100),
            ev(day, "beta", "m1", "s2", 40),
        ];
        let mut a = app_with(today, events);
        let date = NaiveDate::from_ymd_opt(2026, 6, 13).unwrap();
        assert_eq!(a.day_value(date), 140.0); // both projects
        a.set_project(Some("alpha".into()));
        assert_eq!(a.day_value(date), 100.0); // only alpha
        a.set_project(None);
        assert_eq!(a.day_value(date), 140.0); // cleared
    }

    #[test]
    fn day_breakdown_groups_and_sorts_desc() {
        let today = NaiveDate::from_ymd_opt(2026, 6, 14).unwrap();
        let day = "2026-06-13";
        let events = vec![
            ev(day, "alpha", "m1", "s1", 30),
            ev(day, "beta", "m1", "s2", 70),
            ev(day, "alpha", "m1", "s3", 20),
        ];
        let a = app_with(today, events);
        let date = NaiveDate::from_ymd_opt(2026, 6, 13).unwrap();
        let by_proj = a.day_by_project(date);
        assert_eq!(by_proj.len(), 2);
        assert_eq!(by_proj[0], Breakdown { label: "beta".into(), value: 70.0 });
        assert_eq!(by_proj[1], Breakdown { label: "alpha".into(), value: 50.0 });
    }

    #[test]
    fn picker_lists_all_first_and_applies_choice() {
        let today = NaiveDate::from_ymd_opt(2026, 6, 14).unwrap();
        let events = vec![ev("2026-06-13", "alpha", "m1", "s1", 10)];
        let mut a = app_with(today, events);
        a.open_picker(PickerKind::Project);
        let p = a.picker.as_ref().unwrap();
        assert_eq!(p.items[0], "All");
        assert!(p.items.contains(&"alpha".to_string()));
        // select "alpha" (index 1) and apply
        a.picker.as_mut().unwrap().selected = 1;
        a.apply_picker();
        assert_eq!(a.filter.project.as_deref(), Some("alpha"));
        assert!(a.picker.is_none());
    }

    #[test]
    fn cycle_range_clamps_a_cursor_that_falls_outside_the_new_window() {
        let today = NaiveDate::from_ymd_opt(2026, 6, 14).unwrap();
        let mut a = app_with(today, vec![]);
        a.range = Range::TwelveWeeks; // start in the 12-week window
        // Move the cursor far back while in the 12-week window.
        a.move_cursor(-8, 0); // 8 weeks back, still inside 12 weeks
        let far_back = a.cursor;
        assert!(far_back < today);
        // Switch to the 30-day window; the cursor must clamp into the smaller grid.
        a.cycle_range(); // -> ThirtyDays
        assert!(a.cursor >= a.grid_start());
        assert!(a.cursor <= today);
        assert!(a.cursor > far_back); // it actually moved forward into the window
    }
}
