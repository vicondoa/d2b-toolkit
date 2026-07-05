use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::hash_map::DefaultHasher;
use std::fmt;
use std::hash::{Hash, Hasher};

pub const REDACTED: &str = "[redacted]";

/// Wire-serializable sensitive value whose Debug/Display output is always redacted.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Redacted<T>(T);

impl<T> Redacted<T> {
    pub fn new(value: T) -> Self {
        Self(value)
    }

    pub fn into_inner_for_wire(self) -> T {
        self.0
    }
}

impl<T> fmt::Debug for Redacted<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(REDACTED)
    }
}

impl<T> fmt::Display for Redacted<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(REDACTED)
    }
}

pub type SensitiveString = Redacted<String>;
pub type TerminalBytes = Redacted<Vec<u8>>;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CorrelationId(String);

impl CorrelationId {
    pub fn from_non_reversible_digest(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn from_opaque_handle(handle: &OpaqueHandle) -> Self {
        let mut hasher = DefaultHasher::new();
        "d2b-toolkit-opaque-handle".hash(&mut hasher);
        handle.0.hash(&mut hasher);
        Self(format!("{:016x}", hasher.finish()))
    }
}

impl fmt::Debug for CorrelationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("CorrelationId").field(&"<digest>").finish()
    }
}

impl fmt::Display for CorrelationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<digest>")
    }
}

/// Opaque protocol handle. It serializes on the wire but never appears in Debug/Display.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct OpaqueHandle(String);

impl OpaqueHandle {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn into_inner_for_wire(self) -> String {
        self.0
    }
}

impl fmt::Debug for OpaqueHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("OpaqueHandle(")?;
        f.write_str(REDACTED)?;
        f.write_str(")")
    }
}

impl fmt::Display for OpaqueHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(REDACTED)
    }
}

impl Serialize for OpaqueHandle {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for OpaqueHandle {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer).map(Self)
    }
}

#[cfg(test)]
mod tests {
    use super::{CorrelationId, OpaqueHandle, Redacted};

    #[test]
    fn debug_and_display_redact_payloads() {
        let secret = Redacted::new("argv --token=secret".to_string());
        assert_eq!(format!("{secret:?}"), "[redacted]");
        assert_eq!(secret.to_string(), "[redacted]");

        let handle = OpaqueHandle::new("opaque-session-handle");
        assert!(!format!("{handle:?}").contains("opaque-session-handle"));
        assert_eq!(handle.to_string(), "[redacted]");
    }

    #[test]
    fn correlation_id_does_not_expose_raw_digest_value() {
        let correlation = CorrelationId::from_non_reversible_digest("raw-session-handle-digest");
        assert!(!format!("{correlation:?}").contains("raw-session-handle-digest"));
        assert_eq!(correlation.to_string(), "<digest>");
    }

    #[test]
    fn correlation_from_handle_does_not_expose_handle() {
        let handle = OpaqueHandle::new("opaque-session-handle");
        let correlation = CorrelationId::from_opaque_handle(&handle);
        assert!(!format!("{correlation:?}").contains("opaque-session-handle"));
        assert!(!correlation.0.contains("opaque-session-handle"));
    }
}
