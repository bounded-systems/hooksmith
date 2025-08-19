//! Event Bus Manager for Hooksmith Orchestrator
//!
//! This module provides event-driven communication between WIT components
//! and native handlers, integrating with the existing orchestrator system.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::components::ComponentHandle;
use event_types::{EventHandler as XtaskEventHandler, HooksmithEvent};

/// Result of a WASM component call
#[derive(Debug, Clone)]
pub struct WasmCallResult {
    /// Whether the call was successful
    pub success: bool,
    /// Output from the component call
    pub output: Option<String>,
    /// Error message if the call failed
    pub error: Option<String>,
    /// Duration of the call in milliseconds
    pub duration_ms: u64,
}

/// Event registry configuration loaded from JSON
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventRegistryConfig {
    /// Registered events
    pub events: HashMap<String, EventDefinition>,
    /// Registered handlers
    pub handlers: HashMap<String, HandlerDefinition>,
}

/// Definition of an event type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventDefinition {
    /// Handler name for this event
    pub handler: String,
    /// Schema path for validation
    pub schema: String,
    /// Event category
    pub category: String,
    /// Event description
    pub description: String,
}

/// Definition of an event handler
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandlerDefinition {
    /// Type of handler
    pub handler_type: HandlerType,
    /// Component name for WIT handlers
    pub component: Option<String>,
    /// Crate name for native handlers
    pub crate_name: Option<String>,
    /// Supported event types
    pub events: Vec<String>,
    /// Handler description
    pub description: String,
}

/// Type of event handler
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HandlerType {
    /// WIT component handler
    #[serde(rename = "wit")]
    Wit,
    /// Native Rust handler
    #[serde(rename = "native")]
    Native,
}

/// Event subscription for tracking event routing
#[derive(Debug, Clone)]
pub struct EventSubscription {
    /// Name of the subscribing component
    pub component_name: String,
    /// Types of events subscribed to
    pub event_types: Vec<String>,
    /// Unique subscription ID
    pub subscription_id: String,
}

/// Event bus manager for the orchestrator
pub struct EventBusManager {
    /// Event registry loaded from configuration
    registry: EventRegistryConfig,
    /// Active event subscriptions
    subscriptions: Arc<RwLock<HashMap<String, EventSubscription>>>,
    /// WIT component handles
    components: Arc<RwLock<HashMap<String, ComponentHandle>>>,
    /// Native event handlers
    native_handlers: Arc<RwLock<HashMap<String, Box<dyn XtaskEventHandler>>>>,
}

