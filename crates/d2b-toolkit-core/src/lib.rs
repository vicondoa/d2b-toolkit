#![forbid(unsafe_code)]

//! Shared DTOs and redaction primitives for d2b toolkit crates.

pub mod error;
pub mod hello;
pub mod public;
pub mod redaction;
pub mod shell;
pub mod socket;

pub use error::ToolkitError;
pub use hello::{Feature, Hello, HelloResponse, CURRENT_PROTOCOL_VERSION, PROTOCOL_NAME};
pub use public::{PublicRequest, PublicResponse};
pub use redaction::{
    CorrelationId, OpaqueHandle, Redacted, SensitiveString, TerminalBytes, REDACTED,
};
pub use shell::{ShellName, ShellOp, ShellOpResponse};
pub use socket::SocketClass;
