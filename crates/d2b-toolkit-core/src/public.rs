use crate::shell::{ShellOp, ShellOpResponse};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ErrorEnvelope {
    pub kind: String,
    pub exit_code: u8,
    pub message: String,
    pub remediation: String,
}

impl fmt::Debug for ErrorEnvelope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ErrorEnvelope")
            .field("kind", &self.kind)
            .field("exit_code", &self.exit_code)
            .finish_non_exhaustive()
    }
}

impl ErrorEnvelope {
    pub fn metrics_label_value(&self) -> &str {
        &self.kind
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PublicRequest {
    Shell { op_id: Option<u64>, op: ShellOp },
}

impl PublicRequest {
    pub fn shell(op_id: Option<u64>, op: ShellOp) -> Self {
        Self::Shell { op_id, op }
    }

    pub const fn type_label(&self) -> &'static str {
        match self {
            Self::Shell { .. } => "shell",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PublicResponse {
    Shell {
        op_id: Option<u64>,
        response: ShellOpResponse,
    },
    Error {
        op_id: Option<u64>,
        error: ErrorEnvelope,
    },
}

impl PublicResponse {
    pub const fn type_label(&self) -> &'static str {
        match self {
            Self::Shell { .. } => "shellResponse",
            Self::Error { .. } => "error",
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ShellRequestFrame {
    kind: PublicRequestType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    op_id: Option<u64>,
    payload: ShellOp,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ShellResponseFrame {
    #[serde(rename = "type")]
    type_name: PublicResponseType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    op_id: Option<u64>,
    #[serde(flatten)]
    response: ShellOpResponse,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ErrorFrame {
    #[serde(rename = "type")]
    type_name: PublicResponseType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    op_id: Option<u64>,
    error: ErrorEnvelope,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
enum PublicRequestType {
    Shell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
enum PublicResponseType {
    ShellResponse,
    Error,
}

impl Serialize for PublicRequest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Shell { op_id, op } => ShellRequestFrame {
                kind: PublicRequestType::Shell,
                op_id: *op_id,
                payload: op.clone(),
            }
            .serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for PublicRequest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let frame = ShellRequestFrame::deserialize(deserializer)?;
        Ok(Self::Shell {
            op_id: frame.op_id,
            op: frame.payload,
        })
    }
}

impl Serialize for PublicResponse {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Shell { op_id, response } => ShellResponseFrame {
                type_name: PublicResponseType::ShellResponse,
                op_id: *op_id,
                response: response.clone(),
            }
            .serialize(serializer),
            Self::Error { op_id, error } => ErrorFrame {
                type_name: PublicResponseType::Error,
                op_id: *op_id,
                error: error.clone(),
            }
            .serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for PublicResponse {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        let type_name = value
            .get("type")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| serde::de::Error::custom("missing response type"))?;
        match type_name {
            "shellResponse" => {
                let frame =
                    ShellResponseFrame::deserialize(value).map_err(serde::de::Error::custom)?;
                Ok(Self::Shell {
                    op_id: frame.op_id,
                    response: frame.response,
                })
            }
            "error" => {
                let frame = ErrorFrame::deserialize(value).map_err(serde::de::Error::custom)?;
                Ok(Self::Error {
                    op_id: frame.op_id,
                    error: frame.error,
                })
            }
            other => Err(serde::de::Error::custom(format!(
                "unsupported response type {other}"
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ErrorEnvelope;

    #[test]
    fn error_envelope_debug_omits_message_and_remediation() {
        let envelope = ErrorEnvelope {
            kind: "guest-control-shell-timeout".into(),
            exit_code: 69,
            message: "contains opaque-session-handle".into(),
            remediation: "contains /run/d2b/public.sock".into(),
        };
        let debug = format!("{envelope:?}");
        assert!(debug.contains("guest-control-shell-timeout"));
        assert!(!debug.contains("opaque-session-handle"));
        assert!(!debug.contains("/run/d2b"));
    }
}
