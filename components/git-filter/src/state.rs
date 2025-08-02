use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents the state of a Git attribute
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AttributeState {
    /// Attribute is set to true
    Set,
    /// Attribute is set to false
    Unset,
    /// Attribute is unspecified
    #[default]
    Unspecified,
}

impl From<Option<&str>> for AttributeState {
    fn from(value: Option<&str>) -> Self {
        match value {
            Some("true") => AttributeState::Set,
            Some("false") => AttributeState::Unset,
            Some(_val) => AttributeState::Unspecified,
            None => AttributeState::Unspecified,
        }
    }
}

/// Represents the effective state of a file based on Git attributes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct FileState {
    /// Text attribute state
    pub text: AttributeState,
    /// EOL attribute state
    pub eol: AttributeState,
    /// Filter attribute state
    pub filter: AttributeState,
    /// Diff attribute state
    pub diff: AttributeState,
    /// Merge attribute state
    pub merge: AttributeState,
    /// Working tree encoding attribute state
    pub encoding: AttributeState,
    /// Export ignore attribute state
    pub export_ignore: AttributeState,
    /// Export subst attribute state
    pub export_subst: AttributeState,
    /// Custom attributes
    pub custom: HashMap<String, AttributeState>,
}

impl FileState {
    /// Create a new FileState from Git attributes
    pub fn from_attributes(attributes: &HashMap<String, Option<&str>>) -> Self {
        let mut state = FileState {
            text: attributes.get("text").and_then(|v| *v).into(),
            eol: attributes.get("eol").and_then(|v| *v).into(),
            filter: attributes.get("filter").and_then(|v| *v).into(),
            diff: attributes.get("diff").and_then(|v| *v).into(),
            merge: attributes.get("merge").and_then(|v| *v).into(),
            encoding: attributes
                .get("working-tree-encoding")
                .and_then(|v| *v)
                .into(),
            export_ignore: attributes.get("export-ignore").and_then(|v| *v).into(),
            export_subst: attributes.get("export-subst").and_then(|v| *v).into(),
            custom: HashMap::new(),
        };

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
                state
                    .custom
                    .insert(key.clone(), (*value).into());
            }
        }

        state
    }

    /// Check if a specific attribute is enabled
    pub fn is_enabled(&self, attribute: &str) -> bool {
        match attribute {
            "text" => matches!(self.text, AttributeState::Set),
            "eol" => matches!(self.eol, AttributeState::Set | AttributeState::Unset),
            "filter" => matches!(self.filter, AttributeState::Set | AttributeState::Unset),
            "diff" => matches!(self.diff, AttributeState::Set | AttributeState::Unset),
            "merge" => matches!(self.merge, AttributeState::Set | AttributeState::Unset),
            "encoding" => matches!(
                self.encoding,
                AttributeState::Set | AttributeState::Unset
            ),
            "export-ignore" => matches!(self.export_ignore, AttributeState::Set),
            "export-subst" => matches!(self.export_subst, AttributeState::Set),
            _ => self
                .custom
                .get(attribute)
                .map(|state| matches!(state, AttributeState::Set))
                .unwrap_or(false),
        }
    }

    /// Get the value of a specific attribute
    pub fn get_value(&self, attribute: &str) -> Option<&str> {
        match attribute {
            "text" => match &self.text {
                AttributeState::Unspecified => None,
                AttributeState::Set => Some("true"),
                AttributeState::Unset => Some("false"),
            },
            "eol" => match &self.eol {
                AttributeState::Unspecified => None,
                AttributeState::Set => Some("true"),
                AttributeState::Unset => Some("false"),
            },
            "filter" => match &self.filter {
                AttributeState::Unspecified => None,
                AttributeState::Set => Some("true"),
                AttributeState::Unset => Some("false"),
            },
            "diff" => match &self.diff {
                AttributeState::Unspecified => None,
                AttributeState::Set => Some("true"),
                AttributeState::Unset => Some("false"),
            },
            "merge" => match &self.merge {
                AttributeState::Unspecified => None,
                AttributeState::Set => Some("true"),
                AttributeState::Unset => Some("false"),
            },
            "encoding" => match &self.encoding {
                AttributeState::Unspecified => None,
                AttributeState::Set => Some("true"),
                AttributeState::Unset => Some("false"),
            },
            _ => self.custom.get(attribute).and_then(|state| match state {
                AttributeState::Unspecified => None,
                AttributeState::Set => Some("true"),
                AttributeState::Unset => Some("false"),
            }),
        }
    }

    /// Check if the file should be treated as text
    pub fn is_text(&self) -> bool {
        matches!(self.text, AttributeState::Set)
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
