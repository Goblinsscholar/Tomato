use tauri::{AppHandle, Manager, PhysicalPosition, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

const WIDGET_LABEL: &str = "widget";
const WIDGET_WIDTH: f64 = 210.0;
const WIDGET_HEIGHT: f64 = 90.0;
const PADDING: f64 = 16.0;

/// Ensure the widget window exists, creating it at top-right if needed.
/// Returns `None` only on build failure (unlikely).
fn ensure_widget(app: &AppHandle) -> Option<WebviewWindow> {
    if let Some(window) = app.get_webview_window(WIDGET_LABEL) {
        return Some(window);
    }

    // Position at top-right of the primary monitor's work area
    let (pos_x, pos_y) = match app.primary_monitor().ok().flatten() {
        Some(m) => {
            let size = m.size();
            let scale = m.scale_factor();
            let logical_w = size.width as f64 / scale;
            (logical_w - WIDGET_WIDTH - PADDING, PADDING)
        }
        None => (0.0, 0.0),
    };

    let window = WebviewWindowBuilder::new(app, WIDGET_LABEL, WebviewUrl::App("index.html?widget".into()))
        .title("")
        .inner_size(WIDGET_WIDTH, WIDGET_HEIGHT)
        .min_inner_size(WIDGET_WIDTH, WIDGET_HEIGHT)
        .resizable(false)
        .decorations(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .position(pos_x, pos_y)
        .visible(false)
        .build()
        .ok()?;

    // Apply Windows 11 rounded corners to undecorated widget window
    #[cfg(target_os = "windows")]
    crate::apply_window_rounding(&window);

    // Re-assert position after build (Tauri v2 may not respect builder position on all platforms)
    let _ = window.set_position(PhysicalPosition::new(
        (pos_x * app.primary_monitor().ok().flatten().map_or(1.0, |m| m.scale_factor())) as i32,
        (pos_y * app.primary_monitor().ok().flatten().map_or(1.0, |m| m.scale_factor())) as i32,
    ));

    Some(window)
}

pub fn show_widget(app: &AppHandle) {
    if let Some(w) = ensure_widget(app) {
        let _ = w.show();
        let _ = w.set_focus();
    }
}

pub fn hide_widget(app: &AppHandle) {
    if let Some(w) = app.get_webview_window(WIDGET_LABEL) {
        let _ = w.hide();
    }
}

fn show_main(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.set_focus();
    }
}

fn hide_main(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.hide();
    }
}

/// Restore main window and hide the widget.
pub fn show_main_hide_widget(app: &AppHandle) {
    show_main(app);
    hide_widget(app);
}

/// Hide main window and show the widget.
pub fn hide_main_show_widget(app: &AppHandle) {
    hide_main(app);
    show_widget(app);
}