impl EventBusManager {
    /// Create a new event bus manager
    pub fn new(registry_path: &str) -> Result<Self> {
        let registry_content = std::fs::read_to_string(registry_path)?;
        let registry: EventRegistryConfig = serde_json::from_str(&registry_content)?;

        Ok(Self {
            registry,
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            components: Arc::new(RwLock::new(HashMap::new())),
            native_handlers: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Route an event to the appropriate handler
    pub async fn route_event(&self, event: HooksmithEvent) -> Result<()> {
        // For now, just log the event since we don't have a proper registry
        tracing::info!("Routing event: {:?}", event);

        // TODO: Implement proper event routing based on registry
        // This is a simplified implementation

        Ok(())
    }

    /// Route event to WIT component
    #[allow(dead_code)]
    async fn route_to_wit_component(
        &self,
        event: HooksmithEvent,
        handler_def: &HandlerDefinition,
    ) -> Result<()> {
        let component_name = handler_def
            .component
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("WIT handler missing component name"))?;

        let components = self.components.read().await;
        let component = components
            .get(component_name)
            .ok_or_else(|| anyhow::anyhow!("WIT component not found: {}", component_name))?;

        // Call the component's event handler function
        let result = component.call("handle_event", event).await?;

        // Emit the result event back to the event bus
        tracing::info!("Component result: {:?}", result);

        Ok(())
    }

    /// Route event to native handler
    #[allow(dead_code)]
    async fn route_to_native_handler(
        &self,
        event: HooksmithEvent,
        handler_def: &HandlerDefinition,
    ) -> Result<()> {
        let handler_name = handler_def
            .crate_name
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Native handler missing crate name"))?;

        let handlers = self.native_handlers.read().await;
        let _handler = handlers
            .get(handler_name)
            .ok_or_else(|| anyhow::anyhow!("Native handler not found: {}", handler_name))?;

        // Handle the event using the native handler
        // TODO: Convert HooksmithEvent to Event for handler
        tracing::info!("Native handler called for event: {:?}", event);

        Ok(())
    }

    /// Register a WIT component
    pub async fn register_component(&self, name: String, component: ComponentHandle) {
        let mut components = self.components.write().await;
        components.insert(name, component);
    }

    /// Register a native handler
    pub async fn register_native_handler(&self, name: String, handler: Box<dyn XtaskEventHandler>) {
        let mut handlers = self.native_handlers.write().await;
        handlers.insert(name, handler);
    }

    /// Subscribe to events from a component
    pub async fn subscribe_to_events(
        &self,
        component_name: &str,
        event_types: Vec<String>,
    ) -> Result<String> {
        let subscription_id = uuid::Uuid::new_v4().to_string();

        let subscription = EventSubscription {
            component_name: component_name.to_string(),
            event_types,
            subscription_id: subscription_id.clone(),
        };

        let mut subscriptions = self.subscriptions.write().await;
        subscriptions.insert(subscription_id.clone(), subscription);

        Ok(subscription_id)
    }

    /// Unsubscribe from events
    pub async fn unsubscribe(&self, subscription_id: &str) -> Result<()> {
        let mut subscriptions = self.subscriptions.write().await;
        subscriptions.remove(subscription_id);
        Ok(())
    }

    /// Get handler for an event type
    pub fn get_handler_for_event(&self, event_type: &str) -> Result<&HandlerDefinition> {
        let event_def = self
            .registry
            .events
            .get(event_type)
            .ok_or_else(|| anyhow::anyhow!("Unknown event type: {}", event_type))?;

        let handler_name = &event_def.handler;
        self.registry
            .handlers
            .get(handler_name)
            .ok_or_else(|| anyhow::anyhow!("Handler not found: {}", handler_name))
    }

    /// List all registered events
    pub fn list_events(&self) -> Vec<String> {
        self.registry.events.keys().cloned().collect()
    }

    /// List all registered handlers
    pub fn list_handlers(&self) -> Vec<String> {
        self.registry.handlers.keys().cloned().collect()
    }

    /// Get event definition
    pub fn get_event_definition(&self, event_type: &str) -> Option<&EventDefinition> {
        self.registry.events.get(event_type)
    }

    /// Get handler definition
    pub fn get_handler_definition(&self, handler_name: &str) -> Option<&HandlerDefinition> {
        self.registry.handlers.get(handler_name)
    }

    /// Create a result event from a component response
    #[allow(dead_code)]
    fn create_result_event(
        &self,
        _original_event: &HooksmithEvent,
        _result: &WasmCallResult,
    ) -> Option<HooksmithEvent> {
        // For now, return None since we don't have proper result event creation
        // TODO: Implement proper result event creation based on original event type
        None
    }

    /// Validate event against schema
    pub fn validate_event(&self, event: &HooksmithEvent) -> Result<()> {
        // For now, just log the event since we don't have a proper registry
        tracing::info!("Validating event: {:?}", event);

        // TODO: Implement proper event validation based on registry
        // This is a simplified implementation

        Ok(())
    }

    /// Get statistics about event routing
    pub async fn get_statistics(&self) -> EventBusStatistics {
        let subscriptions = self.subscriptions.read().await;
        let components = self.components.read().await;
        let handlers = self.native_handlers.read().await;

        EventBusStatistics {
            total_events: self.registry.events.len(),
            total_handlers: self.registry.handlers.len(),
            active_subscriptions: subscriptions.len(),
            registered_components: components.len(),
            registered_native_handlers: handlers.len(),
        }
    }
}

/// Statistics about the event bus
#[derive(Debug, Clone, Serialize)]
pub struct EventBusStatistics {
    /// Total number of registered events
    pub total_events: usize,
    /// Total number of registered handlers
    pub total_handlers: usize,
    /// Number of active subscriptions
    pub active_subscriptions: usize,
    /// Number of registered WIT components
    pub registered_components: usize,
    /// Number of registered native handlers
    pub registered_native_handlers: usize,
}

/// Event handler trait for native handlers
pub trait EventHandler: Send + Sync {
    /// Handle an event
    fn handle_event(&self, event: &HooksmithEvent) -> Result<()>;

    /// Get handler name
    fn name(&self) -> &str;

    /// Get supported event types
    fn supported_events(&self) -> Vec<String>;
}

/// Event bus manager builder for easy configuration
pub struct EventBusManagerBuilder {
    registry_path: Option<String>,
    auto_load_handlers: bool,
}

impl EventBusManagerBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            registry_path: None,
            auto_load_handlers: false,
        }
    }

    /// Set the registry path
    pub fn registry_path(mut self, path: String) -> Self {
        self.registry_path = Some(path);
        self
    }

    /// Enable auto-loading of handlers
    pub fn auto_load_handlers(mut self, enabled: bool) -> Self {
        self.auto_load_handlers = enabled;
        self
    }

    /// Build the event bus manager
    pub fn build(self) -> Result<EventBusManager> {
        let registry_path = self
            .registry_path
            .unwrap_or_else(|| "crates/xtask/src/config/event-registry.jsonc".to_string());

        EventBusManager::new(&registry_path)
    }
}

impl Default for EventBusManagerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_event_bus_manager_creation() {
        // Create a temporary registry file
        let temp_dir = TempDir::new().unwrap();
        let registry_path = temp_dir.path().join("test-registry.jsonc");

        let registry_content = r#"{
            "events": {
                "test_event": {
                    "handler": "test_handler",
                    "schema": "schemas/test.schema.jsonc",
                    "category": "test",
                    "description": "Test event"
                }
            },
            "handlers": {
                "test_handler": {
                    "handler_type": "native",
                    "crate_name": "test-crate",
                    "events": ["test_event"],
                    "description": "Test handler"
                }
            }
        }"#;

        std::fs::write(&registry_path, registry_content).unwrap();

        // Create event bus manager
        let manager = EventBusManager::new(registry_path.to_str().unwrap()).unwrap();

        // Test basic functionality
        assert_eq!(manager.list_events(), vec!["test_event"]);
        assert_eq!(manager.list_handlers(), vec!["test_handler"]);

        // Test event definition lookup
        let event_def = manager.get_event_definition("test_event").unwrap();
        assert_eq!(event_def.handler, "test_handler");

        // Test handler definition lookup
        let handler_def = manager.get_handler_definition("test_handler").unwrap();
        assert!(matches!(handler_def.handler_type, HandlerType::Native));
    }

    #[test]
    fn test_event_bus_manager_builder() {
        let manager = EventBusManagerBuilder::new()
            .registry_path("crates/xtask/src/config/event-registry.jsonc".to_string())
            .auto_load_handlers(true)
            .build();

        // Should fail because the registry file doesn't exist in test
        assert!(manager.is_err());
    }
}
