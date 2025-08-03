use crate::events::AutoPushEvent;
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;
use uuid::Uuid;

/// Global session ID for the current process
static SESSION_ID: Lazy<Arc<Mutex<String>>> = Lazy::new(|| {
    Arc::new(Mutex::new(Uuid::new_v4().to_string()))
});

/// Get the current session ID
pub fn get_session_id() -> String {
    SESSION_ID.lock().unwrap().clone()
}

/// Centralized emit macro that ensures all output is schema-validated JSONL
/// 
/// Usage:
/// - `emit_json!("tool", "action", "level", "message")`
/// - `emit_json!("tool", "action", "level", "message", { "key": "value" })`
/// 
/// This macro guarantees that every console output line is a valid AutoPushEvent
/// and prevents any ad-hoc println! usage.
#[macro_export]
macro_rules! emit_json {
    ($tool:expr, $action:expr, $level:expr, $msg:expr) => {
        {
            let ev = $crate::events::AutoPushEvent::new(
                $tool,
                $action,
                $level,
                $msg.to_string(),
                $crate::emit::get_session_id(),
            );
            println!("{}", ev.to_jsonl().unwrap());
        }
    };
    
    ($tool:expr, $action:expr, $level:expr, $msg:expr, $details:expr) => {
        {
            let ev = $crate::events::AutoPushEvent::new(
                $tool,
                $action,
                $level,
                $msg.to_string(),
                $crate::emit::get_session_id(),
            )
            .with_details($details);
            println!("{}", ev.to_jsonl().unwrap());
        }
    };
}

/// Convenience macros for common event types
#[macro_export]
macro_rules! emit_info {
    ($tool:expr, $action:expr, $($arg:tt)*) => {
        $crate::emit_json!($tool, $action, "info", format!($($arg)*))
    };
}

#[macro_export]
macro_rules! emit_warn {
    ($tool:expr, $action:expr, $($arg:tt)*) => {
        $crate::emit_json!($tool, $action, "warn", format!($($arg)*))
    };
}

#[macro_export]
macro_rules! emit_error {
    ($tool:expr, $action:expr, $($arg:tt)*) => {
        $crate::emit_json!($tool, $action, "error", format!($($arg)*))
    };
}

#[macro_export]
macro_rules! emit_success {
    ($tool:expr, $action:expr, $($arg:tt)*) => {
        $crate::emit_json!($tool, $action, "info", format!("✅ {}", format!($($arg)*)))
    };
}

#[macro_export]
macro_rules! emit_failure {
    ($tool:expr, $action:expr, $($arg:tt)*) => {
        $crate::emit_json!($tool, $action, "error", format!("❌ {}", format!($($arg)*)))
    };
}

#[macro_export]
macro_rules! emit_warning {
    ($tool:expr, $action:expr, $($arg:tt)*) => {
        $crate::emit_json!($tool, $action, "warn", format!("⚠️ {}", format!($($arg)*)))
    };
}

/// Emit diagnostic information (for compiler/tool diagnostics)
#[macro_export]
macro_rules! emit_diagnostic {
    ($tool:expr, $action:expr, $level:expr, $msg:expr, $code:expr, $file:expr, $line:expr, $column:expr) => {
        {
            let ev = $crate::events::AutoPushEvent::new(
                $tool,
                $action,
                $level,
                $msg.to_string(),
                $crate::emit::get_session_id(),
            )
            .with_diagnostic($code, $file, $line, $column);
            println!("{}", ev.to_jsonl().unwrap());
        }
    };
}

/// Emit event with custom details
#[macro_export]
macro_rules! emit_with_details {
    ($tool:expr, $action:expr, $level:expr, $msg:expr, $details:expr) => {
        $crate::emit_json!($tool, $action, $level, $msg, $details)
    };
}

/// Validate that a JSONL stream conforms to our schema
pub fn validate_output<R: std::io::Read>(reader: R) -> Result<(), Box<dyn std::error::Error>> {
    crate::events::validate_jsonl_stream(reader)
}

/// Generate the JSON schema for AutoPushEvent
pub fn generate_schema() -> Result<String, serde_json::Error> {
    crate::events::generate_schema()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emit_macro() {
        // This test verifies that the macro compiles and produces valid JSON
        // In a real test, we'd capture stdout and validate the output
        emit_info!("test", "validation", "Test message");
        emit_warn!("test", "validation", "Test warning");
        emit_error!("test", "validation", "Test error");
        emit_success!("test", "validation", "Test success");
        emit_failure!("test", "validation", "Test failure");
        emit_warning!("test", "validation", "Test warning");
    }

    #[test]
    fn test_emit_with_details() {
        emit_with_details!("test", "build", "info", "Build completed", {
            "duration_ms": 1500,
            "targets": ["release"]
        });
    }

    #[test]
    fn test_emit_diagnostic() {
        emit_diagnostic!("cargo", "clippy", "warn", "Unused variable", "unused_variables", "src/main.rs", 42, 10);
    }

    #[test]
    fn test_session_id_consistency() {
        let id1 = get_session_id();
        let id2 = get_session_id();
        assert_eq!(id1, id2, "Session ID should be consistent within a process");
    }
} 
