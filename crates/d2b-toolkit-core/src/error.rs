use std::io;

/// Core toolkit errors deliberately avoid carrying terminal bytes, argv, env,
/// cwd, opaque handles, or concrete socket paths.
#[derive(Debug, thiserror::Error)]
pub enum ToolkitError {
    #[error("protocol error: {kind}")]
    Protocol { kind: &'static str },

    #[error("frame too large ({len} bytes, max {max} bytes)")]
    FrameTooLarge { len: usize, max: usize },

    #[error("refused privileged broker socket")]
    PrivilegedBrokerRefused,

    #[error("I/O error while {context}")]
    Io {
        context: &'static str,
        #[source]
        source: io::Error,
    },
}

#[cfg(test)]
mod tests {
    use super::ToolkitError;
    use std::error::Error;

    fn assert_error_bounds<E: Error + Send + Sync + 'static>() {}

    #[test]
    fn error_is_send_sync_static() {
        assert_error_bounds::<ToolkitError>();
    }

    #[test]
    fn debug_does_not_include_sensitive_payloads() {
        let err = ToolkitError::Protocol { kind: "bad-frame" };
        let debug = format!("{err:?}");
        assert!(!debug.contains("super-secret-argv"));
        assert!(!debug.contains("opaque-session-handle"));
    }
}
