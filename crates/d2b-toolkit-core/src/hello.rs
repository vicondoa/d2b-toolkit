use serde::{Deserialize, Serialize};
use std::fmt;

pub const PROTOCOL_NAME: &str = "d2b-public";
pub const CURRENT_PROTOCOL_VERSION: u32 = 3;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Version(String);

impl Version {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Version").field(&self.0).finish()
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SemverRange(String);

impl SemverRange {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for SemverRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("SemverRange").field(&self.0).finish()
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FeatureFlag(String);

impl FeatureFlag {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for FeatureFlag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("FeatureFlag").field(&self.0).finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KnownFeatureFlag {
    TypedErrors,
    ManifestV04,
    StatusCheckBridges,
    ExportBrokerAudit,
}

impl KnownFeatureFlag {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::TypedErrors => "typed-errors",
            Self::ManifestV04 => "manifest-v04",
            Self::StatusCheckBridges => "status-check-bridges",
            Self::ExportBrokerAudit => "export-broker-audit",
        }
    }

    pub fn wire_value(self) -> FeatureFlag {
        FeatureFlag::new(self.as_str())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Hello {
    pub client_version: SemverRange,
    #[serde(default)]
    pub supported_features: Vec<FeatureFlag>,
}

impl Hello {
    pub fn toolkit_client(supported_features: Vec<FeatureFlag>) -> Self {
        Self {
            client_version: SemverRange::new(">=0.4.0, <0.5.0"),
            supported_features,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HelloFrame {
    #[serde(rename = "type")]
    pub type_name: HelloFrameType,
    #[serde(flatten)]
    pub payload: Hello,
}

impl HelloFrame {
    pub fn new(payload: Hello) -> Self {
        Self {
            type_name: HelloFrameType::Hello,
            payload,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum HelloFrameType {
    Hello,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HelloOk {
    pub server_version: Version,
    pub selected_version: Version,
    pub capabilities: Vec<FeatureFlag>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HelloRejected {
    pub reason: HelloRejectedReason,
    pub error: crate::public::ErrorEnvelope,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum HelloRejectedReason {
    VersionMismatch,
    CapabilityNegotiationFailed,
    InternalError,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum HelloResponse {
    HelloOk(HelloOk),
    HelloRejected(HelloRejected),
}
