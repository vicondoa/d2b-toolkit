use crate::{TokenKind, ToolkitError, ValidationReason};
use serde::de::{SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::BTreeSet;
use std::fmt;

pub const PROTOCOL_NAME: &str = "d2b-public";
pub const CURRENT_PROTOCOL_VERSION: u32 = 3;
pub const MAX_FEATURE_FLAG_LEN: usize = 64;
pub const MAX_HELLO_FEATURES: usize = 64;

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

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct FeatureFlag(String);

impl FeatureFlag {
    pub fn new(value: impl Into<String>) -> Result<Self, ToolkitError> {
        let value = value.into();
        let reason = if value.is_empty() {
            Some(ValidationReason::Empty)
        } else if value.len() > MAX_FEATURE_FLAG_LEN {
            Some(ValidationReason::TooLong)
        } else if !value
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        {
            Some(ValidationReason::BadShape)
        } else {
            None
        };
        match reason {
            Some(reason) => Err(ToolkitError::InvalidToken {
                kind: TokenKind::Feature,
                reason,
            }),
            None => Ok(Self(value)),
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn known(&self) -> Option<KnownFeatureFlag> {
        KnownFeatureFlag::from_wire(self.as_str())
    }
}

impl fmt::Debug for FeatureFlag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("FeatureFlag").field(&self.0).finish()
    }
}

impl<'de> Deserialize<'de> for FeatureFlag {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::new(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum KnownFeatureFlag {
    TypedErrors,
    ManifestV04,
    StatusCheckBridges,
    ExportBrokerAudit,
    ConfiguredLaunchV1,
    UnsafeLocalProviderV1,
    UnsafeLocalShellV1,
}

impl KnownFeatureFlag {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::TypedErrors => "typed-errors",
            Self::ManifestV04 => "manifest-v04",
            Self::StatusCheckBridges => "status-check-bridges",
            Self::ExportBrokerAudit => "export-broker-audit",
            Self::ConfiguredLaunchV1 => "configured-launch-v1",
            Self::UnsafeLocalProviderV1 => "unsafe-local-provider-v1",
            Self::UnsafeLocalShellV1 => "unsafe-local-shell-v1",
        }
    }

    pub fn wire_value(self) -> FeatureFlag {
        FeatureFlag(self.as_str().to_owned())
    }

    pub fn from_wire(value: &str) -> Option<Self> {
        Some(match value {
            "typed-errors" => Self::TypedErrors,
            "manifest-v04" => Self::ManifestV04,
            "status-check-bridges" => Self::StatusCheckBridges,
            "export-broker-audit" => Self::ExportBrokerAudit,
            "configured-launch-v1" => Self::ConfiguredLaunchV1,
            "unsafe-local-provider-v1" => Self::UnsafeLocalProviderV1,
            "unsafe-local-shell-v1" => Self::UnsafeLocalShellV1,
            _ => return None,
        })
    }
}

impl fmt::Display for KnownFeatureFlag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

fn deserialize_features<'de, D>(deserializer: D) -> Result<Vec<FeatureFlag>, D::Error>
where
    D: Deserializer<'de>,
{
    struct FeatureVisitor;

    impl<'de> Visitor<'de> for FeatureVisitor {
        type Value = Vec<FeatureFlag>;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("a bounded feature token array")
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            if seq.size_hint().unwrap_or(0) > MAX_HELLO_FEATURES {
                return Err(serde::de::Error::custom(
                    "hello feature set exceeds entry bound",
                ));
            }
            let mut features = Vec::new();
            while let Some(feature) = seq.next_element()? {
                if features.len() == MAX_HELLO_FEATURES {
                    return Err(serde::de::Error::custom(
                        "hello feature set exceeds entry bound",
                    ));
                }
                features.push(feature);
            }
            Ok(features)
        }
    }

    deserializer.deserialize_seq(FeatureVisitor)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Hello {
    pub client_version: SemverRange,
    #[serde(default, deserialize_with = "deserialize_features")]
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
    #[serde(deserialize_with = "deserialize_features")]
    pub capabilities: Vec<FeatureFlag>,
}

impl HelloOk {
    pub fn has_feature(&self, feature: KnownFeatureFlag) -> bool {
        self.capabilities
            .iter()
            .any(|candidate| candidate.known() == Some(feature))
    }

    pub fn require_feature(&self, feature: KnownFeatureFlag) -> Result<(), ToolkitError> {
        if self.has_feature(feature) {
            Ok(())
        } else {
            Err(ToolkitError::FeatureUnavailable { feature })
        }
    }

    pub fn negotiated_capabilities(&self) -> NegotiatedCapabilities {
        NegotiatedCapabilities::from_features(self.capabilities.clone())
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NegotiatedCapabilities {
    features: BTreeSet<FeatureFlag>,
}

impl NegotiatedCapabilities {
    pub fn from_features<I>(features: I) -> Self
    where
        I: IntoIterator<Item = FeatureFlag>,
    {
        Self {
            features: features.into_iter().collect(),
        }
    }

    pub fn from_hello_ok(hello: &HelloOk) -> Self {
        hello.negotiated_capabilities()
    }

    pub fn features(&self) -> impl Iterator<Item = &FeatureFlag> {
        self.features.iter()
    }

    pub fn has(&self, feature: KnownFeatureFlag) -> bool {
        self.features
            .iter()
            .any(|candidate| candidate.known() == Some(feature))
    }

    pub fn require(&self, feature: KnownFeatureFlag) -> Result<(), ToolkitError> {
        if self.has(feature) {
            Ok(())
        } else {
            Err(ToolkitError::FeatureUnavailable { feature })
        }
    }

    pub fn require_all(
        &self,
        features: impl IntoIterator<Item = KnownFeatureFlag>,
    ) -> Result<(), ToolkitError> {
        for feature in features {
            self.require(feature)?;
        }
        Ok(())
    }
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
