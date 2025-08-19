# RPC Schema System for Hooksmith

## Overview

The RPC Schema System provides dynamic, runtime-accessible JSON schemas for all native components in the Hooksmith hybrid architecture. This system enables:

- **Dynamic Schema Discovery**: Components expose their schemas via HTTP endpoints
- **Runtime Validation**: Schemas can be fetched and used for validation at runtime
- **Centralized Registry**: A unified registry aggregates schemas from all components
- **Health Monitoring**: Endpoint health and accessibility can be monitored

## Architecture

### Component Schema Endpoints

Each native component that supports RPC schemas exposes the following HTTP endpoints:

- `GET /schema` - JSON schema for all API types
- `GET /health` - Health check endpoint
- `GET /info` - API information and metadata

### Schema Registry Service

The centralized schema registry service provides:

- **Endpoint Discovery**: Automatically discovers available schema endpoints
- **Schema Aggregation**: Combines schemas from all accessible endpoints
- **Health Monitoring**: Tracks endpoint accessibility and health status
- **Retry Logic**: Handles transient failures with configurable retry attempts

## Supported Components

### Currently RPC-Enabled

1. **file-operations** (Port 3030)
   - File system operations and event handlers
   - Schema: `http://127.0.0.1:3030/schema`

2. **git-operations** (Port 3031)
   - Git operations and repository management
   - Schema: `http://127.0.0.1:3031/schema`

3. **lefthook-rs** (Port 3032)
   - Lefthook integration and configuration
   - Schema: `http://127.0.0.1:3032/schema`

### Planned RPC Support

- **event-types** - Shared event types (static schemas)
- **cli-core** - Core CLI framework
- **xtask** - Build system and development tools

## Usage

### Starting RPC Servers

Each component can be started as an RPC server:

```bash
# File operations server
cargo run -p file-operations --bin server --port 3030

# Git operations server
cargo run -p git-operations --bin server --port 3031

# Lefthook integration server
cargo run -p lefthook-rs --bin server --port 3032
```

### Using the Schema Registry

The schema registry is accessible via the xtask CLI:

```bash
# Discover available endpoints
cargo run -p xtask -- schema-registry --discover

# Fetch schema from specific endpoint
cargo run -p xtask -- schema-registry --fetch file-operations

# Generate combined registry
cargo run -p xtask -- schema-registry --generate --output schemas/registry.json

# Show endpoint status
cargo run -p xtask -- schema-registry --status
```

### Direct HTTP Access

Schemas can also be accessed directly via HTTP:

```bash
# Get file-operations schema
curl http://127.0.0.1:3030/schema

# Check health status
curl http://127.0.0.1:3030/health

# Get API information
curl http://127.0.0.1:3030/info
```

## Schema Format

### Component Schema Structure

Each component schema follows this structure:

```json
{
  "ComponentName": {
    "$schema": "http://json-schema.org/draft-07/schema#",
    "type": "object",
    "properties": {
      // Component-specific properties
    }
  },
  "RequestType": {
    // Request type schemas
  },
  "ResultType": {
    // Result type schemas
  },
  "api_info": {
    "name": "component-name",
    "version": "0.1.0",
    "description": "Component description",
    "endpoints": [
      "POST /api/endpoint1",
      "GET /api/endpoint2"
    ]
  }
}
```

### Registry Schema Structure

The combined registry schema:

```json
{
  "metadata": {
    "name": "hooksmith-schema-registry",
    "version": "0.1.0",
    "description": "Combined schema registry for Hooksmith components",
    "generated_at": "2024-01-01T00:00:00Z"
  },
  "schemas": {
    "file-operations": {
      // file-operations schema
    },
    "git-operations": {
      // git-operations schema
    },
    "lefthook-rs": {
      // lefthook-rs schema
    }
  },
  "endpoints": [
    {
      "name": "file-operations",
      "url": "http://127.0.0.1:3030/schema",
      "category": "system",
      "description": "File operations API",
      "accessible": true,
      "last_check": "2024-01-01T00:00:00Z"
    }
  ]
}
```

## Implementation Details

### Schema Generation

Schemas are generated using the `schemars` crate:

```rust
use schemars::schema_for;
use serde_json::Value;

pub fn generate_api_schema() -> Value {
    let mut schema = serde_json::Map::new();
    
    // Add type schemas
    schema.insert(
        "ComponentType".to_string(),
        serde_json::to_value(schema_for!(ComponentType)).unwrap(),
    );
    
    // Add API info
    let mut api_info = serde_json::Map::new();
    api_info.insert("name".to_string(), "component-name".into());
    // ... more metadata
    
    schema.insert("api_info".to_string(), serde_json::Value::Object(api_info));
    
    serde_json::Value::Object(schema)
}
```

### RPC Server Setup

Each component uses Warp for HTTP endpoints:

```rust
use warp::Filter;

pub fn create_routes() -> impl Filter<Extract = impl warp::Reply> + Clone {
    let schema_route = warp::path("schema")
        .and(warp::get())
        .map(|| {
            let schema = generate_api_schema();
            warp::reply::json(&schema)
        });

    let health_route = warp::path("health")
        .and(warp::get())
        .map(|| {
            warp::reply::json(&serde_json::json!({
                "status": "healthy",
                "service": "component-name",
                "version": "0.1.0"
            }))
        });

    schema_route
        .or(health_route)
        .with(warp::cors().allow_any_origin())
}
```

