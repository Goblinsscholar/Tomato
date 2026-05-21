use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use std::sync::atomic::AtomicBool;
use std::sync::{Mutex, RwLock};
use tauri::{
    image::Image,
    Manager,
    menu::{MenuBuilder, MenuItemBuilder},
    tray::TrayIconBuilder,
};
use std::io::Cursor;
use png;

use tauri_plugin_global_shortcut::GlobalShortcutExt;
use tauri_plugin_notification::NotificationExt;

mod commands;
mod db;
mod notifications;
mod state;
mod timer;
mod watchdog;
mod widget;

use state::AppState;
use timer::TimerState;

/// Append a timestamped error message to %TEMP%\tomato-error.log.
/// Always writable — does not depend on app data dir.
fn write_error_log(context: &str, error: &dyn std::fmt::Display) {
    use std::io::Write;
    let log_dir = std::env::temp_dir();
    let log_path = log_dir.join("tomato-error.log");
    let ts = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S");
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
    {
        let _ = writeln!(f, "[{}] [{}] {}", ts, context, error);
    }
}

// Windows DWM FFI for window corner preference
#[cfg(target_os = "windows")]
#[link(name = "dwmapi")]
extern "system" {
    fn DwmSetWindowAttribute(
        hwnd: isize,
        dw_attribute: u32,
        pv_attribute: *const std::ffi::c_void,
        cb_attribute: u32,
    ) -> i32;
}

