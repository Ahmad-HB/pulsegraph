mod commands;
mod snapshot;
mod state;

use state::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init()) // keep whatever plugins the scaffold added
        .manage(AppState::new())
        .setup(|app| {
            // Prime data once at startup (ignore errors; UI shows empty/stale).
            let handle = app.handle().clone();
            let state: tauri::State<AppState> = handle.state();
            let _ = commands::refresh(state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![commands::refresh, commands::get_snapshot])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
