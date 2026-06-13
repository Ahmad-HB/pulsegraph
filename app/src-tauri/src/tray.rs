use tauri::{
    menu::{Menu, MenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager, PhysicalPosition, Position, Runtime,
};

use std::sync::atomic::{AtomicU64, Ordering};

use crate::state::AppState;
use crate::snapshot::build_snapshot;

/// Update the menu-bar label to today's cost total.
pub fn update_tray_title<R: Runtime>(app: &AppHandle<R>) {
    let state = app.state::<AppState>();
    let events = state.events.lock().unwrap();
    let snap = build_snapshot(&events, &state.pricing, None, None, "cost", 0);
    let today = chrono::Local::now().date_naive().to_string();
    let today_total = snap
        .days
        .iter()
        .find(|d| d.date == today)
        .map(|d| d.value)
        .unwrap_or(0.0);
    if let Some(tray) = app.tray_by_id("main") {
        let _ = tray.set_title(Some(format!("${today_total:.2}")));
    }
}

/// Timestamps (ms since epoch) shared with the blur handler in lib.rs to tame
/// macOS menu-bar focus flapping.
pub static LAST_SHOW_MS: AtomicU64 = AtomicU64::new(0);
pub static LAST_BLUR_HIDE_MS: AtomicU64 = AtomicU64::new(0);

/// Blur events within this window of a show() are the transient focus-flap and
/// are ignored; a tray click within this window of a blur-hide is the close half
/// of a toggle and must not re-open.
const FLAP_GUARD_MS: u64 = 300;

pub fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

pub fn setup_tray<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let quit = MenuItem::with_id(app, "quit", "Quit PulseGraph", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&quit])?;

    TrayIconBuilder::with_id("main")
        .icon(app.default_window_icon().unwrap().clone())
        .icon_as_template(false) // show the full-color icon so it's unmistakable
        .title("PulseGraph") // visible text label in the menu bar
        .menu(&menu)
        .show_menu_on_left_click(false) // left click toggles the popover, not the menu
        .on_menu_event(|app, event| {
            if event.id.as_ref() == "quit" {
                app.exit(0);
            }
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                position,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(win) = app.get_webview_window("popover") {
                    let visible = win.is_visible().unwrap_or(false);
                    let since_blur =
                        now_ms().saturating_sub(LAST_BLUR_HIDE_MS.load(Ordering::Relaxed));
                    if visible {
                        let _ = win.hide();
                    } else if since_blur < FLAP_GUARD_MS {
                        // This same click already blurred+hid the popover — toggle-off; do nothing.
                    } else {
                        // Anchor the popover just under the menu-bar icon.
                        let x = (position.x as i32 - 460).max(8);
                        let y = (position.y as i32 + 8).max(8);
                        let _ = win.set_position(Position::Physical(PhysicalPosition { x, y }));
                        LAST_SHOW_MS.store(now_ms(), Ordering::Relaxed);
                        let _ = win.show();
                        let _ = win.set_focus();
                    }
                }
            }
        })
        .build(app)?;
    Ok(())
}
