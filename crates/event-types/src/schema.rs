//! Shared schema generation utilities for event-types
//!
//! This module provides JSON schema generation for the shared event types
//! used across the Hooksmith hybrid architecture.

use schemars::schema_for;
use serde_json::Value;

use crate::{
    ComputationEvent, ContractCheckConfig, ContractCheckResult, Event, EventMetadata,
    EventPriority, HooksmithEvent, PolicyDecision, PolicyEvaluationResult, SystemEvent,
    ValidationConfig, ValidationResult, Violation, ViolationSeverity,
};

/// Generate JSON schema for all event types
pub fn generate_event_schemas() -> Value {
    let mut schema = serde_json::Map::new();

    // Add event type schemas
    schema.insert(
        "SystemEvent".to_string(),
        serde_json::to_value(schema_for!(SystemEvent)).unwrap(),
    );
    schema.insert(
        "ComputationEvent".to_string(),
        serde_json::to_value(schema_for!(ComputationEvent)).unwrap(),
    );
    schema.insert(
        "HooksmithEvent".to_string(),
        serde_json::to_value(schema_for!(HooksmithEvent)).unwrap(),
    );
    schema.insert(
        "Event".to_string(),
        serde_json::to_value(schema_for!(Event)).unwrap(),
    );
    schema.insert(
        "EventMetadata".to_string(),
        serde_json::to_value(schema_for!(EventMetadata)).unwrap(),
    );
    schema.insert(
        "EventPriority".to_string(),
        serde_json::to_value(schema_for!(EventPriority)).unwrap(),
    );

    // Add validation schemas
    schema.insert(
        "ValidationConfig".to_string(),
        serde_json::to_value(schema_for!(ValidationConfig)).unwrap(),
    );
    schema.insert(
        "ValidationResult".to_string(),
        serde_json::to_value(schema_for!(ValidationResult)).unwrap(),
    );
    schema.insert(
        "ContractCheckConfig".to_string(),
        serde_json::to_value(schema_for!(ContractCheckConfig)).unwrap(),
    );
    schema.insert(
        "ContractCheckResult".to_string(),
        serde_json::to_value(schema_for!(ContractCheckResult)).unwrap(),
    );
    schema.insert(
        "Violation".to_string(),
        serde_json::to_value(schema_for!(Violation)).unwrap(),
    );
    schema.insert(
        "ViolationSeverity".to_string(),
        serde_json::to_value(schema_for!(ViolationSeverity)).unwrap(),
    );
    schema.insert(
        "PolicyEvaluationResult".to_string(),
        serde_json::to_value(schema_for!(PolicyEvaluationResult)).unwrap(),
    );
    schema.insert(
        "PolicyDecision".to_string(),
        serde_json::to_value(schema_for!(PolicyDecision)).unwrap(),
    );

    // Add API info
    let mut api_info = serde_json::Map::new();
    api_info.insert(
        "name".to_string(),
        serde_json::Value::String("event-types".to_string()),
    );
    api_info.insert(
        "version".to_string(),
        serde_json::Value::String("0.1.0".to_string()),
    );
    api_info.insert(
        "description".to_string(),
        serde_json::Value::String(
            "Shared event types for Hooksmith hybrid architecture".to_string(),
        ),
    );
    api_info.insert(
        "event_types".to_string(),
        serde_json::json!([
            "SystemEvent",
            "ComputationEvent",
            "HooksmithEvent",
            "Event",
            "EventMetadata",
            "EventPriority"
        ]),
    );

    schema.insert("api_info".to_string(), serde_json::Value::Object(api_info));

    serde_json::Value::Object(schema)
}

/// Generate a minimal schema for a specific event type
pub fn generate_event_schema<T: serde::Serialize + schemars::JsonSchema>() -> Value {
    serde_json::to_value(schema_for!(T)).unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_schema_generation() {
        let schema = generate_event_schemas();
        assert!(schema.is_object());

        let schema_obj = schema.as_object().unwrap();
        assert!(schema_obj.contains_key("FileOperationEvent"));
        assert!(schema_obj.contains_key("GitOperationEvent"));
        assert!(schema_obj.contains_key("api_info"));

        // Verify schema can be serialized
        let schema_json = serde_json::to_string_pretty(&schema).unwrap();
        assert!(!schema_json.is_empty());
        assert!(schema_json.contains("FileOperationEvent"));
    }
}
