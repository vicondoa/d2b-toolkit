#![forbid(unsafe_code)]

//! Runtime-agnostic client primitives for d2b's public daemon protocol.
//!
//! This crate is intentionally socket-free: callers provide types implementing
//! `futures::io::AsyncRead` / `AsyncWrite`, so Tokio, async-std, smol, and test
//! transports can all own the concrete socket integration.

pub mod error;
pub mod frame;
pub mod hello;
pub mod shell_owner;
pub mod socket_policy;

pub use error::ClientError;
pub use frame::{read_frame, read_json_frame, write_frame, write_json_frame, FrameBounds};
pub use hello::{read_hello_response, send_hello, validate_hello_response};
pub use shell_owner::{ShellOwnerBoundary, ShellOwnerSink, ShellOwnerStream};
pub use socket_policy::ensure_allowed_socket;
