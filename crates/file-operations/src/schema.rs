//! Schema generation and RPC server for file-operations
//!
//! This module provides JSON schema generation and a Warp-based RPC server
//! to expose the file-operations API in a structured way.

use schemars::schema_for;
use serde_json::Value;
use warp::Filter;

use crate::{
    FileOperationEvent, FileReadRequest, FileWriteRequest, FileDeleteRequest, FileExistsRequest,
    FileCopyRequest, FileMoveRequest, DirectoryCreateRequest, DirectoryListRequest, FileChecksumRequest,
    FileReadResult, FileWriteResult, FileDeleteResult, FileExistsResult, FileCopyResult, FileMoveResult,
    DirectoryCreateResult, DirectoryListResult, FileChecksumResult,
    FileMetadata, FileInfo, EventMetadata, FileOperationError,
};

/// Generate JSON schema for the main API types
pub fn generate_api_schema() -> Value {
    let mut schema = serde_json::Map::new();
    
    // Add schemas for main types
    schema.insert("FileOperationEvent".to_string(), serde_json::to_value(schema_for!(FileOperationEvent)).unwrap());
    schema.insert("FileReadRequest".to_string(), serde_json::to_value(schema_for!(FileReadRequest)).unwrap());
    schema.insert("FileWriteRequest".to_string(), serde_json::to_value(schema_for!(FileWriteRequest)).unwrap());
    schema.insert("FileDeleteRequest".to_string(), serde_json::to_value(schema_for!(FileDeleteRequest)).unwrap());
    schema.insert("FileExistsRequest".to_string(), serde_json::to_value(schema_for!(FileExistsRequest)).unwrap());
    schema.insert("FileCopyRequest".to_string(), serde_json::to_value(schema_for!(FileCopyRequest)).unwrap());
    schema.insert("FileMoveRequest".to_string(), serde_json::to_value(schema_for!(FileMoveRequest)).unwrap());
    schema.insert("DirectoryCreateRequest".to_string(), serde_json::to_value(schema_for!(DirectoryCreateRequest)).unwrap());
    schema.insert("DirectoryListRequest".to_string(), serde_json::to_value(schema_for!(DirectoryListRequest)).unwrap());
    schema.insert("FileChecksumRequest".to_string(), serde_json::to_value(schema_for!(FileChecksumRequest)).unwrap());
    
    // Add result schemas
    schema.insert("FileReadResult".to_string(), serde_json::to_value(schema_for!(FileReadResult)).unwrap());
    schema.insert("FileWriteResult".to_string(), serde_json::to_value(schema_for!(FileWriteResult)).unwrap());
    schema.insert("FileDeleteResult".to_string(), serde_json::to_value(schema_for!(FileDeleteResult)).unwrap());
    schema.insert("FileExistsResult".to_string(), serde_json::to_value(schema_for!(FileExistsResult)).unwrap());
    schema.insert("FileCopyResult".to_string(), serde_json::to_value(schema_for!(FileCopyResult)).unwrap());
    schema.insert("FileMoveResult".to_string(), serde_json::to_value(schema_for!(FileMoveResult)).unwrap());
    schema.insert("DirectoryCreateResult".to_string(), serde_json::to_value(schema_for!(DirectoryCreateResult)).unwrap());
    schema.insert("DirectoryListResult".to_string(), serde_json::to_value(schema_for!(DirectoryListResult)).unwrap());
    schema.insert("FileChecksumResult".to_string(), serde_json::to_value(schema_for!(FileChecksumResult)).unwrap());
    
    // Add supporting types
    schema.insert("FileMetadata".to_string(), serde_json::to_value(schema_for!(FileMetadata)).unwrap());
    schema.insert("FileInfo".to_string(), serde_json::to_value(schema_for!(FileInfo)).unwrap());
    schema.insert("EventMetadata".to_string(), serde_json::to_value(schema_for!(EventMetadata)).unwrap());
    schema.insert("FileOperationError".to_string(), serde_json::to_value(schema_for!(FileOperationError)).unwrap());
    
    // Add API info
    let mut api_info = serde_json::Map::new();
    api_info.insert("name".to_string(), serde_json::Value::String("file-operations".to_string()));
    api_info.insert("version".to_string(), serde_json::Value::String("0.1.0".to_string()));
    api_info.insert("description".to_string(), serde_json::Value::String("File operations API for Hooksmith".to_string()));
    api_info.insert("endpoints".to_string(), serde_json::json!([
        "POST /api/file/read",
        "POST /api/file/write", 
        "POST /api/file/delete",
        "POST /api/file/exists",
        "POST /api/file/copy",
        "POST /api/file/move",
        "POST /api/directory/create",
        "POST /api/directory/list",
        "POST /api/file/checksum"
    ]));
    
    schema.insert("api_info".to_string(), serde_json::Value::Object(api_info));
    
    serde_json::Value::Object(schema)
}

/// Create Warp routes for the RPC server
pub fn create_routes() -> impl Filter<Extract = impl warp::Reply> + Clone {
    // Schema endpoint
    let schema_route = warp::path("schema")
        .and(warp::get())
        .map(|| {
            let schema = generate_api_schema();
            warp::reply::json(&schema)
        });
    
    // Health check endpoint
    let health_route = warp::path("health")
        .and(warp::get())
        .map(|| {
            warp::reply::json(&serde_json::json!({
                "status": "healthy",
                "service": "file-operations",
                "version": "0.1.0"
            }))
        });
    
    // API info endpoint
    let info_route = warp::path("info")
        .and(warp::get())
        .map(|| {
            warp::reply::json(&serde_json::json!({
                "name": "file-operations",
                "version": "0.1.0",
                "description": "Native file operations handler for Hooksmith hybrid architecture",
                "category": "system",
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
    
    println!("Starting file-operations RPC server on port {}", port);
    println!("Schema available at: http://localhost:{}/schema", port);
    println!("Health check at: http://localhost:{}/health", port);
    println!("API info at: http://localhost:{}/info", port);
    
    warp::serve(routes)
        .run(([127, 0, 0, 1], port))
        .await;
    
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
        assert!(schema_obj.contains_key("FileOperationEvent"));
        assert!(schema_obj.contains_key("FileReadRequest"));
        assert!(schema_obj.contains_key("api_info"));
        
        // Verify schema can be serialized
        let schema_json = serde_json::to_string_pretty(&schema).unwrap();
        assert!(!schema_json.is_empty());
        assert!(schema_json.contains("FileOperationEvent"));
    }
} 
