use crate::redaction::{OpaqueHandle, SensitiveString};
use serde::{Deserialize, Deserializer, Serialize};
use std::fmt;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct ShellName(String);

impl ShellName {
    pub fn new(value: impl Into<String>) -> Result<Self, ShellNameError> {
        let value = value.into();
        if shell_name_valid(&value) {
            Ok(Self(value))
        } else {
            Err(ShellNameError)
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn metrics_label_value(&self) -> &'static str {
        "shell"
    }
}

impl fmt::Debug for ShellName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ShellName").field(&"[redacted]").finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShellNameError;

impl fmt::Display for ShellNameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("shell name has an invalid shape")
    }
}

impl std::error::Error for ShellNameError {}

impl<'de> Deserialize<'de> for ShellName {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::new(String::deserialize(deserializer)?)
            .map_err(|_| serde::de::Error::custom("shell name has an invalid shape"))
    }
}

fn shell_name_valid(value: &str) -> bool {
    let bytes = value.as_bytes();
    !bytes.is_empty()
        && bytes.len() <= 64
        && (bytes[0].is_ascii_alphanumeric() || bytes[0] == b'_')
        && bytes[1..]
            .iter()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
}

fn deserialize_shell_target<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = String::deserialize(deserializer)?;
    crate::workload::validate_shell_target(&value).map_err(serde::de::Error::custom)?;
    Ok(value)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TerminalStream {
    Stdout,
    Stderr,
}

