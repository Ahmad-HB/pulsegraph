use tokenpulse_core::{Metric, Summary};
use tokenpulse_core::stats::{Streaks, Totals};
use chrono::NaiveDate;

pub fn print_json(_s: &Summary, _st: &Streaks, _t: &Totals, _m: Metric) {
    println!("{{}}");
}

pub fn print_heatmap(
    _s: &Summary,
    _st: &Streaks,
    _t: &Totals,
    _m: Metric,
    _today: NaiveDate,
    _unreadable: u64,
) {
    println!("(heatmap)");
}
