use crate::shell::{ShellOp, ShellOpResponse};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum PublicRequest {
    Shell(ShellOp),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum PublicResponse {
    Shell(ShellOpResponse),
    Error { kind: String },
}
