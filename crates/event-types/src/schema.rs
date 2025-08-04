//! Shared schema generation utilities for event-types
//!
//! This module provides JSON schema generation for the shared event types
//! used across the Hooksmith hybrid architecture.

use schemars::schema_for;
use serde_json::Value;

use crate::{
    ContractValidationEvent, ContractValidationRequest, ContractValidationResult,
    FileOperationEvent, GitOperationEvent, HookBuilderEvent, ValidationHandlerEvent,
    WorktreeRunnerEvent,
};

/// Generate JSON schema for all event types
pub fn generate_event_schemas() -> Value {
    let mut schema = serde_json::Map::new();

    // Add event type schemas
    schema.insert(
        "FileOperationEvent".to_string(),
        serde_json::to_value(schema_for!(FileOperationEvent)).unwrap(),
    );
    schema.insert(
        "GitOperationEvent".to_string(),
        serde_json::to_value(schema_for!(GitOperationEvent)).unwrap(),
    );
    schema.insert(
        "ContractValidationEvent".to_string(),
        serde_json::to_value(schema_for!(ContractValidationEvent)).unwrap(),
    );
    schema.insert(
        "HookBuilderEvent".to_string(),
        serde_json::to_value(schema_for!(HookBuilderEvent)).unwrap(),
    );
    schema.insert(
        "ValidationHandlerEvent".to_string(),
        serde_json::to_value(schema_for!(ValidationHandlerEvent)).unwrap(),
    );
    schema.insert(
        "WorktreeRunnerEvent".to_string(),
        serde_json::to_value(schema_for!(WorktreeRunnerEvent)).unwrap(),
    );

    // Add request/result schemas
    schema.insert(
        "ContractValidationRequest".to_string(),
        serde_json::to_value(schema_for!(ContractValidationRequest)).unwrap(),
    );
    schema.insert(
        "ContractValidationResult".to_string(),
        serde_json::to_value(schema_for!(ContractValidationResult)).unwrap(),
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
            "FileOperationEvent",
            "GitOperationEvent",
            "ContractValidationEvent",
            "HookBuilderEvent",
            "ValidationHandlerEvent",
            "WorktreeRunnerEvent"
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
