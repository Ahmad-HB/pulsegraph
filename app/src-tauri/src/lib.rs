mod commands;
mod snapshot;
mod state;
mod tray;

use state::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init()) // keep whatever plugins the scaffold added
        .plugin(tauri_plugin_store::Builder::new().build())
        .manage(AppState::new())
        .setup(|app| {
            // Menu-bar app: no dock icon, stays resident as a tray-only accessory.
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            // Prime data once at startup (ignore errors; UI shows empty/stale).
            let handle = app.handle().clone();
            let state: tauri::State<AppState> = handle.state();
            let _ = commands::refresh(state);
            tray::setup_tray(app.handle())?;
            // Show today's total on the tray label.
            tray::update_tray_title(app.handle());

            // Frosted-glass vibrancy + rounded corners on the popover (macOS).
            #[cfg(target_os = "macos")]
            {
                use window_vibrancy::{apply_vibrancy, NSVisualEffectMaterial, NSVisualEffectState};
                if let Some(win) = app.get_webview_window("popover") {
                    let _ = apply_vibrancy(
                        &win,
                        NSVisualEffectMaterial::HudWindow,
                        Some(NSVisualEffectState::Active),
                        Some(16.0),
                    );
                }
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            use std::sync::atomic::Ordering;
            if let tauri::WindowEvent::Focused(false) = event {
                if window.label() == "popover" {
                    let since_show =
                        tray::now_ms().saturating_sub(tray::LAST_SHOW_MS.load(Ordering::Relaxed));
                    if since_show > 300 {
                        let _ = window.hide();
                        tray::LAST_BLUR_HIDE_MS.store(tray::now_ms(), Ordering::Relaxed);
                    }
                }
            }
        })
        .invoke_handler(tauri::generate_handler![commands::refresh, commands::get_snapshot, commands::quit])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
