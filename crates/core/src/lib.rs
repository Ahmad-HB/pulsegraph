pub mod model;
pub mod pricing;
pub mod parse;
pub mod discovery;
pub mod aggregate;
pub mod stats;
pub mod cache;
pub mod scan;

pub use aggregate::{summarize, Filter, Summary};
pub use discovery::default_projects_dir;
pub use model::{Metric, TokenCounts, UsageEvent};
pub use pricing::Pricing;
pub use scan::{scan, ScanResult};
pub use stats::{streaks, totals, Streaks, Totals};

#[cfg(test)]
mod smoke {
    #[test]
    fn workspace_builds() {
        assert_eq!(2 + 2, 4);
    }
}
