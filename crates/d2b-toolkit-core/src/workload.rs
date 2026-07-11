use crate::error::{
    IdentityField, LauncherItemCandidate, LauncherItemCandidates, TokenKind, ValidationReason,
};
use crate::ToolkitError;
use serde::de::{SeqAccess, Visitor};
use serde::ser::SerializeSeq;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::BTreeSet;
use std::fmt;
use std::marker::PhantomData;
use std::str::FromStr;

pub const MAX_ID_LEN: usize = 128;
pub const MAX_PROTOCOL_TOKEN_LEN: usize = 64;
pub const MAX_IDENTITY_TOKEN_LEN: usize = 160;
pub const MAX_REALM_LABELS: usize = 16;
pub const MAX_REALM_PATH_BYTES: usize = 255;
pub const MAX_WORKLOAD_TARGET_LEN: usize = 388;
pub const MAX_PRESENTATION_TEXT_LEN: usize = 512;
pub const MAX_CAPABILITY_SET_LEN: usize = 64;
pub const MAX_LAUNCHER_ITEMS_PER_WORKLOAD: usize = 64;
pub const MAX_WORKLOADS_PER_RESPONSE: usize = 512;

fn is_label(value: &str) -> bool {
    let mut chars = value.chars();
    matches!(chars.next(), Some(first) if first.is_ascii_lowercase())
        && chars.all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '-')
}

fn validate_label(value: &str) -> ValidationReason {
    if value.is_empty() {
        ValidationReason::Empty
    } else if value.len() > MAX_ID_LEN {
        ValidationReason::TooLong
    } else if !is_label(value) {
        ValidationReason::BadShape
    } else {
        ValidationReason::Inconsistent
    }
}

macro_rules! label_newtype {
    ($name:ident, $field:expr) => {
        #[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            pub fn parse(value: impl Into<String>) -> Result<Self, ToolkitError> {
                let value = value.into();
                if !value.is_empty() && value.len() <= MAX_ID_LEN && is_label(&value) {
                    Ok(Self(value))
                } else {
                    Err(ToolkitError::InvalidIdentity {
                        field: $field,
                        reason: validate_label(&value),
                    })
                }
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_tuple(stringify!($name)).field(&self.0).finish()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl FromStr for $name {
            type Err = ToolkitError;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                Self::parse(value)
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                Self::parse(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
            }
        }
    };
}

label_newtype!(WorkloadId, IdentityField::WorkloadId);
label_newtype!(RealmId, IdentityField::RealmId);

fn is_identity_token(value: &str) -> bool {
    let mut chars = value.chars();
    matches!(chars.next(), Some(first) if first.is_ascii_alphanumeric())
        && chars.all(|ch| {
            ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | ':' | '/' | '@' | '+' | '-')
        })
}

macro_rules! identity_token_newtype {
    ($name:ident, $field:expr) => {
        #[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            pub fn parse(value: impl Into<String>) -> Result<Self, ToolkitError> {
                let value = value.into();
                let reason = if value.is_empty() {
                    Some(ValidationReason::Empty)
                } else if value.len() > MAX_IDENTITY_TOKEN_LEN {
                    Some(ValidationReason::TooLong)
                } else if !is_identity_token(&value) {
                    Some(ValidationReason::BadShape)
                } else {
                    None
                };
                match reason {
                    Some(reason) => Err(ToolkitError::InvalidIdentity {
                        field: $field,
                        reason,
                    }),
                    None => Ok(Self(value)),
                }
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_tuple(stringify!($name)).field(&self.0).finish()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }

        impl FromStr for $name {
            type Err = ToolkitError;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                Self::parse(value)
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                Self::parse(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
            }
        }
    };
}

identity_token_newtype!(LegacyVmName, IdentityField::LegacyVmName);
identity_token_newtype!(RuntimeKind, IdentityField::RuntimeKind);
identity_token_newtype!(ProviderId, IdentityField::ProviderId);

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct ProtocolToken(String);