/// Create a simple 32x32 RGBA icon in the app's brand green (#22c55e)
fn make_tray_icon() -> Image<'static> {
    // Try to embed and decode the generated 32x32 PNG so the tray icon matches the app icon.
    // Falls back to the original green circle if decoding fails.
    const EMBED: &'static [u8] = include_bytes!("../icons/32x32.png");

    if let Ok(mut reader) = png::Decoder::new(Cursor::new(EMBED)).read_info() {
        let (width, height) = { let info = reader.info(); (info.width, info.height) };
        let mut buf = vec![0; reader.output_buffer_size()];
        if reader.next_frame(&mut buf).is_ok() {
            return Image::new_owned(buf, width, height);
        }
    }

    // Fallback: original green circle 32x32
    let width = 32;
    let height = 32;
    let mut rgba = Vec::with_capacity((width * height * 4) as usize);
    for y in 0..height {
        for x in 0..width {
            let cx = x as i32 - 16;
            let cy = y as i32 - 16;
            let dist = ((cx * cx + cy * cy) as f64).sqrt();
            if dist < 14.0 {
                // Green circle
                rgba.push(34); // R
                rgba.push(197); // G
                rgba.push(94); // B
                rgba.push(255); // A
            } else {
                rgba.push(0);
                rgba.push(0);
                rgba.push(0);
                rgba.push(0);
            }
        }
    }
    Image::new_owned(rgba, width, height)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new()
            .with_handler(|app, shortcut, _event| {
                let shortcut_str = shortcut.to_string();
                if shortcut_str == "shift+control+KeyF" {
                    // Toggle: if timer active and main visible → switch to widget
                    let timer_active = app.state::<AppState>()
                        .timer.lock()
                        .ok()
                        .map_or(false, |t| t.is_active());

                    if timer_active {
                        if let Some(main) = app.get_webview_window("main") {
                            if main.is_visible().unwrap_or(false) {
                                let _ = main.hide();
                                crate::widget::show_widget(app);
                                return;
                            }
                        }
                    }

                    // Default: show main window
                    crate::widget::hide_widget(app);
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            })
            .build()
        )
        .plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .setup(|app| {
            // ---- Database setup ----
            // DB in a writable location: use Tauri app data dir or temp fallback
            let db_dir = match app.path().app_data_dir() {
                Ok(dir) => dir,
                Err(_) => {
                    let fallback = std::env::temp_dir().join("tomato-data");
                    let _ = std::fs::create_dir_all(&fallback);
                    fallback
                }
            };
            let _ = std::fs::create_dir_all(&db_dir);
            let db_path = db_dir.join("tomato.db");

            let pool = tauri::async_runtime::block_on(async {
                let opts = SqliteConnectOptions::new()
                    .filename(&db_path)
                    .create_if_missing(true);
                let pool = SqlitePoolOptions::new()
                    .max_connections(2)
                    .connect_with(opts)
                    .await
                    .map_err(|e| format!("connect: {} (path: {})", e, db_path.display()))?;
                db::init_db(&pool).await.map_err(|e| e.to_string())?;
                Ok::<_, String>(pool)
            })
            .map_err(|e: String| -> Box<dyn std::error::Error> {
                write_error_log("database-init", &e);
                std::io::Error::new(std::io::ErrorKind::Other, e).into()
            })?;

            // ---- Crash recovery ----
            // Any non-Idle state on startup means the app was fully quit while a timer was
            // active. Cancel the session and reset to Idle — the user wasn't focusing.
            let recovered = tauri::async_runtime::block_on(async {
                db::timer_state::load_state(&pool).await.unwrap_or(None)
            });
            let timer_state = match recovered {
                None => TimerState::Idle,
                Some(state @ (TimerState::Focusing { .. }
                | TimerState::Breaking { .. }
                | TimerState::Paused { .. })) => {
                    if let Some(sid) = state.get_session_id() {
                        let _ = tauri::async_runtime::block_on(async {
                            db::sessions::cancel_session(&pool, sid).await
                        });
                    }
                    let _ = tauri::async_runtime::block_on(async {
                        db::timer_state::clear_state(&pool).await
                    });
                    TimerState::Idle
                }
                Some(TimerState::Idle) => TimerState::Idle,
            };

            // Pre-load settings cache
            let settings_map = tauri::async_runtime::block_on(async {
                db::settings::get_all(&pool).await
            })
            .map_err(|e| -> Box<dyn std::error::Error> {
                write_error_log("settings-load", &e);
                std::io::Error::new(std::io::ErrorKind::Other, e).into()
            })?;

            let app_state = AppState {
                timer: Mutex::new(timer_state),
                db: pool,
                settings_cache: RwLock::new(settings_map),
                completing: AtomicBool::new(false),
            };
            app.manage(app_state);

            // ---- Start timer watchdog (background completion detection) ----
            watchdog::start_watchdog(app.handle().clone());

            // ---- Register global shortcuts ----
            // Handler registered via Builder::with_handler above.
            // Shortcuts need to be explicitly registered after the app is ready:
            let _ = app.global_shortcut().register("Ctrl+Shift+F");

            // ---- System tray (non-critical) ----
            if let Err(e) = (|| -> Result<(), Box<dyn std::error::Error>> {
                let show_item = MenuItemBuilder::new("显示 (Ctrl+Shift+F)").id("show").build(app)?;
                let quit_item = MenuItemBuilder::new("退出").id("quit").build(app)?;
                let menu = MenuBuilder::new(app)
                    .item(&show_item)
                    .separator()
                    .item(&quit_item)
                    .build()?;

                TrayIconBuilder::new()
                    .icon(make_tray_icon())
                    .menu(&menu)
                    .tooltip("Tomato")
                    .on_menu_event(|app, event| {
                        match event.id().as_ref() {
                            "show" => {
                                crate::widget::hide_widget(app);
                                if let Some(window) = app.get_webview_window("main") {
                                    let _ = window.show();
                                    let _ = window.set_focus();
                                }
                            }
                            "quit" => {
                                app.exit(0);
                            }
                            _ => {}
                        }
                    })
                    .build(app)?;
                Ok(())
            })() {
                write_error_log("tray", &e);
            }

            // Request notification permission (fire-and-forget, best-effort)
            {
                let handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    let _ = handle.notification().request_permission();
                });
            }

            // Apply Windows 11 rounded corners to undecorated windows
            #[cfg(target_os = "windows")]
            if let Some(window) = app.get_webview_window("main") {
                apply_window_rounding(&window);
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event {
                let label = window.label();
                if label == "widget" {
                    // Widget closed by OS: restore main window, timer keeps running
                    let app = window.app_handle();
                    if let Some(main) = app.get_webview_window("main") {
                        let _ = main.show();
                        let _ = main.set_focus();
                    }
                } else {
                    let app = window.app_handle();
                    // Timer active → show widget before hiding main
                    let timer_active = app.state::<AppState>()
                        .timer.lock()
                        .ok()
                        .map_or(false, |t| t.is_active());
                    if timer_active {
                        crate::widget::show_widget(&app);
                    }
                    let _ = window.hide();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::timer::start_focus,
            commands::timer::pause,
            commands::timer::resume,
            commands::timer::reset,
            commands::timer::get_timer_status,
            commands::sessions::get_today_sessions,
            commands::sessions::get_sessions_in_range,
            commands::sessions::get_daily_stats,
            commands::statistics::get_weekly_stats,
            commands::statistics::get_tag_distribution,
            commands::statistics::get_monthly_stats,
            commands::tags::get_all_tags,
            commands::tags::create_tag,
            commands::tags::delete_tag,
            commands::tags::update_tag,
            commands::settings::get_settings,
            commands::settings::update_setting,
        ])
        .run(tauri::generate_context!())
        .unwrap_or_else(|e| {
            let msg = format!("Tauri application error: {}", e);
            write_error_log("fatal", &msg);
            panic!("{}", msg);
        });
}

/// Apply Windows 11 rounded corners via DWM.
/// WS_THICKFRAME is NOT needed on Win11 22H2+ (build 22621+), so we only set
/// DWMWCP_ROUND to avoid the window geometry recalc that happens when adding
/// WS_THICKFRAME dynamically to an already-visible window.
#[cfg(target_os = "windows")]
pub fn apply_window_rounding(window: &tauri::WebviewWindow) {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};

    if let Ok(handle) = window.window_handle() {
        if let RawWindowHandle::Win32(win32) = handle.as_ref() {
            let hwnd = win32.hwnd.get();

            unsafe {
                const DWMWA_WINDOW_CORNER_PREFERENCE: u32 = 33;
                const DWMWCP_ROUND: u32 = 2;
                let preference = DWMWCP_ROUND;
                DwmSetWindowAttribute(
                    hwnd,
                    DWMWA_WINDOW_CORNER_PREFERENCE,
                    &preference as *const u32 as *const std::ffi::c_void,
                    std::mem::size_of::<u32>() as u32,
                );
            }
        }
    }
}

#[cfg(not(target_os = "windows"))]
pub fn apply_window_rounding(_window: &tauri::WebviewWindow) {}
