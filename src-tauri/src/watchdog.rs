use chrono::{DateTime, Utc};
use std::sync::atomic::Ordering;
use tauri::AppHandle;
use tauri::Manager;

use crate::commands::timer;
use crate::state::AppState;
use crate::timer::TimerState;

/// Start a background watchdog that checks for timer expiration every 500ms
/// and handles completion automatically.
///
/// This eliminates the race condition where two webview windows (main + widget)
/// compete to process completion via `get_timer_status`. The watchdog is the
/// single authority for state transitions; `get_timer_status` only reads state.
pub fn start_watchdog(app_handle: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_millis(500));
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            interval.tick().await;

            // Phase 1: Check if timer is expired (sync, drop state before await)
            let expiry = {
                let state = app_handle.state::<AppState>();
                let Ok(timer) = state.timer.lock() else { continue };
                let now = Utc::now();
                match &*timer {
                    TimerState::Focusing {
                        target_end,
                        started_at,
                        tag,
                        session_id,
                    } if now >= *target_end => Some(CompletionData {
                        phase: "focusing",
                        started_at: *started_at,
                        tag: Some(tag.clone()),
                        session_id: *session_id,
                    }),
                    TimerState::Breaking {
                        target_end,
                        started_at,
                        session_id,
                        ref tag,
                    } if now >= *target_end => Some(CompletionData {
                        phase: "breaking",
                        started_at: *started_at,
                        tag: tag.clone(),
                        session_id: *session_id,
                    }),
                    _ => None,
                }
            };

            // Phase 2: Claim completion guard and process
            if let Some(data) = expiry {
                let claimed = {
                    let state = app_handle.state::<AppState>();
                    state
                        .completing
                        .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
                        .is_ok()
                };

                if claimed {
                    let result = match data.phase {
                        "focusing" => {
                            if let Some(tag) = data.tag {
                                let state = app_handle.state::<AppState>();
                                timer::handle_focus_completion(
                                    &state,
                                    &app_handle,
                                    data.started_at,
                                    tag,
                                    data.session_id,
                                )
                                .await
                            } else {
                                let state = app_handle.state::<AppState>();
                                state.completing.store(false, Ordering::Release);
                                continue;
                            }
                        }
                        "breaking" => {
                            let state = app_handle.state::<AppState>();
                            timer::handle_break_completion(
                                &state,
                                &app_handle,
                                data.started_at,
                                data.tag,
                                data.session_id,
                            )
                            .await
                        }
                        _ => unreachable!(),
                    };

                    // Release completion guard after handling
                    let state = app_handle.state::<AppState>();
                    state.completing.store(false, Ordering::Release);

                    if let Err(e) = result {
                        log_error(&format!("watchdog completion failed: {}", e));
                    }
                }
            }
        }
    });
}

struct CompletionData {
    phase: &'static str,
    started_at: DateTime<Utc>,
    tag: Option<String>,
    session_id: i64,
}

fn log_error(msg: &str) {
    use std::io::Write;
    let log_dir = std::env::temp_dir();
    let log_path = log_dir.join("tomato-error.log");
    let ts = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S");
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
    {
        let _ = writeln!(f, "[{}] [watchdog] {}", ts, msg);
    }
}
