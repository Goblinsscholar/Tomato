use tauri::{AppHandle, Emitter};
use tauri_plugin_notification::NotificationExt;

/// Show a native OS notification via `tauri-plugin-notification`.
/// Cross-platform — uses Windows Toast, macOS Notification Center, or
/// Linux notification daemon depending on the platform.
/// Replaces the old PowerShell-based approach (antivirus-safe, no shell window).
fn show_notification(app_handle: &AppHandle, title: &str, body: &str) {
    if let Err(e) = app_handle
        .notification()
        .builder()
        .title(title)
        .body(body)
        .show()
    {
        crate::write_error_log("notification", &e);
    }
}

/// Emit event so the frontend can play sounds / react to timer completion.
/// Also triggers a native OS notification.
pub fn send_focus_complete(app_handle: &AppHandle, tag: &str) {
    show_notification(app_handle, "专注完成！", &format!("太棒了！\"{}\" 时段已完成。", tag));
    let _ = app_handle.emit("focus-complete", serde_json::json!({ "tag": tag }));
}

/// Emit event and show native notification for break completion.
pub fn send_break_complete(app_handle: &AppHandle) {
    show_notification(app_handle, "休息结束", "该继续专注了！");
    let _ = app_handle.emit("break-complete", serde_json::json!({}));
}

pub fn send_break_ready(app_handle: &AppHandle) {
    let _ = app_handle.emit("break-ready", serde_json::json!({}));
}
