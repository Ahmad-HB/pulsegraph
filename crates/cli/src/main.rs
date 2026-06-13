mod cache_path;
mod render;
mod tui;

use std::io::IsTerminal;
use clap::{Parser, ValueEnum};
use pulsegraph_core::{
    default_projects_dir, scan, streaks, summarize, totals, Filter, Metric, Pricing,
};
use tui::app::App;

#[derive(Parser, Debug)]
#[command(name = "pulsegraph", about = "Claude Code token-usage heatmap")]
struct Args {
    /// Which metric drives the heatmap and stats.
    #[arg(long, value_enum, default_value_t = MetricArg::Cost)]
    metric: MetricArg,
    /// Only count usage for this project (basename of its cwd).
    #[arg(long)]
    project: Option<String>,
    /// Only count usage for this model id.
    #[arg(long)]
    model: Option<String>,
    /// Emit JSON instead of the ANSI heatmap.
    #[arg(long)]
    json: bool,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum MetricArg {
    Cost,
    Billable,
    Output,
    Raw,
}

impl From<MetricArg> for Metric {
    fn from(m: MetricArg) -> Self {
        match m {
            MetricArg::Cost => Metric::Cost,
            MetricArg::Billable => Metric::Billable,
            MetricArg::Output => Metric::Output,
            MetricArg::Raw => Metric::Raw,
        }
    }
}

fn main() {
    let args = Args::parse();
    let metric: Metric = args.metric.into();

    let Some(projects_dir) = default_projects_dir() else {
        eprintln!("Could not locate ~/.claude/projects");
        std::process::exit(1);
    };
    if !projects_dir.exists() {
        println!("No Claude Code usage found yet ({}).", projects_dir.display());
        return;
    }

    let mut cache = match pulsegraph_core::cache::Cache::open(&cache_path::cache_db_path()) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("cache error: {e}");
            std::process::exit(1);
        }
    };

    let scan_result = match scan(&projects_dir, &mut cache) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("scan error: {e}");
            std::process::exit(1);
        }
    };

    let pricing = Pricing::bundled();
    let filter = Filter { project: args.project.clone(), model: args.model.clone() };
    let today = chrono::Local::now().date_naive();

    if args.json {
        let summary = summarize(scan_result.events.iter(), &pricing, &filter);
        let st = streaks(&summary, today);
        let tot = totals(&summary, metric);
        render::print_json(&summary, &st, &tot, metric);
        return;
    }

    // Non-interactive (piped/redirected) output keeps the static heatmap.
    if !std::io::stdout().is_terminal() {
        let summary = summarize(scan_result.events.iter(), &pricing, &filter);
        let st = streaks(&summary, today);
        let tot = totals(&summary, metric);
        render::print_heatmap(&summary, &st, &tot, metric, today, scan_result.unreadable_lines);
        return;
    }

    let app = App::new(scan_result.events, pricing, metric, filter, today);
    if let Err(e) = tui::run(app) {
        eprintln!("tui error: {e}");
        std::process::exit(1);
    }
}
