use std::sync::atomic::{AtomicBool, Ordering};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use tauri::State;
use tauri::AppHandle;

use crate::db;
use crate::notifications;
use crate::state::AppState;
use crate::timer::TimerState;
use crate::widget;

/// RAII guard for the `completing` atomic flag.
/// Guarantees release on `Drop` (including during panics),
/// preventing permanent lockout of completion handling.
struct CompletingGuard<'a> {
    completing: &'a AtomicBool,
    armed: bool,
}

impl<'a> CompletingGuard<'a> {
    fn new(completing: &'a AtomicBool) -> Self {
        Self { completing, armed: false }
    }

    /// Atomically claim the completion slot.
    /// Returns `true` if this call successfully claimed it.
    fn try_claim(&mut self) -> bool {
        if !self.completing.swap(true, Ordering::AcqRel) {
            self.armed = true;
            true
        } else {
            false
        }
    }
}

impl<'a> Drop for CompletingGuard<'a> {
    fn drop(&mut self) {
        if self.armed {
            self.completing.store(false, Ordering::Release);
        }
    }
}

/// Append a timestamped message to the tomato debug log (%TEMP%\tomato-debug.log).
/// Used for diagnosing timer completion flow — not an error, just trace info.
fn write_debug_log(context: &str, msg: &dyn std::fmt::Display) {
    use std::io::Write;
    let log_dir = std::env::temp_dir();
    let log_path = log_dir.join("tomato-debug.log");
    let ts = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S");
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
    {
        let _ = writeln!(f, "[{}] [{}] {}", ts, context, msg);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimerStatusResponse {
    pub phase: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_end: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<String>,
    pub remaining_seconds: f64,
    pub elapsed_seconds: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub current_tag: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<i64>,
    /// Populated only on the first poll after a timer session completes.
    /// Frontend uses this to show an in-app completion dialog.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion_dialog: Option<CompletionDialogData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompletionDialogData {
    pub tag: Option<String>,
    pub duration_seconds: i32,
    pub completed_today: i32,
    pub session_type: String,
}

fn compute_response(timer: &TimerState) -> TimerStatusResponse {
    let now = Utc::now();
    let (remaining_seconds, elapsed_seconds) = timer.get_remaining_and_elapsed(now);
    TimerStatusResponse {
        phase: timer.phase_string().to_string(),
        target_end: timer.get_target_end().map(|dt| dt.to_rfc3339()),
        started_at: timer.get_started_at().map(|dt| dt.to_rfc3339()),
        remaining_seconds,
        elapsed_seconds,
        current_tag: timer.get_tag().map(|s| s.to_string()),
        session_id: timer.get_session_id(),
        completion_dialog: None,
    }
}

fn get_bool_setting(state: &AppState, key: &str) -> bool {
    state
        .settings_cache
        .read()
        .ok()
        .and_then(|cache| cache.get(key).cloned())
        .map(|v| v == "true")
        .unwrap_or_else(|| match key {
            "auto_start_break" | "auto_start_focus" => true,
            _ => false,
        })
}

fn get_int_setting(state: &AppState, key: &str) -> i32 {
    state
        .settings_cache
        .read()
        .ok()
        .and_then(|cache| cache.get(key).and_then(|v| v.parse().ok()))
        .unwrap_or_else(|| match key {
            "focus_duration" => 25,
            "break_duration" => 5,
            "long_break_duration" => 15,
            "sessions_before_long_break" => 4,
            _ => 25,
        })
}

pub(crate) async fn handle_focus_completion(
    state: &AppState,
    app_handle: &AppHandle,
    started_at: DateTime<Utc>,
    tag: String,
    session_id: i64,
) -> Result<TimerStatusResponse, String> {
    let now = Utc::now();
    let actual_duration = (now.signed_duration_since(started_at).num_seconds() as i32).max(0);

    let _ = db::sessions::complete_session(&state.db, session_id, &now.to_rfc3339(), actual_duration).await;
    let completed_today = db::sessions::get_focus_count_today(&state.db).await.unwrap_or(0) as i32;
    notifications::send_focus_complete(app_handle, &tag);

    let dialog = CompletionDialogData {
        tag: Some(tag.clone()),
        duration_seconds: actual_duration,
        completed_today,
        session_type: "focus".to_string(),
    };

    let auto_break = get_bool_setting(state, "auto_start_break");
    write_debug_log("focus-complete", &format!("auto_break={}, tag={}, session={}", auto_break, tag, session_id));

    if auto_break {
        let break_dur = get_int_setting(state, "break_duration");
        let long_break_dur = get_int_setting(state, "long_break_duration");
        let sessions_before_long = get_int_setting(state, "sessions_before_long_break");

        let use_long_break = sessions_before_long > 0 && (completed_today % sessions_before_long == 0);
        let break_minutes = if use_long_break { long_break_dur } else { break_dur };
        write_debug_log("focus-complete", &format!("break_dur={}, long_break_dur={}, sessions_before_long={}, focus_count={}, use_long_break={}, break_minutes={}",
            break_dur, long_break_dur, sessions_before_long, completed_today, use_long_break, break_minutes));

        let break_target_end = now + Duration::minutes(break_minutes as i64);
        match db::sessions::insert_session(
            &state.db,
            "",
            &now.to_rfc3339(),
            break_minutes * 60,
            "break",
        )
        .await
        {
            Ok(new_session_id) => {
                let break_state = TimerState::Breaking {
                    target_end: break_target_end,
                    started_at: now,
                    session_id: new_session_id,
                    tag: Some(tag.clone()),
                };
                let _ = db::timer_state::save_state(&state.db, &break_state).await;
                *state.timer.lock().map_err(|_| "internal_error".to_string())? = break_state;
                write_debug_log("focus-complete", &"break started OK");
            }
            Err(e) => {
                write_debug_log("focus-complete", &format!("insert_session failed: {}", e));
                *state.timer.lock().map_err(|_| "internal_error".to_string())? = TimerState::Idle;
                let _ = db::timer_state::clear_state(&state.db).await;
            }
        }
    } else {
        write_debug_log("focus-complete", &"auto_break=false, going idle");
        *state.timer.lock().map_err(|_| "internal_error".to_string())? = TimerState::Idle;
        let _ = db::timer_state::save_state(&state.db, &TimerState::Idle).await;
    }

    let timer_guard = state.timer.lock().map_err(|_| "internal_error".to_string())?;
    let mut response = compute_response(&timer_guard);
    response.completion_dialog = Some(dialog);
    write_debug_log("focus-complete", &format!("returning phase={}", response.phase));
    if response.phase == "focusing" || response.phase == "breaking" {
        widget::hide_main_show_widget(app_handle);
    } else {
        widget::show_main_hide_widget(app_handle);
    }
    Ok(response)
}

pub(crate) async fn handle_break_completion(
    state: &AppState,
    app_handle: &AppHandle,
    started_at: DateTime<Utc>,
    prior_tag: Option<String>,
    session_id: i64,
) -> Result<TimerStatusResponse, String> {
    let now = Utc::now();
    let actual_duration = (now.signed_duration_since(started_at).num_seconds() as i32).max(0);

    let _ = db::sessions::complete_session(&state.db, session_id, &now.to_rfc3339(), actual_duration).await;
    let completed_today = db::sessions::get_focus_count_today(&state.db).await.unwrap_or(0) as i32;
    notifications::send_break_complete(app_handle);

    let dialog = CompletionDialogData {
        tag: prior_tag.clone(),
        duration_seconds: actual_duration,
        completed_today,
        session_type: "break".to_string(),
    };

    let auto_focus = get_bool_setting(state, "auto_start_focus");
    if auto_focus {
        let focus_dur = get_int_setting(state, "focus_duration");
        let next_tag = prior_tag.unwrap_or_else(|| "默认".to_string());
        // Auto-register tag in tags table (best-effort, non-fatal)
        let color = db::tags::assign_tag_color(&state.db, &next_tag).await;
        let _ = db::tags::insert_tag_if_not_exists(&state.db, &next_tag, &color).await;
        let focus_target_end = now + Duration::minutes(focus_dur as i64);
        match db::sessions::insert_session(
            &state.db,
            &next_tag,
            &now.to_rfc3339(),
            focus_dur * 60,
            "focus",
        )
        .await
        {
            Ok(new_session_id) => {
                let focus_state = TimerState::Focusing {
                    target_end: focus_target_end,
                    started_at: now,
                    tag: next_tag,
                    session_id: new_session_id,
                };
                let _ = db::timer_state::save_state(&state.db, &focus_state).await;
                *state.timer.lock().map_err(|_| "internal_error".to_string())? = focus_state;
            }
            Err(_) => {
                *state.timer.lock().map_err(|_| "internal_error".to_string())? = TimerState::Idle;
                let _ = db::timer_state::save_state(&state.db, &TimerState::Idle).await;
            }
        }
    } else {
        *state.timer.lock().map_err(|_| "internal_error".to_string())? = TimerState::Idle;
        let _ = db::timer_state::save_state(&state.db, &TimerState::Idle).await;
    }

    let timer_guard = state.timer.lock().map_err(|_| "internal_error".to_string())?;
    let mut response = compute_response(&timer_guard);
    response.completion_dialog = Some(dialog);
    if response.phase == "focusing" || response.phase == "breaking" {
        widget::hide_main_show_widget(app_handle);
    } else {
        widget::show_main_hide_widget(app_handle);
    }
    Ok(response)
}

// ---- Tauri Commands ----

#[tauri::command]
pub async fn start_focus(
    app_handle: AppHandle,
    state: State<'_, AppState>,
    tag: String,
    duration_minutes: i32,
) -> Result<TimerStatusResponse, String> {
    if tag.trim().is_empty() {
        return Err("invalid_tag".to_string());
    }
    if !(1..=120).contains(&duration_minutes) {
        return Err("invalid_duration".to_string());
    }

    // Check Idle state (brief lock, then drop)
    {
        let timer = state.timer.lock().map_err(|_| "internal_error".to_string())?;
        if !timer.is_idle() {
            return Err("timer_not_idle".to_string());
        }
    }

    let now = Utc::now();
    let target_end = now + Duration::minutes(duration_minutes as i64);
    let planned_seconds = duration_minutes * 60;

    let session_id = db::sessions::insert_session(
        &state.db,
        &tag,
        &now.to_rfc3339(),
        planned_seconds,
        "focus",
    )
    .await
    .map_err(|e| format!("db_error: {}", e))?;

    // Auto-register tag in tags table (best-effort, non-fatal)
    let color = db::tags::assign_tag_color(&state.db, &tag).await;
    let _ = db::tags::insert_tag_if_not_exists(&state.db, &tag, &color).await;

    let focus_state = TimerState::Focusing {
        target_end,
        started_at: now,
        tag: tag.clone(),
        session_id,
    };

    {
        let mut timer = state.timer.lock().map_err(|_| "internal_error".to_string())?;
        *timer = focus_state;
    }

    let _ = db::timer_state::save_state(&state.db, &TimerState::Focusing {
        target_end,
        started_at: now,
        tag,
        session_id,
    })
    .await;

    widget::hide_main_show_widget(&app_handle);

    let timer = state.timer.lock().map_err(|_| "internal_error".to_string())?;
    Ok(compute_response(&timer))
}

#[tauri::command]
pub async fn pause(state: State<'_, AppState>) -> Result<TimerStatusResponse, String> {
    // Extract timer state (brief lock, clone what we need, drop)
    let (_is_active, is_already_completed, phase, tag, session_id, remaining, elapsed) = {
        let timer = state.timer.lock().map_err(|_| "internal_error".to_string())?;
        match &*timer {
            TimerState::Idle | TimerState::Paused { .. } => {
                return Err("timer_not_running".to_string());
            }
            TimerState::Focusing { target_end, started_at, .. }
            | TimerState::Breaking { target_end, started_at, .. } => {
                let now = Utc::now();
                if now >= *target_end {
                    // Already completed - caller should use get_timer_status
                    (true, true, String::new(), String::new(), 0i64, 0i64, 0i64)
                } else {
                    let p = timer.phase_string().to_string();
                    let t = timer.get_tag().map(|s| s.to_string()).unwrap_or_default();
                    let sid = timer.get_session_id().unwrap_or(0);
                    let rem = target_end.signed_duration_since(now).num_seconds().max(0);
                    let ela = now.signed_duration_since(*started_at).num_seconds().max(0);
                    (true, false, p, t, sid, rem, ela)
                }
            }
        }
    };

    if is_already_completed {
        // Delegate to get_timer_status to handle completion
        let timer = state.timer.lock().map_err(|_| "internal_error".to_string())?;
        return Ok(compute_response(&timer));
    }

    let paused = TimerState::Paused {
        paused_phase: match phase.as_str() {
            "breaking" => crate::timer::PausedPhase::Breaking,
            _ => crate::timer::PausedPhase::Focusing,
        },
        remaining_seconds: remaining,
        elapsed_seconds: elapsed,
        tag,
        session_id,
    };

    {
        let mut timer = state.timer.lock().map_err(|_| "internal_error".to_string())?;
        *timer = paused.clone();
    }
    let _ = db::timer_state::save_state(&state.db, &paused).await;

    let timer = state.timer.lock().map_err(|_| "internal_error".to_string())?;
    Ok(compute_response(&timer))
}

#[tauri::command]
pub async fn resume(
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<TimerStatusResponse, String> {
    // Extract paused state (brief lock, clone)
    let paused_data = {
        let timer = state.timer.lock().map_err(|_| "internal_error".to_string())?;
        match &*timer {
            TimerState::Paused { paused_phase, remaining_seconds, elapsed_seconds, tag, session_id } => {
                Some((paused_phase.clone(), *remaining_seconds, *elapsed_seconds, tag.clone(), *session_id))
            }
            _ => None,
        }
    };

    let (paused_phase, remaining_seconds, elapsed_seconds, tag, session_id) = match paused_data {
        Some(data) => data,
        None => return Err("timer_not_paused".to_string()),
    };

    let now = Utc::now();
    let new_target_end = now + Duration::seconds(remaining_seconds);
    let new_started_at = now - Duration::seconds(elapsed_seconds);

    if paused_phase == crate::timer::PausedPhase::Focusing {
        // Auto-register tag in tags table (best-effort, non-fatal)
        let color = db::tags::assign_tag_color(&state.db, &tag).await;
        let _ = db::tags::insert_tag_if_not_exists(&state.db, &tag, &color).await;

        let resumed = TimerState::Focusing {
            target_end: new_target_end,
            started_at: new_started_at,
            tag,
            session_id,
        };
        {
            let mut timer = state.timer.lock().map_err(|_| "internal_error".to_string())?;
            *timer = resumed.clone();
        }
        let _ = db::timer_state::save_state(&state.db, &resumed).await;
    } else {
        let resumed = TimerState::Breaking {
            target_end: new_target_end,
            started_at: new_started_at,
            session_id,
            tag: if tag.is_empty() { None } else { Some(tag) },
        };
        {
            let mut timer = state.timer.lock().map_err(|_| "internal_error".to_string())?;
            *timer = resumed.clone();
        }
        let _ = db::timer_state::save_state(&state.db, &resumed).await;
    }

    let timer = state.timer.lock().map_err(|_| "internal_error".to_string())?;
    widget::hide_main_show_widget(&app_handle);
    Ok(compute_response(&timer))
}

#[tauri::command]
pub async fn reset(
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<TimerStatusResponse, String> {
    // Extract session_id before doing async work
    let session_id = {
        let timer = state.timer.lock().map_err(|_| "internal_error".to_string())?;
        timer.get_session_id()
    };

    if let Some(sid) = session_id {
        let _ = db::sessions::cancel_session(&state.db, sid).await;
    }

    {
        let mut timer = state.timer.lock().map_err(|_| "internal_error".to_string())?;
        *timer = TimerState::Idle;
    }
    let _ = db::timer_state::clear_state(&state.db).await;

    widget::show_main_hide_widget(&app_handle);

    let timer = state.timer.lock().map_err(|_| "internal_error".to_string())?;
    Ok(compute_response(&timer))
}

#[tauri::command]
pub async fn get_timer_status(
    app_handle: AppHandle,
    state: State<'_, AppState>,
) -> Result<TimerStatusResponse, String> {
    // Check if timer needs completion (brief lock, clone state)
    let completion = {
        let timer = state.timer.lock().map_err(|_| "internal_error".to_string())?;
        let now = Utc::now();
        match &*timer {
            TimerState::Focusing { target_end, started_at, tag, session_id } if now >= *target_end => {
                write_debug_log("get_timer_status", &format!("detected focusing completion, session={}", session_id));
                Some(("focusing", *started_at, Some(tag.clone()), *session_id))
            }
            TimerState::Breaking { target_end, started_at, session_id, .. } if now >= *target_end => {
                write_debug_log("get_timer_status", &format!("detected breaking completion, session={}", session_id));
                let timer_tag = timer.get_tag().map(|s| s.to_string());
                Some(("breaking", *started_at, timer_tag, *session_id))
            }
            _ => None,
        }
    };

    // Claim completion slot atomically — CompletingGuard releases it on drop
    let mut guard = CompletingGuard::new(&state.completing);

    let need_completion = completion.is_some();
    if need_completion && !guard.try_claim() {
        write_debug_log("get_timer_status", &"completing already claimed, bailing out");
        let timer = state.timer.lock().map_err(|_| "internal_error".to_string())?;
        return Ok(compute_response(&timer));
    }

    match completion {
        Some(("focusing", started_at, Some(tag), session_id)) => {
            let response = handle_focus_completion(&state, &app_handle, started_at, tag, session_id).await;
            response
        }
        Some(("focusing", _, None, _)) => {
            let timer = state.timer.lock().map_err(|_| "internal_error".to_string())?;
            Ok(compute_response(&timer))
        }
        Some(("breaking", started_at, prior_tag, session_id)) => {
            let response = handle_break_completion(&state, &app_handle, started_at, prior_tag, session_id).await;
            response
        }
        _ => {
            let timer = state.timer.lock().map_err(|_| "internal_error".to_string())?;
            Ok(compute_response(&timer))
        }
    }
    // guard drops here: releases completing if armed (even on panic)
}