use std::sync::Mutex;
use std::path::PathBuf;
use pulsegraph_core::{Pricing, UsageEvent};

pub struct AppState {
    pub events: Mutex<Vec<UsageEvent>>,
    pub generated_at: Mutex<i64>,
    pub unreadable_lines: Mutex<u64>,
    pub pricing: Pricing,
    pub projects_dir: Option<PathBuf>,
    pub cache_db: PathBuf,
}

impl AppState {
    pub fn new() -> Self {
        let cache_db = directories::ProjectDirs::from("dev", "pulsegraph", "PulseGraph")
            .map(|d| d.cache_dir().join("cache.sqlite"))
            .unwrap_or_else(|| PathBuf::from("pulsegraph-cache.sqlite"));
        AppState {
            events: Mutex::new(Vec::new()),
            generated_at: Mutex::new(0),
            unreadable_lines: Mutex::new(0),
            pricing: Pricing::bundled(),
            projects_dir: pulsegraph_core::default_projects_dir(),
            cache_db,
        }
    }
}