### Registry Service

The registry service handles discovery and aggregation:

```rust
pub struct SchemaRegistry {
    config: SchemaRegistryConfig,
    client: Client,
    endpoints: HashMap<String, SchemaEndpoint>,
}

impl SchemaRegistry {
    pub async fn discover_endpoints(&mut self) -> Result<Vec<SchemaEndpoint>> {
        // Discover and check health of known endpoints
    }
    
    pub async fn fetch_schema(&self, endpoint_name: &str) -> Result<Value> {
        // Fetch schema from specific endpoint with retry logic
    }
    
    pub async fn generate_registry_schema(&self) -> Result<Value> {
        // Combine all accessible schemas into registry
    }
}
```

## Configuration

### Component Registry

The component registry (`config/component-registry.jsonc`) defines schema endpoints:

```jsonc
{
  "native_crates": [
    {
      "name": "file-operations",
      "schema_endpoint": "http://127.0.0.1:3030/schema"
    },
    {
      "name": "git-operations", 
      "schema_endpoint": "http://127.0.0.1:3031/schema"
    }
  ]
}
```

### Registry Configuration

Registry behavior can be configured:

```rust
pub struct SchemaRegistryConfig {
    pub timeout_seconds: u64,        // Request timeout
    pub retry_attempts: u32,         // Retry attempts
    pub retry_delay_seconds: u64,    // Delay between retries
}
```

## Health Monitoring

### Endpoint Health Checks

The registry monitors endpoint health:

- **Accessibility**: Whether the endpoint responds to health checks
- **Response Time**: Time taken to respond to requests
- **Last Check**: Timestamp of last health check
- **Error Tracking**: Failed request attempts and reasons

### Health Status

Endpoint status is tracked as:

- ✅ **Accessible**: Endpoint responds successfully
- ❌ **Inaccessible**: Endpoint fails to respond or returns errors

## Error Handling

### Retry Logic

The registry implements configurable retry logic:

1. **Initial Request**: Attempt to fetch schema
2. **Retry Attempts**: Retry failed requests up to configured limit
3. **Backoff Delay**: Wait between retry attempts
4. **Timeout Handling**: Abort requests that exceed timeout

### Error Types

Common error scenarios:

- **Network Errors**: Connection failures, timeouts
- **HTTP Errors**: 4xx/5xx status codes
- **Schema Errors**: Invalid JSON, malformed schemas
- **Registry Errors**: Configuration issues, discovery failures

## Integration Examples

### Runtime Validation

```rust
use serde_json::Value;
use jsonschema::JSONSchema;

async fn validate_with_rpc_schema(data: &Value, component: &str) -> Result<()> {
    let registry = SchemaRegistry::new(Default::default());
    let schema = registry.fetch_schema(component).await?;
    
    let compiled_schema = JSONSchema::compile(&schema)?;
    compiled_schema.validate(data)?;
    
    Ok(())
}
```

### Dynamic API Documentation

```rust
async fn generate_api_docs() -> Result<String> {
    let registry = SchemaRegistry::new(Default::default());
    let registry_schema = registry.generate_registry_schema().await?;
    
    // Generate documentation from combined schemas
    let docs = generate_docs_from_schemas(&registry_schema)?;
    Ok(docs)
}
```

### Health Monitoring Dashboard

```rust
async fn get_health_dashboard() -> Result<Value> {
    let mut registry = SchemaRegistry::new(Default::default());
    registry.discover_endpoints().await?;
    
    let summary = registry.get_status_summary();
    Ok(summary)
}
```

## Future Enhancements

### Planned Features

1. **Schema Versioning**: Support for schema versioning and compatibility
2. **Caching**: Local caching of schemas to reduce network requests
3. **Authentication**: Secure schema endpoints with authentication
4. **WebSocket Support**: Real-time schema updates via WebSocket
5. **Schema Validation**: Validate schemas against meta-schemas
6. **Metrics Collection**: Detailed metrics on schema usage and performance

### Integration Opportunities

1. **IDE Integration**: IDE plugins for schema-aware development
2. **API Testing**: Automated API testing using schemas
3. **Documentation Generation**: Auto-generated API documentation
4. **Contract Testing**: Schema-based contract validation
5. **Monitoring**: Schema drift detection and alerts

## Troubleshooting

### Common Issues

1. **Endpoint Not Accessible**
   - Check if the RPC server is running
   - Verify port configuration
   - Check firewall settings

2. **Schema Fetch Failures**
   - Verify endpoint health
   - Check network connectivity
   - Review retry configuration

3. **Registry Generation Errors**
   - Ensure at least one endpoint is accessible
   - Check schema format validity
   - Review registry configuration

### Debug Commands

```bash
# Check endpoint health
curl -v http://127.0.0.1:3030/health

# Test schema endpoint
curl -v http://127.0.0.1:3030/schema

# Validate registry
cargo run -p xtask -- schema-registry --status --verbose
```

## Conclusion

The RPC Schema System provides a robust foundation for dynamic schema management in the Hooksmith architecture. It enables runtime schema discovery, validation, and monitoring while maintaining the flexibility and performance characteristics of the hybrid WIT + native Rust approach.

For more information, see:
- [Component Registry Documentation](component-registry.md)
- [Schema Validation Guide](schema-validation.md)
- [API Documentation](api.md) 
