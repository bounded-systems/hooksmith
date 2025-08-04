//! Event Bus Manager for Hooksmith Orchestrator
//!
//! This module provides event-driven communication between WIT components
//! and native handlers, integrating with the existing orchestrator system.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::xtask::event_bus::{HooksmithEvent, EventHandler as XtaskEventHandler};
use super::components::ComponentHandle;

/// Event registry configuration loaded from JSON
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventRegistryConfig {
    pub events: HashMap<String, EventDefinition>,
    pub handlers: HashMap<String, HandlerDefinition>,
}

/// Definition of an event type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventDefinition {
    pub handler: String,
    pub schema: String,
    pub category: String,
    pub description: String,
}

/// Definition of an event handler
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandlerDefinition {
    pub handler_type: HandlerType,
    pub component: Option<String>,
    pub crate_name: Option<String>,
    pub events: Vec<String>,
    pub description: String,
}

/// Type of event handler
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HandlerType {
    #[serde(rename = "wit")]
    Wit,
    #[serde(rename = "native")]
    Native,
}

/// Event subscription for tracking event routing
#[derive(Debug, Clone)]
pub struct EventSubscription {
    pub component_name: String,
    pub event_types: Vec<String>,
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
        let event_type = &event.event;
        
        // Look up the handler for this event type
        let event_def = self.registry.events.get(event_type)
            .ok_or_else(|| anyhow::anyhow!("Unknown event type: {}", event_type))?;
        
        let handler_name = &event_def.handler;
        let handler_def = self.registry.handlers.get(handler_name)
            .ok_or_else(|| anyhow::anyhow!("Handler not found: {}", handler_name))?;

        match handler_def.handler_type {
            HandlerType::Wit => {
                self.route_to_wit_component(event, handler_def).await
            }
            HandlerType::Native => {
                self.route_to_native_handler(event, handler_def).await
            }
        }
    }

    /// Route event to WIT component
    async fn route_to_wit_component(&self, event: HooksmithEvent, handler_def: &HandlerDefinition) -> Result<()> {
        let component_name = handler_def.component.as_ref()
            .ok_or_else(|| anyhow::anyhow!("WIT handler missing component name"))?;
        
        let components = self.components.read().await;
        let component = components.get(component_name)
            .ok_or_else(|| anyhow::anyhow!("WIT component not found: {}", component_name))?;

        // Call the component's event handler function
        let result = component.call("handle_event", event).await?;
        
        // Emit the result event back to the event bus
        if let Some(result_event) = self.create_result_event(&event, &result) {
            crate::xtask::event_bus::emit_event(result_event)?;
        }

        Ok(())
    }

    /// Route event to native handler
    async fn route_to_native_handler(&self, event: HooksmithEvent, handler_def: &HandlerDefinition) -> Result<()> {
        let handler_name = handler_def.crate_name.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Native handler missing crate name"))?;
        
        let handlers = self.native_handlers.read().await;
        let handler = handlers.get(handler_name)
            .ok_or_else(|| anyhow::anyhow!("Native handler not found: {}", handler_name))?;

        // Handle the event using the native handler
        handler.handle_event(&event)?;

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
    pub async fn subscribe_to_events(&self, component_name: &str, event_types: Vec<String>) -> Result<String> {
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
        let event_def = self.registry.events.get(event_type)
            .ok_or_else(|| anyhow::anyhow!("Unknown event type: {}", event_type))?;
        
        let handler_name = &event_def.handler;
        self.registry.handlers.get(handler_name)
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
    fn create_result_event(&self, original_event: &HooksmithEvent, result: &crate::xtask::event_bus::WasmCallResult) -> Option<HooksmithEvent> {
        // Extract the original event type and create a result event type
        let event_type = &original_event.event;
        let result_event_type = if event_type.ends_with("_request") {
            event_type.replace("_request", "_result")
        } else {
            format!("{}_result", event_type)
        };

        // Parse the result data
        if let Ok(result_data) = serde_json::from_str::<serde_json::Value>(&result.data) {
            Some(HooksmithEvent::new(
                "event-bus-manager".to_string(),
                result_event_type,
                result_data,
            ))
        } else {
            None
        }
    }

    /// Validate event against schema
    pub fn validate_event(&self, event: &HooksmithEvent) -> Result<()> {
        let event_def = self.get_event_definition(&event.event)?;
        
        // TODO: Implement schema validation using the schema path in event_def.schema
        // For now, just check that the event type is registered
        if event_def.schema.is_empty() {
            return Err(anyhow::anyhow!("Event schema not defined for: {}", event.event));
        }

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
    pub total_events: usize,
    pub total_handlers: usize,
    pub active_subscriptions: usize,
    pub registered_components: usize,
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
        let registry_path = self.registry_path
            .unwrap_or_else(|| "config/event-registry.jsonc".to_string());
        
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
            .registry_path("config/event-registry.jsonc".to_string())
            .auto_load_handlers(true)
            .build();
        
        // Should fail because the registry file doesn't exist in test
        assert!(manager.is_err());
    }
} 
