//! Schema generation and RPC server for git-operations
//!
//! This module provides JSON schema generation and a Warp-based RPC server
//! to expose the git-operations API in a structured way.

use schemars::schema_for;
use serde_json::Value;
use warp::Filter;

use crate::{
    GitCommitRequest, GitCommitResult, GitOperationError, GitOperationEvent, GitPullRequest,
    GitPullResult, GitPushRequest, GitPushResult, GitStatusRequest, GitStatusResult,
    RepositoryInfo, WorktreeCreateRequest, WorktreeCreateResult, WorktreeListRequest,
    WorktreeListResult, WorktreeRemoveRequest, WorktreeRemoveResult, WorktreeSwitchRequest,
    WorktreeSwitchResult,
};

/// Generate JSON schema for the main API types
pub fn generate_api_schema() -> Value {
    let mut schema = serde_json::Map::new();

    // Add event schemas
    schema.insert(
        "GitOperationEvent".to_string(),
        serde_json::to_value(schema_for!(GitOperationEvent)).unwrap(),
    );

    // Add request schemas
    schema.insert(
        "GitCommitRequest".to_string(),
        serde_json::to_value(schema_for!(GitCommitRequest)).unwrap(),
    );
    schema.insert(
        "GitPushRequest".to_string(),
        serde_json::to_value(schema_for!(GitPushRequest)).unwrap(),
    );
    schema.insert(
        "GitPullRequest".to_string(),
        serde_json::to_value(schema_for!(GitPullRequest)).unwrap(),
    );
    schema.insert(
        "GitStatusRequest".to_string(),
        serde_json::to_value(schema_for!(GitStatusRequest)).unwrap(),
    );
    schema.insert(
        "WorktreeCreateRequest".to_string(),
        serde_json::to_value(schema_for!(WorktreeCreateRequest)).unwrap(),
    );
    schema.insert(
        "WorktreeSwitchRequest".to_string(),
        serde_json::to_value(schema_for!(WorktreeSwitchRequest)).unwrap(),
    );
    schema.insert(
        "WorktreeRemoveRequest".to_string(),
        serde_json::to_value(schema_for!(WorktreeRemoveRequest)).unwrap(),
    );
    schema.insert(
        "WorktreeListRequest".to_string(),
        serde_json::to_value(schema_for!(WorktreeListRequest)).unwrap(),
    );

    // Add result schemas
    schema.insert(
        "GitCommitResult".to_string(),
        serde_json::to_value(schema_for!(GitCommitResult)).unwrap(),
    );
    schema.insert(
        "GitPushResult".to_string(),
        serde_json::to_value(schema_for!(GitPushResult)).unwrap(),
    );
    schema.insert(
        "GitPullResult".to_string(),
        serde_json::to_value(schema_for!(GitPullResult)).unwrap(),
    );
    schema.insert(
        "GitStatusResult".to_string(),
        serde_json::to_value(schema_for!(GitStatusResult)).unwrap(),
    );
    schema.insert(
        "WorktreeCreateResult".to_string(),
        serde_json::to_value(schema_for!(WorktreeCreateResult)).unwrap(),
    );
    schema.insert(
        "WorktreeSwitchResult".to_string(),
        serde_json::to_value(schema_for!(WorktreeSwitchResult)).unwrap(),
    );
    schema.insert(
        "WorktreeRemoveResult".to_string(),
        serde_json::to_value(schema_for!(WorktreeRemoveResult)).unwrap(),
    );
    schema.insert(
        "WorktreeListResult".to_string(),
        serde_json::to_value(schema_for!(WorktreeListResult)).unwrap(),
    );

    // Add supporting types
    schema.insert(
        "RepositoryInfo".to_string(),
        serde_json::to_value(schema_for!(RepositoryInfo)).unwrap(),
    );
    schema.insert(
        "GitOperationError".to_string(),
        serde_json::to_value(schema_for!(GitOperationError)).unwrap(),
    );

    // Add API info
    let mut api_info = serde_json::Map::new();
    api_info.insert(
        "name".to_string(),
        serde_json::Value::String("git-operations".to_string()),
    );
    api_info.insert(
        "version".to_string(),
        serde_json::Value::String("0.1.0".to_string()),
    );
    api_info.insert(
        "description".to_string(),
        serde_json::Value::String("Git operations API for Hooksmith".to_string()),
    );
    api_info.insert(
        "endpoints".to_string(),
        serde_json::json!([
            "POST /api/git/commit",
            "POST /api/git/push",
            "POST /api/git/pull",
            "POST /api/git/status",
            "POST /api/worktree/create",
            "POST /api/worktree/switch",
            "POST /api/worktree/remove",
            "POST /api/worktree/list"
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
            "service": "git-operations",
            "version": "0.1.0"
        }))
    });

    // API info endpoint
    let info_route = warp::path("info").and(warp::get()).map(|| {
        warp::reply::json(&serde_json::json!({
            "name": "git-operations",
            "version": "0.1.0",
            "description": "Native Git operations handler for Hooksmith hybrid architecture",
            "category": "git",
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

    println!("🔧 Git Operations RPC Server");
    println!("============================");
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
        assert!(schema_obj.contains_key("GitOperationEvent"));
        assert!(schema_obj.contains_key("GitCommitRequest"));
        assert!(schema_obj.contains_key("api_info"));

        // Verify schema can be serialized
        let schema_json = serde_json::to_string_pretty(&schema).unwrap();
        assert!(!schema_json.is_empty());
        assert!(schema_json.contains("GitOperationEvent"));
    }
}
