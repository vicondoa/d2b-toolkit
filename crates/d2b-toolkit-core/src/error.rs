use std::io;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationReason {
    Empty,
    TooLong,
    BadShape,
    TooManyItems,
    Inconsistent,
}

impl std::fmt::Display for ValidationReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Empty => "empty",
            Self::TooLong => "too-long",
            Self::BadShape => "bad-shape",
            Self::TooManyItems => "too-many-items",
            Self::Inconsistent => "inconsistent",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Feature,
    Protocol,
    Operation,
}

impl std::fmt::Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Feature => "feature",
            Self::Protocol => "protocol",
            Self::Operation => "operation",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdentityField {
    WorkloadId,
    WorkloadName,
    RealmId,
    RealmPath,
    LegacyVmName,
    RuntimeKind,
    ProviderId,
    LauncherName,
    LauncherIcon,
}

impl std::fmt::Display for IdentityField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::WorkloadId => "workload-id",
            Self::WorkloadName => "workload-name",
            Self::RealmId => "realm-id",
            Self::RealmPath => "realm-path",
            Self::LegacyVmName => "legacy-vm-name",
            Self::RuntimeKind => "runtime-kind",
            Self::ProviderId => "provider-id",
            Self::LauncherName => "launcher-name",
            Self::LauncherIcon => "launcher-icon",
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LauncherItemCandidate {
    id: String,
    name: String,
}

impl LauncherItemCandidate {
    pub(crate) fn new(id: String, name: String) -> Self {
        Self { id, name }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LauncherItemCandidates(Vec<LauncherItemCandidate>);

impl LauncherItemCandidates {
    pub(crate) fn new(candidates: Vec<LauncherItemCandidate>) -> Self {
        Self(candidates)
    }

    pub fn as_slice(&self) -> &[LauncherItemCandidate] {
        &self.0
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl std::fmt::Display for LauncherItemCandidates {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} candidates", self.0.len())
    }
}

/// Core toolkit errors deliberately avoid carrying terminal bytes, argv, env,
/// cwd, opaque handles, or concrete socket paths.
#[derive(Debug, thiserror::Error)]
pub enum ToolkitError {
    #[error("protocol error: {kind}")]
    Protocol { kind: &'static str },

    #[error("frame too large ({len} bytes, max {max} bytes)")]
    FrameTooLarge { len: usize, max: usize },

    #[error("refused privileged broker socket")]
    PrivilegedBrokerRefused,

    #[error("required daemon feature is unavailable: {feature}")]
    FeatureUnavailable {
        feature: crate::hello::KnownFeatureFlag,
    },

    #[error("invalid workload target ({reason})")]
    InvalidTarget { reason: ValidationReason },

    #[error("invalid {kind} token ({reason})")]
    InvalidToken {
        kind: TokenKind,
        reason: ValidationReason,
    },

    #[error("invalid workload identity field {field} ({reason})")]
    InvalidIdentity {
        field: IdentityField,
        reason: ValidationReason,
    },

    #[error("launcher item is unavailable")]
    LauncherItemUnavailable,

    #[error("launcher item selection is ambiguous: {candidates}")]
    AmbiguousLauncherItems { candidates: LauncherItemCandidates },

    #[error("I/O error while {context}")]
    Io {
        context: &'static str,
        #[source]
        source: io::Error,
    },
}

#[cfg(test)]
mod tests {
    use super::{
        IdentityField, LauncherItemCandidate, LauncherItemCandidates, ToolkitError,
        ValidationReason,
    };
    use std::error::Error;

    fn assert_error_bounds<E: Error + Send + Sync + 'static>() {}

    #[test]
    fn error_is_send_sync_static() {
        assert_error_bounds::<ToolkitError>();
    }

    #[test]
    fn debug_does_not_include_sensitive_payloads() {
        let err = ToolkitError::Protocol { kind: "bad-frame" };
        let debug = format!("{err:?}");
        assert!(!debug.contains("super-secret-argv"));
        assert!(!debug.contains("opaque-session-handle"));
    }

    #[test]
    fn validation_errors_never_echo_rejected_values() {
        let err = ToolkitError::InvalidIdentity {
            field: IdentityField::ProviderId,
            reason: ValidationReason::BadShape,
        };
        let rendered = format!("{err:?} {err}");
        assert!(!rendered.contains("operation-secret-canary"));
        assert!(!rendered.contains("/sensitive/cwd"));
    }

    #[test]
    fn ambiguity_exposes_only_bounded_presentation_fields() {
        let err = ToolkitError::AmbiguousLauncherItems {
            candidates: LauncherItemCandidates::new(vec![LauncherItemCandidate::new(
                "browser".into(),
                "Firefox".into(),
            )]),
        };
        let debug = format!("{err:?}");
        assert!(debug.contains("browser"));
        assert!(debug.contains("Firefox"));
        assert_eq!(
            err.to_string(),
            "launcher item selection is ambiguous: 1 candidates"
        );
    }
}
