#![forbid(unsafe_code)]

//! Shared DTOs and redaction primitives for d2b toolkit crates.

pub mod error;
pub mod hello;
pub mod public;
pub mod redaction;
pub mod shell;
pub mod socket;

pub use error::ToolkitError;
pub use hello::{
    FeatureFlag, Hello, HelloFrame, HelloOk, HelloRejected, HelloRejectedReason, HelloResponse,
    KnownFeatureFlag, SemverRange, Version, CURRENT_PROTOCOL_VERSION, PROTOCOL_NAME,
};
pub use public::{ErrorEnvelope, PublicRequest, PublicResponse};
pub use redaction::{
    CorrelationId, OpaqueHandle, Redacted, SensitiveString, TerminalBytes, REDACTED,
};
pub use shell::{
    ShellAttachArgs, ShellAttachResult, ShellCloseAttachArgs, ShellCloseCause, ShellDetachArgs,
    ShellDetachResult, ShellKillArgs, ShellKillResult, ShellListArgs, ShellListEntry,
    ShellListResult, ShellName, ShellOp, ShellOpResponse, ShellSessionState, TerminalClose,
    TerminalCloseResult, TerminalControlResult, TerminalReadOutput, TerminalReadOutputChunk,
    TerminalResize, TerminalSize, TerminalStatus, TerminalStream, TerminalWait, TerminalWaitResult,
    TerminalWriteStdin, TerminalWriteStdinResult,
};
pub use socket::SocketClass;
