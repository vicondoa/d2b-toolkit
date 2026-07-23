#![forbid(unsafe_code)]

//! Shared DTOs and redaction primitives for d2b toolkit crates.

pub mod error;
pub mod hello;
pub mod public;
pub mod redaction;
pub mod shell;
pub mod socket;
pub mod workload;

pub use error::{
    IdentityField, LauncherItemCandidate, LauncherItemCandidates, TokenKind, ToolkitError,
    ValidationReason,
};
pub use hello::{
    FeatureFlag, Hello, HelloFrame, HelloOk, HelloRejected, HelloRejectedReason, HelloResponse,
    KnownFeatureFlag, NegotiatedCapabilities, SemverRange, Version, CURRENT_PROTOCOL_VERSION,
    MAX_FEATURE_FLAG_LEN, MAX_HELLO_FEATURES, PROTOCOL_NAME,
};
pub use public::{ErrorEnvelope, PublicRequest, PublicResponse};
pub use redaction::{
    CorrelationId, OpaqueHandle, Redacted, SensitiveString, TerminalBytes, REDACTED,
};
pub use shell::{
    ShellAttachArgs, ShellAttachResult, ShellCloseAttachArgs, ShellCloseCause, ShellDetachArgs,
    ShellDetachResult, ShellKillArgs, ShellKillResult, ShellListArgs, ShellListEntry,
    ShellListResult, ShellName, ShellNameError, ShellOp, ShellOpResponse, ShellSessionState,
    TerminalClose, TerminalCloseResult, TerminalControlResult, TerminalReadOutput,
    TerminalReadOutputChunk, TerminalResize, TerminalSize, TerminalStatus, TerminalStream,
    TerminalWait, TerminalWaitResult, TerminalWriteStdin, TerminalWriteStdinResult,
};
pub use socket::SocketClass;
pub use workload::{
    Capability, CapabilitySet, DisplayEnvironmentPosture, EnvironmentPosture,
    ExecutionIdentityPosture, GraphicalLaunchPosture, IsolationPosture, LauncherExecArgs,
    LauncherExecDisposition, LauncherExecResult, LauncherIcon, LauncherItemKind,
    LauncherItemSummary, LegacyVmName, OperationId, ProtocolToken, ProviderId, RealmId, RealmPath,
    RuntimeKind, SessionPersistencePosture, WorkloadAvailability, WorkloadExecutionPosture,
    WorkloadId, WorkloadIdentity, WorkloadListArgs, WorkloadListResult, WorkloadOp,
    WorkloadOpResponse, WorkloadProviderKind, WorkloadPublicSummary, WorkloadState,
    WorkloadStatusArgs, WorkloadStatusResult, WorkloadTarget, MAX_CAPABILITY_SET_LEN,
    MAX_IDENTITY_TOKEN_LEN, MAX_ID_LEN, MAX_LAUNCHER_ITEMS_PER_WORKLOAD, MAX_PRESENTATION_TEXT_LEN,
    MAX_PROTOCOL_TOKEN_LEN, MAX_REALM_LABELS, MAX_REALM_PATH_BYTES, MAX_WORKLOADS_PER_RESPONSE,
    MAX_WORKLOAD_TARGET_LEN,
};
