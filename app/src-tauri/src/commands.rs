use tauri::State;
use tauri::Manager;
use pulsegraph_core::{scan, cache::Cache};
use crate::state::AppState;
use crate::snapshot::{build_snapshot, Snapshot};

/// Re-scan transcripts (incremental, cache-backed) into in-memory events,
/// then refresh the menu-bar title so it stays in sync with the popover.
#[tauri::command]
pub fn refresh(app: tauri::AppHandle, state: State<AppState>) -> Result<(), String> {
    let Some(dir) = state.projects_dir.clone() else {
        return Err("Could not locate ~/.claude/projects".into());
    };
    let mut cache = Cache::open(&state.cache_db).map_err(|e| e.to_string())?;
    let result = scan(&dir, &mut cache).map_err(|e| e.to_string())?;
    *state.events.lock().unwrap() = result.events;
    *state.unreadable_lines.lock().unwrap() = result.unreadable_lines;
    *state.generated_at.lock().unwrap() = now_secs();
    crate::tray::update_tray_title(&app);
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

/// Recognize a supported image type from its leading bytes (magic numbers),
/// returning its mime. `None` means "not a supported image" — the safeguard
/// that rejects a non-image file even if it was renamed to .png/.svg.
fn detect_image_mime(bytes: &[u8]) -> Option<&'static str> {
    if bytes.starts_with(&[0x89, b'P', b'N', b'G']) { Some("image/png") }
    else if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) { Some("image/jpeg") }
    else if bytes.starts_with(b"GIF8") { Some("image/gif") }
    else if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" { Some("image/webp") }
    else if looks_like_svg(bytes) { Some("image/svg+xml") }
    else { None }
}

/// SVG is text, not magic-byte tagged: accept a leading `<svg` (optionally
/// preceded by whitespace/BOM or an `<?xml …?>` prolog).
fn looks_like_svg(bytes: &[u8]) -> bool {
    let head = &bytes[..bytes.len().min(512)];
    let s = String::from_utf8_lossy(head);
    let trimmed = s.trim_start_matches(|c: char| c.is_whitespace() || c == '\u{feff}');
    trimmed.starts_with("<svg") || (trimmed.starts_with("<?xml") && s.contains("<svg"))
}

/// Mime for a file we've already accepted (defaults to png defensively).
fn sniff_mime(bytes: &[u8]) -> &'static str {
    detect_image_mime(bytes).unwrap_or("image/png")
}

fn avatar_path(app: &tauri::AppHandle) -> Result<std::path::PathBuf, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.join("avatar.bin"))
}

fn to_data_url(bytes: &[u8]) -> String {
    use base64::Engine;
    let b64 = base64::engine::general_purpose::STANDARD.encode(bytes);
    format!("data:{};base64,{}", sniff_mime(bytes), b64)
}

/// Open a native image picker, validate the chosen file really is an image,
/// store it, and return it as a data URL. Returns `Ok(None)` if the user
/// cancels. The picker runs Rust-side and temporarily drops the popover's
/// always-on-top + auto-hide so the OS panel is visible and the popover doesn't
/// vanish when it loses focus to the panel.
#[tauri::command]
pub async fn pick_avatar(app: tauri::AppHandle) -> Result<Option<String>, String> {
    use std::sync::atomic::Ordering;

    if let Some(win) = app.get_webview_window("popover") {
        let _ = win.set_always_on_top(false);
    }
    crate::tray::SUPPRESS_HIDE.store(true, Ordering::Relaxed);

    // Run the modal pick on a worker thread: a synchronous command would block
    // the main thread, which the native panel also needs — that deadlocks. The
    // async command keeps the main thread free to drive the panel.
    let app2 = app.clone();
    let picked = tauri::async_runtime::spawn_blocking(move || {
        use tauri_plugin_dialog::DialogExt;
        app2.dialog()
            .file()
            .add_filter("Images", &["png", "svg", "jpg", "jpeg", "gif", "webp"])
            .blocking_pick_file()
    })
    .await
    .map_err(|e| e.to_string())?;

    crate::tray::SUPPRESS_HIDE.store(false, Ordering::Relaxed);
    if let Some(win) = app.get_webview_window("popover") {
        let _ = win.set_always_on_top(true);
        let _ = win.set_focus();
    }

    let Some(file) = picked else { return Ok(None); };
    let path = file.into_path().map_err(|e| e.to_string())?;
    let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
    if detect_image_mime(&bytes).is_none() {
        return Err("That file isn't a supported image (PNG, SVG, JPEG, GIF, or WebP).".into());
    }
    let dest = avatar_path(&app)?;
    std::fs::write(&dest, &bytes).map_err(|e| e.to_string())?;
    Ok(Some(to_data_url(&bytes)))
}

/// Return the stored avatar as a data URL, or None if unset.
#[tauri::command]
pub fn get_avatar(app: tauri::AppHandle) -> Option<String> {
    let path = avatar_path(&app).ok()?;
    let bytes = std::fs::read(&path).ok()?;
    Some(to_data_url(&bytes))
}

/// Remove the stored avatar.
#[tauri::command]
pub fn clear_avatar(app: tauri::AppHandle) -> Result<(), String> {
    let path = avatar_path(&app)?;
    if path.exists() { std::fs::remove_file(&path).map_err(|e| e.to_string())?; }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{detect_image_mime, sniff_mime};
    #[test]
    fn sniffs_known_image_magic_bytes() {
        assert_eq!(sniff_mime(&[0x89, b'P', b'N', b'G', 0x0d]), "image/png");
        assert_eq!(sniff_mime(&[0xFF, 0xD8, 0xFF, 0xE0]), "image/jpeg");
        assert_eq!(sniff_mime(b"GIF89a"), "image/gif");
        assert_eq!(sniff_mime(b"RIFF\0\0\0\0WEBPVP8 "), "image/webp");
        assert_eq!(sniff_mime(b"nope"), "image/png");
    }
    #[test]
    fn detects_svg_and_rejects_non_images() {
        assert_eq!(detect_image_mime(b"<svg xmlns=\"x\"></svg>"), Some("image/svg+xml"));
        assert_eq!(detect_image_mime(b"  \n<?xml version=\"1.0\"?>\n<svg></svg>"), Some("image/svg+xml"));
        assert_eq!(detect_image_mime(&[0x89, b'P', b'N', b'G', 0x0d]), Some("image/png"));
        assert!(detect_image_mime(b"this is not an image").is_none());
    }
}
