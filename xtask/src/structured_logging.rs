use chrono::Utc;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize)]
pub struct BootstrapEvent {
    timestamp: String,
    level: String,
    action: String,
    message: String,
    details: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    file: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    line: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<HashMap<String, String>>,
}

#[derive(Serialize)]
pub struct SarifResult {
    #[serde(rename = "$schema")]
    schema: String,
    version: String,
    runs: Vec<SarifRun>,
}

#[derive(Serialize)]
pub struct SarifRun {
    tool: SarifTool,
    results: Vec<SarifResultItem>,
}

#[derive(Serialize)]
pub struct SarifTool {
    driver: SarifToolComponent,
}

#[derive(Serialize)]
pub struct SarifToolComponent {
    name: String,
    version: String,
}

#[derive(Serialize)]
pub struct SarifResultItem {
    level: String,
    message: SarifMessage,
    #[serde(skip_serializing_if = "Option::is_none")]
    locations: Option<Vec<SarifLocation>>,
}

#[derive(Serialize)]
pub struct SarifMessage {
    text: String,
}

#[derive(Serialize)]
pub struct SarifLocation {
    #[serde(skip_serializing_if = "Option::is_none")]
    physical_location: Option<SarifPhysicalLocation>,
}

#[derive(Serialize)]
pub struct SarifPhysicalLocation {
    #[serde(skip_serializing_if = "Option::is_none")]
    artifact_location: Option<SarifArtifactLocation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    region: Option<SarifRegion>,
}

#[derive(Serialize)]
pub struct SarifArtifactLocation {
    uri: String,
}

#[derive(Serialize)]
pub struct SarifRegion {
    #[serde(skip_serializing_if = "Option::is_none")]
    start_line: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    start_column: Option<u32>,
}

#[macro_export]
macro_rules! log_event {
    ($level:expr, $action:expr, $msg:expr, $details:expr) => {{
        use crate::structured_logging::BootstrapEvent;
        use chrono::Utc;
        use serde_json;
        use std::collections::HashMap;

        let event = BootstrapEvent {
            timestamp: Utc::now().to_rfc3339(),
            level: $level.to_string(),
            action: $action.to_string(),
            message: $msg.to_string(),
            details: $details.map(|s| s.to_string()),
            file: None,
            line: None,
            metadata: None,
        };
        println!("{}", serde_json::to_string(&event).unwrap());
    }};
}

#[macro_export]
macro_rules! log_event_with_location {
    ($level:expr, $action:expr, $msg:expr, $details:expr, $file:expr, $line:expr) => {{
        use crate::structured_logging::BootstrapEvent;
        use chrono::Utc;
        use serde_json;
        use std::collections::HashMap;

        let event = BootstrapEvent {
            timestamp: Utc::now().to_rfc3339(),
            level: $level.to_string(),
            action: $action.to_string(),
            message: $msg.to_string(),
            details: $details.map(|s| s.to_string()),
            file: Some($file.to_string()),
            line: Some($line),
            metadata: None,
        };
        println!("{}", serde_json::to_string(&event).unwrap());
    }};
}

#[macro_export]
macro_rules! log_event_with_metadata {
    ($level:expr, $action:expr, $msg:expr, $details:expr, $metadata:expr) => {{
        use crate::structured_logging::BootstrapEvent;
        use chrono::Utc;
        use serde_json;
        use std::collections::HashMap;

        let event = BootstrapEvent {
            timestamp: Utc::now().to_rfc3339(),
            level: $level.to_string(),
            action: $action.to_string(),
            message: $msg.to_string(),
            details: $details.map(|s| s.to_string()),
            file: None,
            line: None,
            metadata: Some($metadata),
        };
        println!("{}", serde_json::to_string(&event).unwrap());
    }};
}

pub fn emit_sarif_error(file: &str, line: u32, msg: &str) {
    let sarif = SarifResult {
        schema: "https://schemastore.azurewebsites.net/schemas/json/sarif-2.1.0-rtm.5.json".to_string(),
        version: "2.1.0".to_string(),
        runs: vec![SarifRun {
            tool: SarifTool {
                driver: SarifToolComponent {
                    name: "hooksmith-xtask-bootstrap".to_string(),
                    version: "0.1.0".to_string(),
                },
            },
            results: vec![SarifResultItem {
                level: "error".to_string(),
                message: SarifMessage {
                    text: msg.to_string(),
                },
                locations: Some(vec![SarifLocation {
                    physical_location: Some(SarifPhysicalLocation {
                        artifact_location: Some(SarifArtifactLocation {
                            uri: file.to_string(),
                        }),
                        region: Some(SarifRegion {
                            start_line: Some(line),
                            start_column: Some(1),
                        }),
                    }),
                }]),
            }],
        }],
    };

    println!("{}", serde_json::to_string_pretty(&sarif).unwrap());
}

pub fn emit_sarif_warning(file: &str, line: u32, msg: &str) {
    let sarif = SarifResult {
        schema: "https://schemastore.azurewebsites.net/schemas/json/sarif-2.1.0-rtm.5.json".to_string(),
        version: "2.1.0".to_string(),
        runs: vec![SarifRun {
            tool: SarifTool {
                driver: SarifToolComponent {
                    name: "hooksmith-xtask-bootstrap".to_string(),
                    version: "0.1.0".to_string(),
                },
            },
            results: vec![SarifResultItem {
                level: "warning".to_string(),
                message: SarifMessage {
                    text: msg.to_string(),
                },
                locations: Some(vec![SarifLocation {
                    physical_location: Some(SarifPhysicalLocation {
                        artifact_location: Some(SarifArtifactLocation {
                            uri: file.to_string(),
                        }),
                        region: Some(SarifRegion {
                            start_line: Some(line),
                            start_column: Some(1),
                        }),
                    }),
                }]),
            }],
        }],
    };

    println!("{}", serde_json::to_string_pretty(&sarif).unwrap());
}
