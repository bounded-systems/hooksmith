use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::io::BufRead;

/// Schema-validated event structure for all CLI output
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AutoPushEvent {
    /// Event timestamp in RFC3339 format
    #[serde(with = "chrono::serde::ts_seconds")]
    pub timestamp: DateTime<Utc>,
    /// Tool that generated the event (e.g., "git", "cargo-clippy", "hooksmith")
    pub tool: String,
    /// Action being performed (e.g., "commit", "validation", "build")
    pub action: String,
    /// Event level: "info", "warn", "error"
    pub level: String,
    /// Human-readable message
    pub message: String,
    /// Optional details (JSON object)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
    /// Optional error code (for diagnostics)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    /// Optional file path (for diagnostics)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    /// Optional line number (for diagnostics)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    /// Optional column number (for diagnostics)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<u32>,
    /// Session ID for grouping related events
    pub session_id: String,
}

impl AutoPushEvent {
    /// Create a new event with required fields
    pub fn new(
        tool: impl Into<String>,
        action: impl Into<String>,
        level: impl Into<String>,
        message: impl Into<String>,
        session_id: String,
    ) -> Self {
        Self {
            timestamp: Utc::now(),
            tool: tool.into(),
            action: action.into(),
            level: level.into(),
            message: message.into(),
            details: None,
            code: None,
            file: None,
            line: None,
            column: None,
            session_id,
        }
    }

    /// Add optional details
    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }

    /// Add diagnostic information
    pub fn with_diagnostic(
        mut self,
        code: impl Into<String>,
        file: impl Into<String>,
        line: u32,
        column: u32,
    ) -> Self {
        self.code = Some(code.into());
        self.file = Some(file.into());
        self.line = Some(line);
        self.column = Some(column);
        self
    }

    /// Convert to JSONL format (single line)
    pub fn to_jsonl(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// Print to stdout in JSONL format
    pub fn print_jsonl(&self) -> Result<(), serde_json::Error> {
        println!("{}", self.to_jsonl()?);
        Ok(())
    }
}

/// Generate JSON Schema for AutoPushEvent
pub fn generate_schema() -> Result<String, serde_json::Error> {
    // Create a manual schema since we can't derive JsonSchema for DateTime<Utc>
    let schema = serde_json::json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        "type": "object",
        "title": "AutoPushEvent",
        "properties": {
            "timestamp": {
                "type": "string",
                "format": "date-time",
                "description": "Event timestamp in RFC3339 format"
            },
            "tool": {
                "type": "string",
                "description": "Tool that generated the event"
            },
            "action": {
                "type": "string",
                "description": "Action being performed"
            },
            "level": {
                "type": "string",
                "enum": ["info", "warn", "error"],
                "description": "Event level"
            },
            "message": {
                "type": "string",
                "description": "Human-readable message"
            },
            "details": {
                "type": "object",
                "description": "Optional details (JSON object)"
            },
            "code": {
                "type": "string",
                "description": "Optional error code (for diagnostics)"
            },
            "file": {
                "type": "string",
                "description": "Optional file path (for diagnostics)"
            },
            "line": {
                "type": "integer",
                "description": "Optional line number (for diagnostics)"
            },
            "column": {
                "type": "integer",
                "description": "Optional column number (for diagnostics)"
            },
            "sessionId": {
                "type": "string",
                "description": "Session ID for grouping related events"
            }
        },
        "required": ["timestamp", "tool", "action", "level", "message", "sessionId"],
        "additionalProperties": false
    });
    serde_json::to_string_pretty(&schema)
}

/// Validate JSON against the AutoPushEvent schema
pub fn validate_json(json_str: &str) -> Result<bool, Box<dyn std::error::Error>> {
    // For now, just validate that it's valid JSON and has required fields
    let value: serde_json::Value = serde_json::from_str(json_str)?;

    // Check required fields
    if !value.is_object() {
        return Ok(false);
    }

    let obj = value.as_object().unwrap();
    let required_fields = [
        "timestamp",
        "tool",
        "action",
        "level",
        "message",
        "sessionId",
    ];

    for field in &required_fields {
        if !obj.contains_key(*field) {
            return Ok(false);
        }
    }

    // Check level enum
    if let Some(level) = obj.get("level") {
        if let Some(level_str) = level.as_str() {
            if !["info", "warn", "error"].contains(&level_str) {
                return Ok(false);
            }
        } else {
            return Ok(false);
        }
    }

    Ok(true)
}

/// Validate a stream of JSONL events
pub fn validate_jsonl_stream<R: std::io::Read>(
    reader: R,
) -> Result<(), Box<dyn std::error::Error>> {
    let lines = std::io::BufReader::new(reader).lines();

    for (line_num, line) in lines.enumerate() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        if !validate_json(&line)? {
            return Err(format!("Invalid JSON at line {}: {}", line_num + 1, line).into());
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_event_creation() {
        let session_id = Uuid::new_v4().to_string();
        let event = AutoPushEvent::new(
            "hooksmith",
            "validation",
            "info",
            "Test message",
            session_id.clone(),
        );

        assert_eq!(event.tool, "hooksmith");
        assert_eq!(event.action, "validation");
        assert_eq!(event.level, "info");
        assert_eq!(event.message, "Test message");
        assert_eq!(event.session_id, session_id);
    }

    #[test]
    fn test_event_with_details() {
        let session_id = Uuid::new_v4().to_string();
        let event = AutoPushEvent::new("cargo", "build", "info", "Build completed", session_id)
            .with_details(serde_json::json!({
                "duration_ms": 1500,
                "targets": ["release"]
            }));

        assert!(event.details.is_some());
        let details = event.details.unwrap();
        assert_eq!(details["duration_ms"], 1500);
        assert_eq!(details["targets"][0], "release");
    }

    #[test]
    fn test_event_with_diagnostic() {
        let session_id = Uuid::new_v4().to_string();
        let event = AutoPushEvent::new("cargo", "clippy", "warn", "Unused variable", session_id)
            .with_diagnostic("unused_variables", "src/main.rs", 42, 10);

        assert_eq!(event.code, Some("unused_variables".to_string()));
        assert_eq!(event.file, Some("src/main.rs".to_string()));
        assert_eq!(event.line, Some(42));
        assert_eq!(event.column, Some(10));
    }

    #[test]
    fn test_jsonl_serialization() {
        let session_id = Uuid::new_v4().to_string();
        let event = AutoPushEvent::new("hooksmith", "test", "info", "Test event", session_id);

        let jsonl = event.to_jsonl().unwrap();
        let parsed: AutoPushEvent = serde_json::from_str(&jsonl).unwrap();

        assert_eq!(parsed.tool, "hooksmith");
        assert_eq!(parsed.action, "test");
        assert_eq!(parsed.level, "info");
        assert_eq!(parsed.message, "Test event");
    }

    #[test]
    fn test_schema_generation() {
        let schema = generate_schema().unwrap();
        assert!(schema.contains("AutoPushEvent"));
        assert!(schema.contains("timestamp"));
        assert!(schema.contains("tool"));
        assert!(schema.contains("action"));
        assert!(schema.contains("level"));
        assert!(schema.contains("message"));
        assert!(schema.contains("sessionId"));
    }

    #[test]
    fn test_json_validation() {
        let session_id = Uuid::new_v4().to_string();
        let event =
            AutoPushEvent::new("hooksmith", "validation", "info", "Valid event", session_id);

        let jsonl = event.to_jsonl().unwrap();
        assert!(validate_json(&jsonl).unwrap());

        // Test invalid JSON
        assert!(!validate_json("invalid json").unwrap());
    }
}
