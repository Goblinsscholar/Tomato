use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::{Mutex, RwLock};
use sqlx::SqlitePool;

use crate::timer::TimerState;

pub struct AppState {
    pub timer: Mutex<TimerState>,
    pub db: SqlitePool,
    pub settings_cache: RwLock<HashMap<String, String>>,
    pub completing: AtomicBool,
}
