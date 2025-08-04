//! Schema generation and RPC server for lefthook-rs
//!
//! This module provides JSON schema generation and a Warp-based RPC server
//! to expose the lefthook-rs API in a structured way.

use schemars::schema_for;
use serde_json::Value;
use warp::Filter;

use crate::{
    HookConfig, HookSection, JobConfig, GlobalConfig, LefthookError,
};

/// Generate JSON schema for the main API types
pub fn generate_api_schema() -> Value {
    let mut schema = serde_json::Map::new();

    // Add configuration schemas
    schema.insert(
        "GlobalConfig".to_string(),
        serde_json::to_value(schema_for!(GlobalConfig)).unwrap(),
    );
    schema.insert(
        "HookConfig".to_string(),
        serde_json::to_value(schema_for!(HookConfig)).unwrap(),
    );
    schema.insert(
        "HookSection".to_string(),
        serde_json::to_value(schema_for!(HookSection)).unwrap(),
    );
    schema.insert(
        "JobConfig".to_string(),
        serde_json::to_value(schema_for!(JobConfig)).unwrap(),
    );

    // Add error schema
    schema.insert(
        "LefthookError".to_string(),
        serde_json::to_value(schema_for!(LefthookError)).unwrap(),
    );

    // Add API info
    let mut api_info = serde_json::Map::new();
    api_info.insert(
        "name".to_string(),
        serde_json::Value::String("lefthook-rs".to_string()),
    );
    api_info.insert(
        "version".to_string(),
        serde_json::Value::String("0.1.0".to_string()),
    );
    api_info.insert(
        "description".to_string(),
        serde_json::Value::String("Lefthook integration API for Hooksmith".to_string()),
    );
    api_info.insert(
        "endpoints".to_string(),
        serde_json::json!([
            "GET  /api/config/validate",
            "GET  /api/config/generate",
            "POST /api/config/write",
            "GET  /api/binary/version",
            "POST /api/binary/install"
        ]),
    );

    schema.insert("api_info".to_string(), serde_json::Value::Object(api_info));

    serde_json::Value::Object(schema)
}

/// Create Warp routes for the RPC server
pub fn create_routes() -> impl Filter<Extract = impl warp::Reply> + Clone {
    // Schema endpoint
    let schema_route = warp::path("schema").and(warp::get()).map(|| {
        let schema = generate_api_schema();
        warp::reply::json(&schema)
    });

    // Health check endpoint
    let health_route = warp::path("health").and(warp::get()).map(|| {
        warp::reply::json(&serde_json::json!({
            "status": "healthy",
            "service": "lefthook-rs",
            "version": "0.1.0"
        }))
    });

    // API info endpoint
    let info_route = warp::path("info").and(warp::get()).map(|| {
        warp::reply::json(&serde_json::json!({
            "name": "lefthook-rs",
            "version": "0.1.0",
            "description": "Native Lefthook integration handler for Hooksmith hybrid architecture",
            "category": "integration",
            "schema_endpoint": "/schema",
            "health_endpoint": "/health"
        }))
    });

    // Combine all routes
    schema_route
        .or(health_route)
        .or(info_route)
        .with(warp::cors().allow_any_origin())
}

/// Start the RPC server
pub async fn start_server(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let routes = create_routes();

    println!("🔧 Lefthook Integration RPC Server");
    println!("==================================");
    println!("Starting server on port {}", port);
    println!("Schema available at: http://localhost:{}/schema", port);
    println!("Health check at: http://localhost:{}/health", port);
    println!("API info at: http://localhost:{}/info", port);

    warp::serve(routes).run(([127, 0, 0, 1], port)).await;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_generation() {
        let schema = generate_api_schema();
        assert!(schema.is_object());

        let schema_obj = schema.as_object().unwrap();
        assert!(schema_obj.contains_key("LefthookConfig"));
        assert!(schema_obj.contains_key("HookConfig"));
        assert!(schema_obj.contains_key("api_info"));

        // Verify schema can be serialized
        let schema_json = serde_json::to_string_pretty(&schema).unwrap();
        assert!(!schema_json.is_empty());
        assert!(schema_json.contains("LefthookConfig"));
    }
}
