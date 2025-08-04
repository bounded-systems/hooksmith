//! Schema Registry Service for Hooksmith
//!
//! This module provides a centralized schema registry that can discover
//! and aggregate schemas from all RPC-enabled components in the Hooksmith
//! hybrid architecture.

use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::timeout;

/// Schema endpoint information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaEndpoint {
    /// Component name
    pub name: String,
    /// Schema endpoint URL
    pub url: String,
    /// Component category
    pub category: String,
    /// Component description
    pub description: String,
    /// Whether the endpoint is currently accessible
    pub accessible: bool,
    /// Last check timestamp
    pub last_check: Option<chrono::DateTime<chrono::Utc>>,
}

/// Schema registry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaRegistryConfig {
    /// Default timeout for schema requests (seconds)
    pub timeout_seconds: u64,
    /// Retry attempts for failed requests
    pub retry_attempts: u32,
    /// Retry delay between attempts (seconds)
    pub retry_delay_seconds: u64,
}

impl Default for SchemaRegistryConfig {
    fn default() -> Self {
        Self {
            timeout_seconds: 5,
            retry_attempts: 3,
            retry_delay_seconds: 1,
        }
    }
}

/// Schema registry service
pub struct SchemaRegistry {
    config: SchemaRegistryConfig,
    client: Client,
    endpoints: HashMap<String, SchemaEndpoint>,
}

impl SchemaRegistry {
    /// Create a new schema registry
    pub fn new(config: SchemaRegistryConfig) -> Self {
        Self {
            client: Client::new(),
            endpoints: HashMap::new(),
            config,
        }
    }

    /// Add a schema endpoint
    pub fn add_endpoint(&mut self, endpoint: SchemaEndpoint) {
        self.endpoints.insert(endpoint.name.clone(), endpoint);
    }

    /// Discover available schema endpoints
    pub async fn discover_endpoints(&mut self) -> Result<Vec<SchemaEndpoint>> {
        let mut discovered = Vec::new();

        // Define known endpoints based on component registry
        let known_endpoints = vec![
            SchemaEndpoint {
                name: "file-operations".to_string(),
                url: "http://127.0.0.1:3030/schema".to_string(),
                category: "system".to_string(),
                description: "File operations API".to_string(),
                accessible: false,
                last_check: None,
            },
            SchemaEndpoint {
                name: "git-operations".to_string(),
                url: "http://127.0.0.1:3031/schema".to_string(),
                category: "git".to_string(),
                description: "Git operations API".to_string(),
                accessible: false,
                last_check: None,
            },
            SchemaEndpoint {
                name: "lefthook-rs".to_string(),
                url: "http://127.0.0.1:3032/schema".to_string(),
                category: "integration".to_string(),
                description: "Lefthook integration API".to_string(),
                accessible: false,
                last_check: None,
            },
        ];

        for mut endpoint in known_endpoints {
            // Check if endpoint is accessible
            match self.check_endpoint_health(&endpoint.url).await {
                Ok(true) => {
                    endpoint.accessible = true;
                    endpoint.last_check = Some(chrono::Utc::now());
                }
                Ok(false) => {
                    endpoint.accessible = false;
                    endpoint.last_check = Some(chrono::Utc::now());
                }
                Err(_) => {
                    endpoint.accessible = false;
                    endpoint.last_check = Some(chrono::Utc::now());
                }
            }

            self.endpoints
                .insert(endpoint.name.clone(), endpoint.clone());
            discovered.push(endpoint);
        }

        Ok(discovered)
    }

    /// Check if an endpoint is healthy
    async fn check_endpoint_health(&self, base_url: &str) -> Result<bool> {
        let health_url = base_url.replace("/schema", "/health");

        let timeout_duration = Duration::from_secs(self.config.timeout_seconds);

        match timeout(timeout_duration, self.client.get(&health_url).send()).await {
            Ok(Ok(response)) => Ok(response.status().is_success()),
            _ => Ok(false),
        }
    }