impl ProtocolToken {
    pub fn parse(value: impl Into<String>) -> Result<Self, ToolkitError> {
        let value = value.into();
        let reason = if value.is_empty() {
            Some(ValidationReason::Empty)
        } else if value.len() > MAX_PROTOCOL_TOKEN_LEN {
            Some(ValidationReason::TooLong)
        } else if !value.bytes().all(|byte| byte.is_ascii_graphic()) {
            Some(ValidationReason::BadShape)
        } else {
            None
        };
        match reason {
            Some(reason) => Err(ToolkitError::InvalidToken {
                kind: TokenKind::Protocol,
                reason,
            }),
            None => Ok(Self(value)),
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for ProtocolToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ProtocolToken").field(&self.0).finish()
    }
}

impl fmt::Display for ProtocolToken {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for ProtocolToken {
    type Err = ToolkitError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value)
    }
}

impl<'de> Deserialize<'de> for ProtocolToken {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::parse(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

const SECRET_MARKERS: &[&str] = &[
    "secret",
    "password",
    "passwd",
    "bearer",
    "credential",
    "private",
    "apikey",
    "token",
    "privatekey",
    "accesstoken",
    "refreshtoken",
    "sessiontoken",
];

fn is_operation_id(value: &str) -> bool {
    let mut chars = value.chars();
    if !matches!(chars.next(), Some(first) if first.is_ascii_alphanumeric())
        || !chars.all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.'))
        || value.contains("..")
    {
        return false;
    }
    let compact = value
        .chars()
        .filter(|ch| !matches!(ch, '-' | '_' | '.'))
        .flat_map(char::to_lowercase)
        .collect::<String>();
    !SECRET_MARKERS.iter().any(|marker| compact.contains(marker))
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct OperationId(String);

impl OperationId {
    pub fn parse(value: impl Into<String>) -> Result<Self, ToolkitError> {
        let value = value.into();
        let reason = if value.is_empty() {
            Some(ValidationReason::Empty)
        } else if value.len() > MAX_ID_LEN {
            Some(ValidationReason::TooLong)
        } else if !is_operation_id(&value) {
            Some(ValidationReason::BadShape)
        } else {
            None
        };
        match reason {
            Some(reason) => Err(ToolkitError::InvalidToken {
                kind: TokenKind::Operation,
                reason,
            }),
            None => Ok(Self(value)),
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for OperationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("OperationId([redacted])")
    }
}

impl fmt::Display for OperationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[redacted]")
    }
}

impl FromStr for OperationId {
    type Err = ToolkitError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value)
    }
}

impl<'de> Deserialize<'de> for OperationId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::parse(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct RealmPath(Vec<RealmId>);

impl RealmPath {
    pub fn new(labels: Vec<RealmId>) -> Result<Self, ToolkitError> {
        let total = labels
            .iter()
            .map(|label| label.as_str().len())
            .sum::<usize>()
            + labels.len().saturating_sub(1);
        let reason = if labels.is_empty() {
            Some(ValidationReason::Empty)
        } else if labels.len() > MAX_REALM_LABELS || total > MAX_REALM_PATH_BYTES {
            Some(ValidationReason::TooLong)
        } else {
            None
        };
        match reason {
            Some(reason) => Err(ToolkitError::InvalidIdentity {
                field: IdentityField::RealmPath,
                reason,
            }),
            None => Ok(Self(labels)),
        }
    }

    pub fn labels(&self) -> &[RealmId] {
        &self.0
    }

    pub fn target_form(&self) -> String {
        self.0
            .iter()
            .map(RealmId::as_str)
            .collect::<Vec<_>>()
            .join(".")
    }
}

impl fmt::Debug for RealmPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("RealmPath")
            .field(&self.target_form())
            .finish()
    }
}

impl<'de> Deserialize<'de> for RealmPath {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct RealmPathVisitor;

        impl<'de> Visitor<'de> for RealmPathVisitor {
            type Value = RealmPath;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a bounded non-empty realm label array")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                if seq.size_hint().unwrap_or(0) > MAX_REALM_LABELS {
                    return Err(serde::de::Error::custom("realm path exceeds label bound"));
                }
                let mut labels = Vec::new();
                while let Some(label) = seq.next_element()? {
                    if labels.len() == MAX_REALM_LABELS {
                        return Err(serde::de::Error::custom("realm path exceeds label bound"));
                    }
                    labels.push(label);
                }
                RealmPath::new(labels).map_err(serde::de::Error::custom)
            }
        }

        deserializer.deserialize_seq(RealmPathVisitor)
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WorkloadTarget {
    canonical: String,
    workload: WorkloadId,
    realm: RealmPath,
}

