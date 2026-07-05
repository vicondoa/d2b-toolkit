use crate::redaction::{OpaqueHandle, SensitiveString, TerminalBytes};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ShellName(String);

impl ShellName {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Low-cardinality metrics label. The concrete shell name is never a label value.
    pub fn metrics_label_value(&self) -> &'static str {
        "shell"
    }
}

impl fmt::Debug for ShellName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ShellName").field(&"[redacted]").finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "kebab-case")]
pub enum ShellOp {
    List {
        vm: String,
    },
    Attach {
        vm: String,
        name: Option<ShellName>,
        force: bool,
        rows: u32,
        cols: u32,
    },
    Write {
        handle: OpaqueHandle,
        bytes: TerminalBytes,
    },
    Resize {
        handle: OpaqueHandle,
        cols: u16,
        rows: u16,
    },
    Detach {
        vm: String,
        name: Option<ShellName>,
    },
    Kill {
        vm: String,
        name: ShellName,
    },
    CloseAttach {
        handle: OpaqueHandle,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "response", rename_all = "kebab-case")]
pub enum ShellOpResponse {
    List {
        sessions: Vec<ShellName>,
    },
    Attached {
        handle: OpaqueHandle,
    },
    Output {
        handle: OpaqueHandle,
        bytes: TerminalBytes,
    },
    Exited {
        handle: OpaqueHandle,
        status: i32,
    },
    Failed {
        handle: Option<OpaqueHandle>,
        message: SensitiveString,
    },
}

#[cfg(test)]
mod tests {
    use super::{ShellName, ShellOp};
    #[test]
    fn shell_debug_redacts_process_boundary_payloads() {
        let op = ShellOp::Attach {
            vm: "work".to_string(),
            name: Some(ShellName::new("project-shell")),
            force: false,
            rows: 24,
            cols: 80,
        };

        let debug = format!("{op:?}");
        assert!(!debug.contains("project-shell"));
        assert!(debug.contains("redacted"));
    }

    #[test]
    fn shell_name_is_not_metric_label() {
        let shell = ShellName::new("customer-specific-shell");
        assert_eq!(shell.metrics_label_value(), "shell");
    }
}
