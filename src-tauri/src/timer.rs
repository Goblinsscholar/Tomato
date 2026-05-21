use chrono::{DateTime, Utc};
#[cfg(test)]
use chrono::Duration;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum PausedPhase {
    Focusing,
    Breaking,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TimerState {
    Idle,
    Focusing {
        target_end: DateTime<Utc>,
        started_at: DateTime<Utc>,
        tag: String,
        session_id: i64,
    },
    Breaking {
        target_end: DateTime<Utc>,
        started_at: DateTime<Utc>,
        session_id: i64,
        tag: Option<String>,
    },
    Paused {
        paused_phase: PausedPhase,
        remaining_seconds: i64,
        elapsed_seconds: i64,
        tag: String,
        session_id: i64,
    },
}

impl TimerState {
    pub fn phase_string(&self) -> &str {
        match self {
            Self::Idle => "idle",
            Self::Focusing { .. } => "focusing",
            Self::Breaking { .. } => "breaking",
            Self::Paused { .. } => "paused",
        }
    }

    pub fn is_idle(&self) -> bool {
        matches!(self, Self::Idle)
    }

    pub fn is_active(&self) -> bool {
        matches!(self, Self::Focusing { .. } | Self::Breaking { .. })
    }

    pub fn is_paused(&self) -> bool {
        matches!(self, Self::Paused { .. })
    }

    pub fn get_tag(&self) -> Option<&str> {
        match self {
            Self::Focusing { tag, .. } => Some(tag.as_str()),
            Self::Breaking { tag: Some(tag), .. } => Some(tag.as_str()),
            Self::Paused { tag, .. } => Some(tag.as_str()),
            _ => None,
        }
    }

    pub fn get_session_id(&self) -> Option<i64> {
        match self {
            Self::Focusing { session_id, .. } => Some(*session_id),
            Self::Breaking { session_id, .. } => Some(*session_id),
            Self::Paused { session_id, .. } => Some(*session_id),
            Self::Idle => None,
        }
    }

    pub fn get_target_end(&self) -> Option<DateTime<Utc>> {
        match self {
            Self::Focusing { target_end, .. } => Some(*target_end),
            Self::Breaking { target_end, .. } => Some(*target_end),
            _ => None,
        }
    }

    pub fn get_started_at(&self) -> Option<DateTime<Utc>> {
        match self {
            Self::Focusing { started_at, .. } => Some(*started_at),
            Self::Breaking { started_at, .. } => Some(*started_at),
            _ => None,
        }
    }

    pub fn get_remaining_and_elapsed(&self, now: DateTime<Utc>) -> (f64, f64) {
        match self {
            Self::Focusing {
                target_end,
                started_at,
                ..
            }
            | Self::Breaking {
                target_end,
                started_at,
                ..
            } => {
                let remaining = (target_end.signed_duration_since(now).num_milliseconds() as f64
                    / 1000.0)
                    .max(0.0);
                let elapsed = (now.signed_duration_since(*started_at).num_milliseconds() as f64
                    / 1000.0)
                    .max(0.0);
                (remaining, elapsed)
            }
            Self::Paused {
                remaining_seconds,
                elapsed_seconds,
                ..
            } => (*remaining_seconds as f64, *elapsed_seconds as f64),
            Self::Idle => (0.0, 0.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_focusing(remaining_secs: i64) -> TimerState {
        let now = Utc::now();
        TimerState::Focusing {
            target_end: now + Duration::seconds(remaining_secs),
            started_at: now,
            tag: "test".to_string(),
            session_id: 1,
        }
    }

    fn make_breaking(remaining_secs: i64) -> TimerState {
        let now = Utc::now();
        TimerState::Breaking {
            target_end: now + Duration::seconds(remaining_secs),
            started_at: now,
            session_id: 2,
            tag: None,
        }
    }

    fn make_paused() -> TimerState {
        TimerState::Paused {
            paused_phase: PausedPhase::Focusing,
            remaining_seconds: 500,
            elapsed_seconds: 1000,
            tag: "test".to_string(),
            session_id: 1,
        }
    }

    #[test]
    fn test_idle_defaults() {
        let state = TimerState::Idle;
        assert_eq!(state.phase_string(), "idle");
        assert!(state.is_idle());
        assert!(!state.is_active());
        assert!(!state.is_paused());
        assert_eq!(state.get_tag(), None);
        assert_eq!(state.get_session_id(), None);
        assert_eq!(state.get_target_end(), None);
        assert_eq!(state.get_started_at(), None);
        assert_eq!(state.get_remaining_and_elapsed(Utc::now()), (0.0, 0.0));
    }

    #[test]
    fn test_focusing_phase() {
        let state = make_focusing(1500);
        assert_eq!(state.phase_string(), "focusing");
        assert!(!state.is_idle());
        assert!(state.is_active());
        assert!(!state.is_paused());
        assert_eq!(state.get_tag(), Some("test"));
        assert_eq!(state.get_session_id(), Some(1));
        assert!(state.get_target_end().is_some());
        assert!(state.get_started_at().is_some());
    }

    #[test]
    fn test_breaking_phase() {
        let state = make_breaking(300);
        assert_eq!(state.phase_string(), "breaking");
        assert!(state.is_active());
        assert_eq!(state.get_session_id(), Some(2));
        assert_eq!(state.get_tag(), None);
    }

    #[test]
    fn test_paused_phase() {
        let state = make_paused();
        assert_eq!(state.phase_string(), "paused");
        assert!(!state.is_active());
        assert!(state.is_paused());
        assert_eq!(state.get_tag(), Some("test"));
        assert_eq!(state.get_session_id(), Some(1));
        assert_eq!(state.get_target_end(), None);
        assert_eq!(state.get_started_at(), None);
    }

    #[test]
    fn test_remaining_and_elapsed_focusing() {
        let state = make_focusing(100);
        let now = Utc::now();
        let (remaining, elapsed) = state.get_remaining_and_elapsed(now);
        // Should be approximately 100 seconds remaining
        assert!((remaining - 100.0).abs() < 1.0);
        // Should be approximately 0 seconds elapsed
        assert!(elapsed < 1.0);
    }

    #[test]
    fn test_remaining_and_elapsed_paused() {
        let state = make_paused();
        let (remaining, elapsed) = state.get_remaining_and_elapsed(Utc::now());
        assert_eq!(remaining, 500.0);
        assert_eq!(elapsed, 1000.0);
    }

    #[test]
    fn test_remaining_clamped_to_zero() {
        let now = Utc::now();
        let state = TimerState::Focusing {
            target_end: now - Duration::seconds(10),
            started_at: now - Duration::seconds(610),
            tag: "test".to_string(),
            session_id: 1,
        };
        let (remaining, elapsed) = state.get_remaining_and_elapsed(now);
        assert_eq!(remaining, 0.0);
        assert!((elapsed - 610.0).abs() < 1.0);
    }
}