impl WorkloadTarget {
    pub fn parse(raw: &str) -> Result<Self, ToolkitError> {
        let body = raw.strip_prefix("d2b://").unwrap_or(raw);
        if body.is_empty() {
            return Err(ToolkitError::InvalidTarget {
                reason: ValidationReason::Empty,
            });
        }
        if body.len() > MAX_WORKLOAD_TARGET_LEN {
            return Err(ToolkitError::InvalidTarget {
                reason: ValidationReason::TooLong,
            });
        }
        let mut labels = body.split('.').collect::<Vec<_>>();
        if labels.len() < 3 || labels.pop() != Some("d2b") {
            return Err(ToolkitError::InvalidTarget {
                reason: ValidationReason::BadShape,
            });
        }
        if labels
            .iter()
            .any(|label| matches!(*label, "all" | "*" | "d2b"))
        {
            return Err(ToolkitError::InvalidTarget {
                reason: ValidationReason::BadShape,
            });
        }
        let workload = WorkloadId::parse(labels[0]).map_err(|_| ToolkitError::InvalidTarget {
            reason: ValidationReason::BadShape,
        })?;
        let realm_labels = labels[1..]
            .iter()
            .map(|label| {
                RealmId::parse(*label).map_err(|_| ToolkitError::InvalidTarget {
                    reason: ValidationReason::BadShape,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        let realm = RealmPath::new(realm_labels).map_err(|_| ToolkitError::InvalidTarget {
            reason: ValidationReason::TooLong,
        })?;
        let canonical = format!("{}.{}.d2b", workload.as_str(), realm.target_form());
        Ok(Self {
            canonical,
            workload,
            realm,
        })
    }

    pub fn as_str(&self) -> &str {
        &self.canonical
    }

    pub fn to_canonical(&self) -> String {
        self.canonical.clone()
    }

    pub fn workload(&self) -> &WorkloadId {
        &self.workload
    }

    pub fn realm(&self) -> &RealmPath {
        &self.realm
    }
}

impl fmt::Debug for WorkloadTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("WorkloadTarget")
            .field(&self.canonical)
            .finish()
    }
}

impl fmt::Display for WorkloadTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.canonical)
    }
}

impl FromStr for WorkloadTarget {
    type Err = ToolkitError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value)
    }
}

impl Serialize for WorkloadTarget {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.canonical)
    }
}

impl<'de> Deserialize<'de> for WorkloadTarget {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::parse(&String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
    }
}

fn validate_presentation(
    value: &str,
    field: IdentityField,
    allow_empty: bool,
) -> Result<(), ToolkitError> {
    let reason = if !allow_empty && value.is_empty() {
        Some(ValidationReason::Empty)
    } else if value.len() > MAX_PRESENTATION_TEXT_LEN {
        Some(ValidationReason::TooLong)
    } else if value.chars().any(char::is_control) {
        Some(ValidationReason::BadShape)
    } else {
        None
    };
    match reason {
        Some(reason) => Err(ToolkitError::InvalidIdentity { field, reason }),
        None => Ok(()),
    }
}

fn deserialize_optional_workload_name<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<String>::deserialize(deserializer)?;
    if let Some(value) = value.as_deref() {
        validate_presentation(value, IdentityField::WorkloadName, false)
            .map_err(serde::de::Error::custom)?;
    }
    Ok(value)
}

fn deserialize_launcher_name<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    validate_presentation(&value, IdentityField::LauncherName, false)
        .map_err(serde::de::Error::custom)?;
    Ok(value)
}

