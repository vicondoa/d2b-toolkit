use crate::shell::{ShellOp, ShellOpResponse};
use crate::workload::{WorkloadOp, WorkloadOpResponse};
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
    pub const fn metrics_label_value(&self) -> &'static str {
        "daemon-error"
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PublicRequest {
    Shell { op_id: Option<u64>, op: ShellOp },
    Workload { op_id: Option<u64>, op: WorkloadOp },
}

impl PublicRequest {
    pub fn shell(op_id: Option<u64>, op: ShellOp) -> Self {
        Self::Shell { op_id, op }
    }

    pub fn workload(op_id: Option<u64>, op: WorkloadOp) -> Self {
        Self::Workload { op_id, op }
    }

    pub const fn type_label(&self) -> &'static str {
        match self {
            Self::Shell { .. } => "shell",
            Self::Workload { .. } => "workload",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PublicResponse {
    Shell {
        op_id: Option<u64>,
        response: ShellOpResponse,
    },
    Workload {
        op_id: Option<u64>,
        response: WorkloadOpResponse,
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
            Self::Workload { .. } => "workloadResponse",
            Self::Error { .. } => "error",
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ShellRequestFrame {
    #[serde(rename = "type")]
    type_name: PublicRequestType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    op_id: Option<u64>,
    #[serde(flatten)]
    op: ShellOp,
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
struct WorkloadRequestFrame {
    #[serde(rename = "type")]
    type_name: PublicRequestType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    op_id: Option<u64>,
    #[serde(flatten)]
    op: WorkloadOp,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct WorkloadResponseFrame {
    #[serde(rename = "type")]
    type_name: PublicResponseType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    op_id: Option<u64>,
    #[serde(flatten)]
    response: WorkloadOpResponse,
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
    Workload,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
enum PublicResponseType {
    ShellResponse,
    WorkloadResponse,
    Error,
}

impl Serialize for PublicRequest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Shell { op_id, op } => ShellRequestFrame {
                type_name: PublicRequestType::Shell,
                op_id: *op_id,
                op: op.clone(),
            }
            .serialize(serializer),
            Self::Workload { op_id, op } => WorkloadRequestFrame {
                type_name: PublicRequestType::Workload,
                op_id: *op_id,
                op: op.clone(),
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
        let value = serde_json::Value::deserialize(deserializer)?;
        let type_name = frame_type::<D::Error>(&value, "request")?;
        ensure_only_fields::<D::Error>(&value, &["type", "opId", "op", "args"])?;
        match type_name {
            "shell" => {
                let frame =
                    ShellRequestFrame::deserialize(value).map_err(serde::de::Error::custom)?;
                Ok(Self::Shell {
                    op_id: frame.op_id,
                    op: frame.op,
                })
            }
            "workload" => {
                let frame =
                    WorkloadRequestFrame::deserialize(value).map_err(serde::de::Error::custom)?;
                Ok(Self::Workload {
                    op_id: frame.op_id,
                    op: frame.op,
                })
            }
            _ => Err(serde::de::Error::custom("unsupported request type")),
        }
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
            Self::Workload { op_id, response } => WorkloadResponseFrame {
                type_name: PublicResponseType::WorkloadResponse,
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
        let type_name = frame_type::<D::Error>(&value, "response")?;
        match type_name {
            "shellResponse" => {
                ensure_only_fields::<D::Error>(&value, &["type", "opId", "op", "result"])?;
                let frame =
                    ShellResponseFrame::deserialize(value).map_err(serde::de::Error::custom)?;
                Ok(Self::Shell {
                    op_id: frame.op_id,
                    response: frame.response,
                })
            }
            "workloadResponse" => {
                ensure_only_fields::<D::Error>(&value, &["type", "opId", "op", "result"])?;
                let frame =
                    WorkloadResponseFrame::deserialize(value).map_err(serde::de::Error::custom)?;
                Ok(Self::Workload {
                    op_id: frame.op_id,
                    response: frame.response,
                })
            }
            "error" => {
                ensure_only_fields::<D::Error>(&value, &["type", "opId", "error"])?;
                let frame = ErrorFrame::deserialize(value).map_err(serde::de::Error::custom)?;
                Ok(Self::Error {
                    op_id: frame.op_id,
                    error: frame.error,
                })
            }
            _ => Err(serde::de::Error::custom("unsupported response type")),
        }
    }
}

fn frame_type<'a, E>(value: &'a serde_json::Value, frame: &str) -> Result<&'a str, E>
where
    E: serde::de::Error,
{
    value
        .as_object()
        .and_then(|object| object.get("type"))
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| E::custom(format!("missing {frame} type")))
}

fn ensure_only_fields<E>(value: &serde_json::Value, allowed: &[&str]) -> Result<(), E>
where
    E: serde::de::Error,
{
    let object = value
        .as_object()
        .ok_or_else(|| E::custom("public frame must be an object"))?;
    if object
        .keys()
        .any(|key| !allowed.iter().any(|allowed| key == allowed))
    {
        return Err(E::custom("public frame contains an unknown field"));
    }
    Ok(())
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