impl TerminalStream {
    pub const fn metrics_label_value(self) -> &'static str {
        match self {
            Self::Stdout => "stdout",
            Self::Stderr => "stderr",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TerminalSize {
    pub rows: u32,
    pub cols: u32,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TerminalWriteStdin {
    pub session: OpaqueHandle,
    pub offset: u64,
    pub chunk_base64: SensitiveString,
    #[serde(default)]
    pub eof: bool,
}

impl fmt::Debug for TerminalWriteStdin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TerminalWriteStdin")
            .field("session", &self.session)
            .field("offset", &self.offset)
            .field("chunk_base64", &"[redacted]")
            .field("eof", &self.eof)
            .finish()
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TerminalReadOutput {
    pub session: OpaqueHandle,
    pub stream: TerminalStream,
    pub offset: u64,
    pub max_len: u64,
    #[serde(default)]
    pub wait: bool,
    #[serde(default)]
    pub timeout_ms: u64,
}

impl fmt::Debug for TerminalReadOutput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TerminalReadOutput")
            .field("session", &self.session)
            .field("stream", &self.stream)
            .field("offset", &self.offset)
            .field("max_len", &self.max_len)
            .field("wait", &self.wait)
            .field("timeout_ms", &self.timeout_ms)
            .finish()
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TerminalResize {
    pub session: OpaqueHandle,
    pub rows: u32,
    pub cols: u32,
    #[serde(default)]
    pub op_id: u64,
}

impl fmt::Debug for TerminalResize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TerminalResize")
            .field("session", &self.session)
            .field("rows", &self.rows)
            .field("cols", &self.cols)
            .field("op_id", &self.op_id)
            .finish()
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TerminalWait {
    pub session: OpaqueHandle,
    #[serde(default)]
    pub timeout_ms: u64,
}

impl fmt::Debug for TerminalWait {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TerminalWait")
            .field("session", &self.session)
            .field("timeout_ms", &self.timeout_ms)
            .finish()
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TerminalClose {
    pub session: OpaqueHandle,
}

impl fmt::Debug for TerminalClose {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TerminalClose")
            .field("session", &self.session)
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TerminalWriteStdinResult {
    pub accepted_len: u64,
    pub next_offset: u64,
    #[serde(default)]
    pub backpressured: bool,
    #[serde(default)]
    pub stdin_closed: bool,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TerminalReadOutputChunk {
    pub data_base64: SensitiveString,
    pub next_offset: u64,
    #[serde(default)]
    pub eof: bool,
    #[serde(default)]
    pub dropped_bytes: u64,
    #[serde(default)]
    pub truncated: bool,
    #[serde(default)]
    pub timed_out: bool,
}

impl fmt::Debug for TerminalReadOutputChunk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TerminalReadOutputChunk")
            .field("data_base64", &"[redacted]")
            .field("next_offset", &self.next_offset)
            .field("eof", &self.eof)
            .field("dropped_bytes", &self.dropped_bytes)
            .field("truncated", &self.truncated)
            .field("timed_out", &self.timed_out)
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TerminalControlResult {
    #[serde(default)]
    pub delivered: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "camelCase")]
pub enum TerminalStatus {
    Exited { code: i32 },
    Signaled { signal: u32 },
    Error { slug: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TerminalWaitResult {
    #[serde(default)]
    pub running: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub terminal_status: Option<TerminalStatus>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TerminalCloseResult {
    #[serde(default)]
    pub stdin_closed: bool,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ShellAttachArgs {
    #[serde(deserialize_with = "deserialize_shell_target")]
    pub vm: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<ShellName>,
    #[serde(default)]
    pub force: bool,
    pub initial_terminal_size: TerminalSize,
}

impl fmt::Debug for ShellAttachArgs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ShellAttachArgs")
            .field("vm", &self.vm)
            .field("has_name", &self.name.is_some())
            .field("force", &self.force)
            .field("initial_terminal_size", &self.initial_terminal_size)
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ShellListArgs {
    #[serde(deserialize_with = "deserialize_shell_target")]
    pub vm: String,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ShellDetachArgs {
    #[serde(deserialize_with = "deserialize_shell_target")]
    pub vm: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<ShellName>,
}

impl fmt::Debug for ShellDetachArgs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ShellDetachArgs")
            .field("vm", &self.vm)
            .field("has_name", &self.name.is_some())
            .finish()
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ShellKillArgs {
    #[serde(deserialize_with = "deserialize_shell_target")]
    pub vm: String,
    pub name: ShellName,
}

impl fmt::Debug for ShellKillArgs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ShellKillArgs")
            .field("vm", &self.vm)
            .field("name", &"<redacted>")
            .finish()
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ShellCloseAttachArgs {
    pub session: OpaqueHandle,
}

impl fmt::Debug for ShellCloseAttachArgs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ShellCloseAttachArgs")
            .field("session", &self.session)
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", content = "args", rename_all = "camelCase")]
pub enum ShellOp {
    Attach(ShellAttachArgs),
    WriteStdin(TerminalWriteStdin),
    ReadOutput(TerminalReadOutput),
    Resize(TerminalResize),
    Wait(TerminalWait),
    CloseStdin(TerminalClose),
    CloseAttach(ShellCloseAttachArgs),
    List(ShellListArgs),
    Detach(ShellDetachArgs),
    Kill(ShellKillArgs),
}

impl ShellOp {
    pub const fn metrics_label_value(&self) -> &'static str {
        match self {
            Self::Attach(_) => "attach",
            Self::WriteStdin(_) => "write-stdin",
            Self::ReadOutput(_) => "read-output",
            Self::Resize(_) => "resize",
            Self::Wait(_) => "wait",
            Self::CloseStdin(_) => "close-stdin",
            Self::CloseAttach(_) => "close-attach",
            Self::List(_) => "list",
            Self::Detach(_) => "detach",
            Self::Kill(_) => "kill",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ShellSessionState {
    Attached,
    Detached,
    Killed,
    PoolUnavailable,
    FeatureDisabled,
    OutputGap,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ShellCloseCause {
    ClientDetach,
    EvictedByForce,
    EvictedByAdminDetach,
    KilledByAdmin,
    PoolUnavailable,
    OutputGap,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ShellAttachResult {
    pub session: OpaqueHandle,
    pub resolved_name: ShellName,
    pub state: ShellSessionState,
    #[serde(default)]
    pub force_evicted: bool,
}

impl fmt::Debug for ShellAttachResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ShellAttachResult")
            .field("session", &self.session)
            .field("resolved_name", &"<redacted>")
            .field("state", &self.state)
            .field("force_evicted", &self.force_evicted)
            .finish()
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ShellListEntry {
    pub name: ShellName,
    pub state: ShellSessionState,
    #[serde(default)]
    pub attached: bool,
    #[serde(default)]
    pub is_default: bool,
}

impl fmt::Debug for ShellListEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ShellListEntry")
            .field("name", &"<redacted>")
            .field("state", &self.state)
            .field("attached", &self.attached)
            .field("is_default", &self.is_default)
            .finish()
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ShellListResult {
    pub default_name: ShellName,
    pub sessions: Vec<ShellListEntry>,
}

impl fmt::Debug for ShellListResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ShellListResult")
            .field("default_name", &"<redacted>")
            .field("sessions_len", &self.sessions.len())
            .finish()
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ShellDetachResult {
    pub resolved_name: ShellName,
    pub detached: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cause: Option<ShellCloseCause>,
}

impl fmt::Debug for ShellDetachResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ShellDetachResult")
            .field("resolved_name", &"<redacted>")
            .field("detached", &self.detached)
            .field("cause", &self.cause)
            .finish()
    }
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ShellKillResult {
    pub name: ShellName,
    pub killed: bool,
    pub state: ShellSessionState,
}

impl fmt::Debug for ShellKillResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ShellKillResult")
            .field("name", &"<redacted>")
            .field("killed", &self.killed)
            .field("state", &self.state)
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", content = "result", rename_all = "camelCase")]
pub enum ShellOpResponse {
    Attach(ShellAttachResult),
    WriteStdin(TerminalWriteStdinResult),
    ReadOutput(TerminalReadOutputChunk),
    Resize(TerminalControlResult),
    Wait(TerminalWaitResult),
    CloseStdin(TerminalCloseResult),
    CloseAttach(ShellDetachResult),
    List(ShellListResult),
    Detach(ShellDetachResult),
    Kill(ShellKillResult),
}

#[cfg(test)]
mod tests {
    use super::{ShellName, ShellOp, TerminalStream};

    #[test]
    fn shell_debug_redacts_process_boundary_payloads() {
        let op = ShellOp::Attach(super::ShellAttachArgs {
            vm: "work".to_string(),
            name: Some(ShellName::new("project-shell").unwrap()),
            force: false,
            initial_terminal_size: super::TerminalSize { rows: 24, cols: 80 },
        });

        let debug = format!("{op:?}");
        assert!(!debug.contains("project-shell"));
        assert!(debug.contains("has_name"));
    }

    #[test]
    fn shell_name_is_not_metric_label() {
        let shell = ShellName::new("customer-specific-shell").unwrap();
        assert_eq!(shell.metrics_label_value(), "shell");
        assert_eq!(TerminalStream::Stdout.metrics_label_value(), "stdout");
    }
}