fn deserialize_optional_icon<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<String>::deserialize(deserializer)?;
    if let Some(value) = value.as_deref() {
        validate_presentation(value, IdentityField::LauncherIcon, true)
            .map_err(serde::de::Error::custom)?;
    }
    Ok(value)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorkloadIdentity {
    pub workload_id: WorkloadId,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_optional_workload_name"
    )]
    pub workload_name: Option<String>,
    pub realm_id: RealmId,
    pub realm_path: RealmPath,
    pub canonical_target: WorkloadTarget,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub legacy_vm_name: Option<LegacyVmName>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime_kind: Option<RuntimeKind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_id: Option<ProviderId>,
}

impl WorkloadIdentity {
    pub fn new(
        workload_id: WorkloadId,
        realm_id: RealmId,
        realm_path: RealmPath,
        canonical_target: WorkloadTarget,
    ) -> Self {
        Self {
            workload_id,
            workload_name: None,
            realm_id,
            realm_path,
            canonical_target,
            legacy_vm_name: None,
            runtime_kind: None,
            provider_id: None,
        }
    }

    pub fn target(&self) -> &WorkloadTarget {
        &self.canonical_target
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Capability {
    Lifecycle,
    Exec,
    Pty,
    Logs,
    FileCopy,
    PortForward,
    PersistentShell,
    Vsock,
    Virtiofs,
    WindowForwarding,
    DisplayStreaming,
    Clipboard,
    AudioPlayback,
    AudioCapture,
    Hid,
    Usb,
    GpuAccel,
    Snapshots,
    Hotplug,
    EphemeralSessions,
    ProviderManagedIsolation,
    ConfiguredLaunch,
}

impl Capability {
    pub const fn code(self) -> &'static str {
        match self {
            Self::Lifecycle => "lifecycle",
            Self::Exec => "exec",
            Self::Pty => "pty",
            Self::Logs => "logs",
            Self::FileCopy => "file-copy",
            Self::PortForward => "port-forward",
            Self::PersistentShell => "persistent-shell",
            Self::Vsock => "vsock",
            Self::Virtiofs => "virtiofs",
            Self::WindowForwarding => "window-forwarding",
            Self::DisplayStreaming => "display-streaming",
            Self::Clipboard => "clipboard",
            Self::AudioPlayback => "audio-playback",
            Self::AudioCapture => "audio-capture",
            Self::Hid => "hid",
            Self::Usb => "usb",
            Self::GpuAccel => "gpu-accel",
            Self::Snapshots => "snapshots",
            Self::Hotplug => "hotplug",
            Self::EphemeralSessions => "ephemeral-sessions",
            Self::ProviderManagedIsolation => "provider-managed-isolation",
            Self::ConfiguredLaunch => "configured-launch",
        }
    }

    pub fn from_code(code: &str) -> Option<Self> {
        Some(match code {
            "lifecycle" => Self::Lifecycle,
            "exec" => Self::Exec,
            "pty" => Self::Pty,
            "logs" => Self::Logs,
            "file-copy" => Self::FileCopy,
            "port-forward" => Self::PortForward,
            "persistent-shell" => Self::PersistentShell,
            "vsock" => Self::Vsock,
            "virtiofs" => Self::Virtiofs,
            "window-forwarding" => Self::WindowForwarding,
            "display-streaming" => Self::DisplayStreaming,
            "clipboard" => Self::Clipboard,
            "audio-playback" => Self::AudioPlayback,
            "audio-capture" => Self::AudioCapture,
            "hid" => Self::Hid,
            "usb" => Self::Usb,
            "gpu-accel" => Self::GpuAccel,
            "snapshots" => Self::Snapshots,
            "hotplug" => Self::Hotplug,
            "ephemeral-sessions" => Self::EphemeralSessions,
            "provider-managed-isolation" => Self::ProviderManagedIsolation,
            "configured-launch" => Self::ConfiguredLaunch,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CapabilitySet {
    known: BTreeSet<Capability>,
    unknown: BTreeSet<ProtocolToken>,
}

impl CapabilitySet {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn from_caps<I>(capabilities: I) -> Self
    where
        I: IntoIterator<Item = Capability>,
    {
        Self {
            known: capabilities.into_iter().collect(),
            unknown: BTreeSet::new(),
        }
    }

    pub fn from_tokens<I>(tokens: I) -> Self
    where
        I: IntoIterator<Item = ProtocolToken>,
    {
        let mut set = Self::empty();
        for token in tokens {
            if let Some(capability) = Capability::from_code(token.as_str()) {
                set.known.insert(capability);
            } else {
                set.unknown.insert(token);
            }
        }
        set
    }

    pub fn with(mut self, capability: Capability) -> Self {
        self.known.insert(capability);
        self
    }

    pub fn has(&self, capability: Capability) -> bool {
        self.known.contains(&capability)
    }

    pub fn iter(&self) -> impl Iterator<Item = Capability> + '_ {
        self.known.iter().copied()
    }

    pub fn unknown_iter(&self) -> impl Iterator<Item = &ProtocolToken> {
        self.unknown.iter()
    }
}

impl Serialize for CapabilitySet {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut codes = self
            .known
            .iter()
            .map(|capability| capability.code())
            .chain(self.unknown.iter().map(ProtocolToken::as_str))
            .collect::<Vec<_>>();
        codes.sort_unstable();
        let mut sequence = serializer.serialize_seq(Some(codes.len()))?;
        for code in codes {
            sequence.serialize_element(code)?;
        }
        sequence.end()
    }
}

impl<'de> Deserialize<'de> for CapabilitySet {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct CapabilitySetVisitor;

        impl<'de> Visitor<'de> for CapabilitySetVisitor {
            type Value = CapabilitySet;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a bounded capability token array")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                if seq.size_hint().unwrap_or(0) > MAX_CAPABILITY_SET_LEN {
                    return Err(serde::de::Error::custom(
                        "capability set exceeds entry bound",
                    ));
                }
                let mut tokens = Vec::new();
                while let Some(token) = seq.next_element()? {
                    if tokens.len() == MAX_CAPABILITY_SET_LEN {
                        return Err(serde::de::Error::custom(
                            "capability set exceeds entry bound",
                        ));
                    }
                    tokens.push(token);
                }
                Ok(CapabilitySet::from_tokens(tokens))
            }
        }

        deserializer.deserialize_seq(CapabilitySetVisitor)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WorkloadProviderKind {
    LocalVm,
    QemuMedia,
    ProviderManaged,
    UnsafeLocal,
}

impl WorkloadProviderKind {
    pub const fn metrics_label_value(self) -> &'static str {
        match self {
            Self::LocalVm => "local-vm",
            Self::QemuMedia => "qemu-media",
            Self::ProviderManaged => "provider-managed",
            Self::UnsafeLocal => "unsafe-local",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum IsolationPosture {
    VirtualMachine,
    ProviderManaged,
    UnsafeLocal,
}

impl IsolationPosture {
    pub const fn metrics_label_value(self) -> &'static str {
        match self {
            Self::VirtualMachine => "virtual-machine",
            Self::ProviderManaged => "provider-managed",
            Self::UnsafeLocal => "unsafe-local",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EnvironmentPosture {
    RuntimeManaged,
    SystemdUserManagerAmbient,
}

impl EnvironmentPosture {
    pub const fn metrics_label_value(self) -> &'static str {
        match self {
            Self::RuntimeManaged => "runtime-managed",
            Self::SystemdUserManagerAmbient => "systemd-user-manager-ambient",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DisplayEnvironmentPosture {
    RuntimeManaged,
    WaylandProxyOnly,
    NotApplicable,
}

impl DisplayEnvironmentPosture {
    pub const fn metrics_label_value(self) -> &'static str {
        match self {
            Self::RuntimeManaged => "runtime-managed",
            Self::WaylandProxyOnly => "wayland-proxy-only",
            Self::NotApplicable => "not-applicable",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExecutionIdentityPosture {
    WorkloadUser,
    ProviderManaged,
    AuthenticatedRequesterUid,
}

impl ExecutionIdentityPosture {
    pub const fn metrics_label_value(self) -> &'static str {
        match self {
            Self::WorkloadUser => "workload-user",
            Self::ProviderManaged => "provider-managed",
            Self::AuthenticatedRequesterUid => "authenticated-requester-uid",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SessionPersistencePosture {
    RuntimeManaged,
    UserManagerLifetime,
}

impl SessionPersistencePosture {
    pub const fn metrics_label_value(self) -> &'static str {
        match self {
            Self::RuntimeManaged => "runtime-managed",
            Self::UserManagerLifetime => "user-manager-lifetime",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorkloadExecutionPosture {
    pub isolation: IsolationPosture,
    pub environment: EnvironmentPosture,
    pub display_environment: DisplayEnvironmentPosture,
    pub execution_identity: ExecutionIdentityPosture,
    pub session_persistence: SessionPersistencePosture,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LauncherItemKind {
    Exec,
    Shell,
}

impl LauncherItemKind {
    pub const fn metrics_label_value(self) -> &'static str {
        match self {
            Self::Exec => "exec",
            Self::Shell => "shell",
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LauncherIcon {
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_optional_icon"
    )]
    pub id: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_optional_icon"
    )]
    pub name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LauncherItemSummary {
    pub id: ProtocolToken,
    #[serde(deserialize_with = "deserialize_launcher_name")]
    pub name: String,
    #[serde(default)]
    pub icon: LauncherIcon,
    #[serde(rename = "type")]
    pub kind: LauncherItemKind,
    #[serde(default)]
    pub graphical: bool,
    #[serde(default)]
    pub capabilities: CapabilitySet,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WorkloadState {
    Stopped,
    Starting,
    Running,
    Stopping,
    Failed,
}

impl WorkloadState {
    pub const fn metrics_label_value(self) -> &'static str {
        match self {
            Self::Stopped => "stopped",
            Self::Starting => "starting",
            Self::Running => "running",
            Self::Stopping => "stopping",
            Self::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WorkloadAvailability {
    Ready,
    HelperUnavailable,
    HelperStale,
    UserManagerUnavailable,
    GraphicalSessionInactive,
    WaylandUnavailable,
    ProxyUnavailable,
    Degraded,
}

impl WorkloadAvailability {
    pub const fn metrics_label_value(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::HelperUnavailable => "helper-unavailable",
            Self::HelperStale => "helper-stale",
            Self::UserManagerUnavailable => "user-manager-unavailable",
            Self::GraphicalSessionInactive => "graphical-session-inactive",
            Self::WaylandUnavailable => "wayland-unavailable",
            Self::ProxyUnavailable => "proxy-unavailable",
            Self::Degraded => "degraded",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum GraphicalLaunchPosture {
    Proxied,
    NotApplicable,
    GraphicalSessionInactive,
    WaylandUnavailable,
    ProxyUnavailable,
}

impl GraphicalLaunchPosture {
    pub const fn metrics_label_value(self) -> &'static str {
        match self {
            Self::Proxied => "proxied",
            Self::NotApplicable => "not-applicable",
            Self::GraphicalSessionInactive => "graphical-session-inactive",
            Self::WaylandUnavailable => "wayland-unavailable",
            Self::ProxyUnavailable => "proxy-unavailable",
        }
    }
}

struct BoundedVecVisitor<T, const MAX: usize> {
    expected: &'static str,
    marker: PhantomData<T>,
}

impl<'de, T, const MAX: usize> Visitor<'de> for BoundedVecVisitor<T, MAX>
where
    T: Deserialize<'de>,
{
    type Value = Vec<T>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.expected)
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        if seq.size_hint().unwrap_or(0) > MAX {
            return Err(serde::de::Error::custom("array exceeds entry bound"));
        }
        let mut values = Vec::new();
        while let Some(value) = seq.next_element()? {
            if values.len() == MAX {
                return Err(serde::de::Error::custom("array exceeds entry bound"));
            }
            values.push(value);
        }
        Ok(values)
    }
}

fn deserialize_bounded_vec<'de, D, T, const MAX: usize>(
    deserializer: D,
    expected: &'static str,
) -> Result<Vec<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    deserializer.deserialize_seq(BoundedVecVisitor::<T, MAX> {
        expected,
        marker: PhantomData,
    })
}

fn deserialize_launcher_items<'de, D>(deserializer: D) -> Result<Vec<LauncherItemSummary>, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_bounded_vec::<D, LauncherItemSummary, MAX_LAUNCHER_ITEMS_PER_WORKLOAD>(
        deserializer,
        "a bounded launcher item array",
    )
}

fn deserialize_workloads<'de, D>(deserializer: D) -> Result<Vec<WorkloadPublicSummary>, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_bounded_vec::<D, WorkloadPublicSummary, MAX_WORKLOADS_PER_RESPONSE>(
        deserializer,
        "a bounded workload summary array",
    )
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorkloadPublicSummary {
    pub identity: WorkloadIdentity,
    pub provider_kind: WorkloadProviderKind,
    pub state: WorkloadState,
    pub execution_posture: WorkloadExecutionPosture,
    pub availability: WorkloadAvailability,
    pub graphical_posture: GraphicalLaunchPosture,
    #[serde(default)]
    pub capabilities: CapabilitySet,
    #[serde(default, deserialize_with = "deserialize_launcher_items")]
    pub launcher_items: Vec<LauncherItemSummary>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_item_id: Option<ProtocolToken>,
}

impl WorkloadPublicSummary {
    pub fn select_launcher_item(
        &self,
        explicit_item: Option<&ProtocolToken>,
    ) -> Result<&LauncherItemSummary, ToolkitError> {
        if let Some(explicit_item) = explicit_item {
            return self
                .launcher_items
                .iter()
                .find(|item| item.id == *explicit_item)
                .ok_or(ToolkitError::LauncherItemUnavailable);
        }
        if let Some(default_item) = self.default_item_id.as_ref() {
            return self
                .launcher_items
                .iter()
                .find(|item| item.id == *default_item)
                .ok_or(ToolkitError::LauncherItemUnavailable);
        }
        match self.launcher_items.as_slice() {
            [item] => Ok(item),
            [] => Err(ToolkitError::LauncherItemUnavailable),
            items => {
                let candidates = items
                    .iter()
                    .take(MAX_LAUNCHER_ITEMS_PER_WORKLOAD)
                    .map(|item| {
                        LauncherItemCandidate::new(
                            item.id.as_str().to_owned(),
                            bounded_candidate_name(&item.name),
                        )
                    })
                    .collect();
                Err(ToolkitError::AmbiguousLauncherItems {
                    candidates: LauncherItemCandidates::new(candidates),
                })
            }
        }
    }
}

fn bounded_candidate_name(value: &str) -> String {
    let mut bounded = String::new();
    for ch in value.chars() {
        if ch.is_control() || bounded.len() + ch.len_utf8() > MAX_PRESENTATION_TEXT_LEN {
            break;
        }
        bounded.push(ch);
    }
    bounded
}

fn validate_realm_filter(value: &str) -> Result<(), ToolkitError> {
    if value.is_empty() {
        return Err(ToolkitError::InvalidIdentity {
            field: IdentityField::RealmPath,
            reason: ValidationReason::Empty,
        });
    }
    if value.len() > MAX_REALM_PATH_BYTES {
        return Err(ToolkitError::InvalidIdentity {
            field: IdentityField::RealmPath,
            reason: ValidationReason::TooLong,
        });
    }
    let labels = value.split('.').collect::<Vec<_>>();
    if labels.is_empty()
        || labels.len() > MAX_REALM_LABELS
        || labels
            .iter()
            .any(|label| *label == "d2b" || !is_label(label))
    {
        return Err(ToolkitError::InvalidIdentity {
            field: IdentityField::RealmPath,
            reason: ValidationReason::BadShape,
        });
    }
    Ok(())
}

fn deserialize_optional_realm_filter<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<String>::deserialize(deserializer)?;
    if let Some(value) = value.as_deref() {
        validate_realm_filter(value).map_err(serde::de::Error::custom)?;
    }
    Ok(value)
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorkloadListArgs {
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_optional_realm_filter"
    )]
    pub realm: Option<String>,
}

impl WorkloadListArgs {
    pub fn new(realm: Option<String>) -> Result<Self, ToolkitError> {
        if let Some(realm) = realm.as_deref() {
            validate_realm_filter(realm)?;
        }
        Ok(Self { realm })
    }

    pub fn inventory() -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorkloadStatusArgs {
    pub target: WorkloadTarget,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LauncherExecArgs {
    pub target: WorkloadTarget,
    pub item_id: ProtocolToken,
    pub operation_id: OperationId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", content = "args", rename_all = "camelCase")]
pub enum WorkloadOp {
    List(WorkloadListArgs),
    Status(WorkloadStatusArgs),
    LauncherExec(LauncherExecArgs),
}

impl WorkloadOp {
    pub const fn metrics_label_value(&self) -> &'static str {
        match self {
            Self::List(_) => "list",
            Self::Status(_) => "status",
            Self::LauncherExec(_) => "launcher-exec",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", content = "result", rename_all = "camelCase")]
pub enum WorkloadOpResponse {
    List(WorkloadListResult),
    Status(Box<WorkloadStatusResult>),
    LauncherExec(LauncherExecResult),
}

impl WorkloadOpResponse {
    pub const fn metrics_label_value(&self) -> &'static str {
        match self {
            Self::List(_) => "list",
            Self::Status(_) => "status",
            Self::LauncherExec(_) => "launcher-exec",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorkloadListResult {
    #[serde(deserialize_with = "deserialize_workloads")]
    pub workloads: Vec<WorkloadPublicSummary>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WorkloadStatusResult {
    pub workload: WorkloadPublicSummary,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum LauncherExecDisposition {
    Committed,
    AlreadyCommitted,
}

impl LauncherExecDisposition {
    pub const fn metrics_label_value(self) -> &'static str {
        match self {
            Self::Committed => "committed",
            Self::AlreadyCommitted => "already-committed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LauncherExecResult {
    pub target: WorkloadTarget,
    pub item_id: ProtocolToken,
    pub operation_id: OperationId,
    pub disposition: LauncherExecDisposition,
}

pub fn validate_shell_target(value: &str) -> Result<(), ToolkitError> {
    if value.contains('.') || value.starts_with("d2b://") {
        WorkloadTarget::parse(value).map(|_| ())
    } else {
        WorkloadId::parse(value)
            .map(|_| ())
            .map_err(|_| ToolkitError::InvalidTarget {
                reason: if value.len() > MAX_ID_LEN {
                    ValidationReason::TooLong
                } else if value.is_empty() {
                    ValidationReason::Empty
                } else {
                    ValidationReason::BadShape
                },
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_target_normalizes_optional_scheme() {
        let target = WorkloadTarget::parse("d2b://browser.work.d2b").unwrap();
        assert_eq!(target.as_str(), "browser.work.d2b");
        assert_eq!(target.workload().as_str(), "browser");
        assert_eq!(target.realm().target_form(), "work");
    }

    #[test]
    fn unknown_capabilities_round_trip_deterministically() {
        let capabilities: CapabilitySet =
            serde_json::from_str(r#"["future-window-mode","pty","configured-launch"]"#).unwrap();
        assert!(capabilities.has(Capability::Pty));
        assert_eq!(
            capabilities
                .unknown_iter()
                .map(ProtocolToken::as_str)
                .collect::<Vec<_>>(),
            vec!["future-window-mode"]
        );
        assert_eq!(
            serde_json::to_string(&capabilities).unwrap(),
            r#"["configured-launch","future-window-mode","pty"]"#
        );
    }

    #[test]
    fn operation_ids_are_redacted() {
        let operation_id = OperationId::parse("launch-123").unwrap();
        assert_eq!(format!("{operation_id:?}"), "OperationId([redacted])");
        assert_eq!(operation_id.to_string(), "[redacted]");
        assert_eq!(
            serde_json::to_string(&operation_id).unwrap(),
            r#""launch-123""#
        );
    }

    #[test]
    fn bounded_decoders_reject_oversize_arrays() {
        let capabilities = format!(
            "[{}]",
            std::iter::repeat(r#""exec""#)
                .take(MAX_CAPABILITY_SET_LEN + 1)
                .collect::<Vec<_>>()
                .join(",")
        );
        assert!(serde_json::from_str::<CapabilitySet>(&capabilities).is_err());
    }
}
