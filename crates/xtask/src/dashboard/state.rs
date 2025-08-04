use crate::event_bus::HooksmithEvent;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Error statistics for deduplication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorStats {
    /// Error hash
    pub hash: String,
    /// Error type (clippy, fmt, validation, etc.)
    pub error_type: String,
    /// Error message (normalized)
    pub message: String,
    /// Number of times this error has been seen
    pub count: u32,
    /// When this error was first seen
    pub first_seen: DateTime<Utc>,
    /// When this error was last seen
    pub last_seen: DateTime<Utc>,
    /// Whether this error is currently active
    pub is_active: bool,
    /// File location (if available)
    pub file: Option<String>,
    /// Line number (if available)
    pub line: Option<u32>,
    /// Error code (if available)
    pub code: Option<String>,
}

/// Event statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventStats {
    pub total_events: u64,
    pub events_by_type: HashMap<String, u64>,
    pub events_by_actor: HashMap<String, u64>,
    pub errors_count: u64,
    pub warnings_count: u64,
    pub info_count: u64,
    pub last_event_time: Option<DateTime<Utc>>,
}

/// Recent event for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentEvent {
    pub timestamp: DateTime<Utc>,
    pub event_type: String,
    pub actor: String,
    pub message: String,
    pub severity: EventSeverity,
}

/// Event severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EventSeverity {
    Info,
    Warning,
    Error,
}

/// System status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStatus {
    pub uptime: Duration,
    pub is_running: bool,
    pub auto_push_enabled: bool,
    pub file_watch_enabled: bool,
    pub heartbeat_interval: u64,
    pub last_heartbeat: Option<DateTime<Utc>>,
}

/// Central dashboard state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardState {
    /// Error statistics by hash
    pub errors: HashMap<String, ErrorStats>,
    /// Event statistics
    pub event_stats: EventStats,
    /// Recent events for display
    pub recent_events: Vec<RecentEvent>,
    /// System status
    pub system_status: SystemStatus,
    /// Last update time
    pub last_update: DateTime<Utc>,
    /// Current tab selection
    pub selected_tab: usize,
}

impl Default for DashboardState {
    fn default() -> Self {
        Self {
            errors: HashMap::new(),
            event_stats: EventStats {
                total_events: 0,
                events_by_type: HashMap::new(),
                events_by_actor: HashMap::new(),
                errors_count: 0,
                warnings_count: 0,
                info_count: 0,
                last_event_time: None,
            },
            recent_events: Vec::new(),
            system_status: SystemStatus {
                uptime: Duration::from_secs(0),
                is_running: true,
                auto_push_enabled: false,
                file_watch_enabled: false,
                heartbeat_interval: 30,
                last_heartbeat: None,
            },
            last_update: Utc::now(),
            selected_tab: 0,
        }
    }
}

/// Thread-safe state manager
#[derive(Clone)]
pub struct StateManager {
    state: Arc<Mutex<DashboardState>>,
    start_time: Instant,
}

impl StateManager {
    /// Create a new state manager
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(DashboardState::default())),
            start_time: Instant::now(),
        }
    }

    /// Get a clone of the current state
    pub fn get_state(&self) -> DashboardState {
        let mut state = self.state.lock().unwrap();
        state.system_status.uptime = self.start_time.elapsed();
        state.clone()
    }

    /// Update state from an event
    pub fn update_from_event(&self, event: &HooksmithEvent) {
        let mut state = self.state.lock().unwrap();

        // Update last update time
        state.last_update = event.ts;
        state.system_status.uptime = self.start_time.elapsed();

        // Update event statistics
        state.event_stats.total_events += 1;
        state.event_stats.last_event_time = Some(event.ts);

        // Count by event type
        *state
            .event_stats
            .events_by_type
            .entry(event.event.clone())
            .or_insert(0) += 1;

        // Count by actor
        *state
            .event_stats
            .events_by_actor
            .entry(event.actor.clone())
            .or_insert(0) += 1;

        // Handle error events
        if let Some(error) = &event.error {
            state.event_stats.errors_count += 1;
            if let Some(error_str) = error.as_str() {
                self.process_error_event(&mut state, event, error_str);
            }
        } else {
            state.event_stats.info_count += 1;
        }

        // Add to recent events
        let recent_event = RecentEvent {
            timestamp: event.ts,
            event_type: event.event.clone(),
            actor: event.actor.clone(),
            message: event
                .error
                .as_ref()
                .and_then(|e| e.as_str())
                .unwrap_or(&event.event)
                .to_string(),
            severity: if event.error.is_some() {
                EventSeverity::Error
            } else {
                EventSeverity::Info
            },
        };

        state.recent_events.push(recent_event);

        // Keep only last 100 events
        if state.recent_events.len() > 100 {
            state.recent_events.remove(0);
        }

        // Handle validation events
        if event.event == "validation_passed" {
            // Clear active errors
            for error in state.errors.values_mut() {
                error.is_active = false;
            }
        }
    }

    /// Process error events and update error statistics
    fn process_error_event(&self, state: &mut DashboardState, event: &HooksmithEvent, error: &str) {
        let error_message = error;
        let error_type = event.event.clone();

        // Normalize and hash the error
        let normalized = crate::error_deduplication::normalize_error(error_message);
        let hash = crate::error_deduplication::hash_error(&normalized);

        // Extract file and line information if available
        let (file, line, code) = self.extract_error_metadata(event);

        // Update error statistics
        let error_stats = state
            .errors
            .entry(hash.clone())
            .or_insert_with(|| ErrorStats {
                hash: hash.clone(),
                error_type: error_type.clone(),
                message: normalized.clone(),
                count: 0,
                first_seen: event.ts,
                last_seen: event.ts,
                is_active: true,
                file,
                line,
                code,
            });

        error_stats.count += 1;
        error_stats.last_seen = event.ts;
        error_stats.is_active = true;
    }

    /// Extract metadata from error events
    fn extract_error_metadata(
        &self,
        event: &HooksmithEvent,
    ) -> (Option<String>, Option<u32>, Option<String>) {
        // Try to extract from context if available
        if let Some(obj) = event.context.as_object() {
            let file = obj
                .get("file")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let line = obj.get("line").and_then(|v| v.as_u64()).map(|n| n as u32);
            let code = obj
                .get("code")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            return (file, line, code);
        }
        (None, None, None)
    }

    /// Update system status
    pub fn update_system_status(
        &self,
        auto_push_enabled: bool,
        file_watch_enabled: bool,
        heartbeat_interval: u64,
    ) {
        let mut state = self.state.lock().unwrap();
        state.system_status.auto_push_enabled = auto_push_enabled;
        state.system_status.file_watch_enabled = file_watch_enabled;
        state.system_status.heartbeat_interval = heartbeat_interval;
    }

    /// Update heartbeat
    pub fn update_heartbeat(&self) {
        let mut state = self.state.lock().unwrap();
        state.system_status.last_heartbeat = Some(Utc::now());
    }

    /// Change selected tab
    pub fn set_selected_tab(&self, tab: usize) {
        let mut state = self.state.lock().unwrap();
        state.selected_tab = tab;
    }

    /// Get the state Arc for sharing
    pub fn get_state_arc(&self) -> Arc<Mutex<DashboardState>> {
        self.state.clone()
    }
}

impl Default for StateManager {
    fn default() -> Self {
        Self::new()
    }
}
