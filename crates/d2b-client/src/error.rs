use d2b_toolkit_core::ToolkitError;

/// Client errors carry only bounded metadata. Sensitive protocol payloads stay
/// out of Display/Debug/log output by construction.
#[derive(Debug, thiserror::Error)]
pub enum ClientError {
    #[error(transparent)]
    Core(#[from] ToolkitError),

    #[error("codec error while {context}")]
    Codec {
        context: &'static str,
        #[source]
        source: serde_json::Error,
    },

    #[error("hello negotiation failed: {reason}")]
    Hello { reason: &'static str },

    #[error("daemon returned typed error: {kind}")]
    Daemon { kind: String },

    #[error("unexpected daemon response while {context}")]
    UnexpectedResponse { context: &'static str },

    #[error("daemon response correlation mismatch")]
    CorrelationMismatch,
}

impl From<ClientError> for ToolkitError {
    fn from(value: ClientError) -> Self {
        match value {
            ClientError::Core(err) => err,
            ClientError::Codec { .. }
            | ClientError::Hello { .. }
            | ClientError::Daemon { .. }
            | ClientError::UnexpectedResponse { .. }
            | ClientError::CorrelationMismatch => ToolkitError::Protocol { kind: "client" },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ClientError;
    use std::error::Error;

    fn assert_error_bounds<E: Error + Send + Sync + 'static>() {}

    #[test]
    fn error_is_send_sync_static() {
        assert_error_bounds::<ClientError>();
    }

    #[test]
    fn debug_does_not_include_sensitive_payloads() {
        let err = ClientError::Hello { reason: "version" };
        let debug = format!("{err:?}");
        assert!(!debug.contains("opaque-session-handle"));
        assert!(!debug.contains("terminal bytes"));
        assert!(!debug.contains("SECRET_ENV"));
    }

    #[test]
    fn daemon_error_does_not_include_message_or_remediation() {
        let err = ClientError::Daemon {
            kind: "guest-control-shell-stale-session".into(),
        };
        let rendered = err.to_string();
        assert!(rendered.contains("guest-control-shell-stale-session"));
        assert!(!rendered.contains("opaque-session-handle"));
    }
}