    /// Fetch schema from an endpoint
    pub async fn fetch_schema(&self, endpoint_name: &str) -> Result<Value> {
        let endpoint = self
            .endpoints
            .get(endpoint_name)
            .context(format!("Endpoint '{}' not found", endpoint_name))?;

        if !endpoint.accessible {
            return Err(anyhow::anyhow!(
                "Endpoint '{}' is not accessible",
                endpoint_name
            ));
        }

        let timeout_duration = Duration::from_secs(self.config.timeout_seconds);

        for attempt in 1..=self.config.retry_attempts {
            match timeout(timeout_duration, self.client.get(&endpoint.url).send()).await {
                Ok(Ok(response)) => {
                    if response.status().is_success() {
                        let schema: Value = response
                            .json()
                            .await
                            .context("Failed to parse schema JSON")?;
                        return Ok(schema);
                    } else {
                        return Err(anyhow::anyhow!(
                            "HTTP error {} for endpoint '{}'",
                            response.status(),
                            endpoint_name
                        ));
                    }
                }
                Ok(Err(e)) => {
                    if attempt == self.config.retry_attempts {
                        return Err(anyhow::anyhow!(
                            "Request failed for endpoint '{}': {}",
                            endpoint_name,
                            e
                        ));
                    }
                }
                Err(_) => {
                    if attempt == self.config.retry_attempts {
                        return Err(anyhow::anyhow!(
                            "Request timeout for endpoint '{}'",
                            endpoint_name
                        ));
                    }
                }
            }

            if attempt < self.config.retry_attempts {
                tokio::time::sleep(Duration::from_secs(self.config.retry_delay_seconds)).await;
            }
        }

        Err(anyhow::anyhow!(
            "Failed to fetch schema from endpoint '{}' after {} attempts",
            endpoint_name,
            self.config.retry_attempts
        ))
    }

    /// Fetch all available schemas
    pub async fn fetch_all_schemas(&self) -> Result<HashMap<String, Value>> {
        let mut schemas = HashMap::new();

        for (name, endpoint) in &self.endpoints {
            if endpoint.accessible {
                match self.fetch_schema(name).await {
                    Ok(schema) => {
                        schemas.insert(name.clone(), schema);
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to fetch schema for '{}': {}", name, e);
                    }
                }
            }
        }

        Ok(schemas)
    }

    /// Generate a combined schema registry
    pub async fn generate_registry_schema(&self) -> Result<Value> {
        let schemas = self.fetch_all_schemas().await?;

        let mut registry = serde_json::Map::new();

        // Add registry metadata
        let mut metadata = serde_json::Map::new();
        metadata.insert(
            "name".to_string(),
            serde_json::Value::String("hooksmith-schema-registry".to_string()),
        );
        metadata.insert(
            "version".to_string(),
            serde_json::Value::String("0.1.0".to_string()),
        );
        metadata.insert(
            "description".to_string(),
            serde_json::Value::String(
                "Combined schema registry for Hooksmith components".to_string(),
            ),
        );
        metadata.insert(
            "generated_at".to_string(),
            serde_json::Value::String(chrono::Utc::now().to_rfc3339()),
        );

        registry.insert("metadata".to_string(), serde_json::Value::Object(metadata));

        // Add component schemas
        registry.insert(
            "schemas".to_string(),
            serde_json::Value::Object(serde_json::Map::from_iter(schemas)),
        );

        // Add endpoint information
        let endpoints: Vec<Value> = self
            .endpoints
            .values()
            .map(|e| serde_json::to_value(e).unwrap())
            .collect();
        registry.insert("endpoints".to_string(), serde_json::Value::Array(endpoints));

        Ok(serde_json::Value::Object(registry))
    }

    /// Get endpoint status summary
    pub fn get_status_summary(&self) -> Value {
        let mut summary = serde_json::Map::new();

        let total = self.endpoints.len();
        let accessible = self.endpoints.values().filter(|e| e.accessible).count();
        let inaccessible = total - accessible;

        summary.insert(
            "total_endpoints".to_string(),
            serde_json::Value::Number(total.into()),
        );
        summary.insert(
            "accessible_endpoints".to_string(),
            serde_json::Value::Number(accessible.into()),
        );
        summary.insert(
            "inaccessible_endpoints".to_string(),
            serde_json::Value::Number(inaccessible.into()),
        );

        let endpoints: Vec<Value> = self
            .endpoints
            .values()
            .map(|e| {
                let mut endpoint = serde_json::Map::new();
                endpoint.insert(
                    "name".to_string(),
                    serde_json::Value::String(e.name.clone()),
                );
                endpoint.insert(
                    "category".to_string(),
                    serde_json::Value::String(e.category.clone()),
                );
                endpoint.insert(
                    "accessible".to_string(),
                    serde_json::Value::Bool(e.accessible),
                );
                endpoint.insert("url".to_string(), serde_json::Value::String(e.url.clone()));
                serde_json::Value::Object(endpoint)
            })
            .collect();

        summary.insert("endpoints".to_string(), serde_json::Value::Array(endpoints));

        serde_json::Value::Object(summary)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_schema_registry_creation() {
        let config = SchemaRegistryConfig::default();
        let registry = SchemaRegistry::new(config);

        assert_eq!(registry.endpoints.len(), 0);
    }

    #[tokio::test]
    async fn test_endpoint_discovery() {
        let config = SchemaRegistryConfig::default();
        let mut registry = SchemaRegistry::new(config);

        let endpoints = registry.discover_endpoints().await.unwrap();
        assert!(!endpoints.is_empty());

        // All endpoints should be marked as inaccessible in test environment
        for endpoint in endpoints {
            assert!(!endpoint.accessible);
        }
    }
}
