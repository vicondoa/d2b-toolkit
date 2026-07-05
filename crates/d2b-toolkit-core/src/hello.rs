use serde::{Deserialize, Serialize};

pub const PROTOCOL_NAME: &str = "d2b-toolkit";
pub const CURRENT_PROTOCOL_VERSION: u16 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Feature {
    ShellOwnerStream,
    WaylandColors,
    WaybarSurface,
}

/// Public daemon hello frame. This is intentionally metadata-only and must not carry
/// argv/env/cwd, terminal bytes, or opaque session handles.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Hello {
    pub client_name: String,
    pub version_range: String,
    pub features: Vec<Feature>,
}

impl Hello {
    pub fn toolkit_client(client_name: impl Into<String>, features: Vec<Feature>) -> Self {
        Self {
            client_name: client_name.into(),
            version_range: ">=0.4.0, <0.5.0".to_string(),
            features,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum HelloResponse {
    HelloOk {
        protocol_version: u16,
        accepted_features: Vec<Feature>,
    },
    HelloRejected {
        reason: String,
    },
}
