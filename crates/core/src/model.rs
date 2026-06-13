use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct TokenCounts {
    pub input: u64,
    pub output: u64,
    pub cache_write_5m: u64,
    pub cache_write_1h: u64,
    pub cache_read: u64,
}

impl TokenCounts {
    pub fn cache_write(&self) -> u64 {
        self.cache_write_5m + self.cache_write_1h
    }
    pub fn billable(&self) -> u64 {
        self.input + self.output + self.cache_write()
    }
    pub fn raw(&self) -> u64 {
        self.input + self.output + self.cache_write() + self.cache_read
    }
}

impl std::ops::Add for TokenCounts {
    type Output = TokenCounts;
    fn add(self, o: TokenCounts) -> TokenCounts {
        TokenCounts {
            input: self.input + o.input,
            output: self.output + o.output,
            cache_write_5m: self.cache_write_5m + o.cache_write_5m,
            cache_write_1h: self.cache_write_1h + o.cache_write_1h,
            cache_read: self.cache_read + o.cache_read,
        }
    }
}

impl std::ops::AddAssign for TokenCounts {
    fn add_assign(&mut self, o: TokenCounts) {
        *self = *self + o;
    }
}

/// A single usage record, source-agnostic so non-Claude-Code parsers can emit it later.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageEvent {
    pub source: String,
    pub timestamp: DateTime<Utc>,
    pub model: String,
    pub project: String,
    pub session_id: String,
    pub tokens: TokenCounts,
}

/// Which quantity the heatmap/stat cards display.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Metric {
    Cost,
    Billable,
    Output,
    Raw,
}

impl Metric {
    /// Token count for the non-Cost metrics. (Cost is computed via the pricing table.)
    pub fn value_tokens(&self, t: &TokenCounts) -> u64 {
        match self {
            Metric::Billable => t.billable(),
            Metric::Output => t.output,
            Metric::Raw => t.raw(),
            Metric::Cost => 0, // Cost is not a token count; see aggregate::DayStats::metric
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metrics_compute_correctly() {
        let t = TokenCounts { input: 100, output: 50, cache_write_5m: 10, cache_write_1h: 20, cache_read: 1000 };
        assert_eq!(t.cache_write(), 30);
        assert_eq!(t.billable(), 100 + 50 + 30);
        assert_eq!(t.raw(), 100 + 50 + 30 + 1000);
    }

    #[test]
    fn add_sums_fields() {
        let a = TokenCounts { input: 1, output: 2, cache_write_5m: 3, cache_write_1h: 4, cache_read: 5 };
        let b = TokenCounts { input: 10, output: 20, cache_write_5m: 30, cache_write_1h: 40, cache_read: 50 };
        let mut c = a;
        c += b;
        assert_eq!(c, TokenCounts { input: 11, output: 22, cache_write_5m: 33, cache_write_1h: 44, cache_read: 55 });
    }

    #[test]
    fn metric_value_selects_field() {
        let t = TokenCounts { input: 100, output: 50, cache_write_5m: 10, cache_write_1h: 20, cache_read: 1000 };
        assert_eq!(Metric::Billable.value_tokens(&t), 180);
        assert_eq!(Metric::Output.value_tokens(&t), 50);
        assert_eq!(Metric::Raw.value_tokens(&t), 1180);
    }
}
