use tauri::State;
use pulsegraph_core::{scan, cache::Cache};
use crate::state::AppState;
use crate::snapshot::{build_snapshot, Snapshot};

/// Re-scan transcripts (incremental, cache-backed) into in-memory events.
#[tauri::command]
pub fn refresh(state: State<AppState>) -> Result<(), String> {
    let Some(dir) = state.projects_dir.clone() else {
        return Err("Could not locate ~/.claude/projects".into());
    };
    let mut cache = Cache::open(&state.cache_db).map_err(|e| e.to_string())?;
    let result = scan(&dir, &mut cache).map_err(|e| e.to_string())?;
    *state.events.lock().unwrap() = result.events;
    *state.unreadable_lines.lock().unwrap() = result.unreadable_lines;
    *state.generated_at.lock().unwrap() = now_secs();
    Ok(())
}

/// Aggregate the in-memory events for the given filter + metric.
#[tauri::command]
pub fn get_snapshot(
    state: State<AppState>,
    project: Option<String>,
    model: Option<String>,
    metric: String,
) -> Snapshot {
    let events = state.events.lock().unwrap();
    let generated_at = *state.generated_at.lock().unwrap();
    let mut snap = build_snapshot(&events, &state.pricing, project, model, &metric, generated_at);
    snap.unreadable_lines = *state.unreadable_lines.lock().unwrap();
    snap
}

#[tauri::command]
pub fn quit(app: tauri::AppHandle) {
    app.exit(0);
}

fn now_secs() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}
