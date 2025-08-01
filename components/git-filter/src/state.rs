use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents the state of a Git attribute
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AttributeState {
    /// Attribute is set to true
    True,
    /// Attribute is set to false
    False,
    /// Attribute is set to a specific value
    Value(String),
    /// Attribute is unspecified
    Unspecified,
}

impl Default for AttributeState {
    fn default() -> Self {
        AttributeState::Unspecified
    }
}

impl From<Option<&str>> for AttributeState {
    fn from(value: Option<&str>) -> Self {
        match value {
            Some("true") => AttributeState::True,
            Some("false") => AttributeState::False,
            Some(val) => AttributeState::Value(val.to_string()),
            None => AttributeState::Unspecified,
        }
    }
}

/// Represents the effective state of a file based on Git attributes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileState {
    /// Text attribute state
    pub text: AttributeState,
    /// End-of-line attribute state
    pub eol: AttributeState,
    /// Filter driver attribute state
    pub filter: AttributeState,
    /// Diff driver attribute state
    pub diff: AttributeState,
    /// Merge driver attribute state
    pub merge: AttributeState,
    /// Working tree encoding attribute state
    pub encoding: AttributeState,
    /// Export ignore attribute state
    pub export_ignore: AttributeState,
    /// Export substitute attribute state
    pub export_subst: AttributeState,
    /// Custom attributes
    pub custom: HashMap<String, AttributeState>,
}

impl Default for FileState {
    fn default() -> Self {
        Self {
            text: AttributeState::Unspecified,
            eol: AttributeState::Unspecified,
            filter: AttributeState::Unspecified,
            diff: AttributeState::Unspecified,
            merge: AttributeState::Unspecified,
            encoding: AttributeState::Unspecified,
            export_ignore: AttributeState::Unspecified,
            export_subst: AttributeState::Unspecified,
            custom: HashMap::new(),
        }
    }
}

impl FileState {
    /// Create a new FileState from Git attributes
    pub fn from_attributes(attributes: &HashMap<String, Option<&str>>) -> Self {
        let mut state = FileState::default();

        // Map standard attributes
        state.text = attributes.get("text").cloned().into();
        state.eol = attributes.get("eol").cloned().into();
        state.filter = attributes.get("filter").cloned().into();
        state.diff = attributes.get("diff").cloned().into();
        state.merge = attributes.get("merge").cloned().into();
        state.encoding = attributes.get("working-tree-encoding").cloned().into();
        state.export_ignore = attributes.get("export-ignore").cloned().into();
        state.export_subst = attributes.get("export-subst").cloned().into();

        // Map custom attributes
        for (key, value) in attributes {
            if !matches!(
                key.as_str(),
                "text"
                    | "eol"
                    | "filter"
                    | "diff"
                    | "merge"
                    | "working-tree-encoding"
                    | "export-ignore"
                    | "export-subst"
            ) {
                state.custom.insert(key.clone(), value.clone().into());
            }
        }

        state
    }

    /// Check if a specific attribute is enabled
    pub fn is_enabled(&self, attribute: &str) -> bool {
        match attribute {
            "text" => matches!(self.text, AttributeState::True),
            "eol" => matches!(self.eol, AttributeState::True | AttributeState::Value(_)),
            "filter" => matches!(self.filter, AttributeState::True | AttributeState::Value(_)),
            "diff" => matches!(self.diff, AttributeState::True | AttributeState::Value(_)),
            "merge" => matches!(self.merge, AttributeState::True | AttributeState::Value(_)),
            "encoding" => matches!(
                self.encoding,
                AttributeState::True | AttributeState::Value(_)
            ),
            "export-ignore" => matches!(self.export_ignore, AttributeState::True),
            "export-subst" => matches!(self.export_subst, AttributeState::True),
            _ => self
                .custom
                .get(attribute)
                .map(|state| matches!(state, AttributeState::True))
                .unwrap_or(false),
        }
    }

    /// Get the value of a specific attribute
    pub fn get_value(&self, attribute: &str) -> Option<&str> {
        match attribute {
            "text" => match &self.text {
                AttributeState::Value(val) => Some(val),
                _ => None,
            },
            "eol" => match &self.eol {
                AttributeState::Value(val) => Some(val),
                _ => None,
            },
            "filter" => match &self.filter {
                AttributeState::Value(val) => Some(val),
                _ => None,
            },
            "diff" => match &self.diff {
                AttributeState::Value(val) => Some(val),
                _ => None,
            },
            "merge" => match &self.merge {
                AttributeState::Value(val) => Some(val),
                _ => None,
            },
            "encoding" => match &self.encoding {
                AttributeState::Value(val) => Some(val),
                _ => None,
            },
            _ => self.custom.get(attribute).and_then(|state| match state {
                AttributeState::Value(val) => Some(val.as_str()),
                _ => None,
            }),
        }
    }

    /// Check if the file should be treated as text
    pub fn is_text(&self) -> bool {
        matches!(self.text, AttributeState::True)
    }

    /// Get the end-of-line setting
    pub fn get_eol(&self) -> Option<&str> {
        self.get_value("eol")
    }

    /// Get the filter driver name
    pub fn get_filter_driver(&self) -> Option<&str> {
        self.get_value("filter")
    }

    /// Get the working tree encoding
    pub fn get_encoding(&self) -> Option<&str> {
        self.get_value("encoding")
    }
}
