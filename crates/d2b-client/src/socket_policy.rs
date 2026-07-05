use crate::ClientError;
use d2b_toolkit_core::SocketClass;

pub fn ensure_allowed_socket(class: SocketClass) -> Result<(), ClientError> {
    match class {
        SocketClass::PrivilegedBroker => {
            Err(d2b_toolkit_core::ToolkitError::PrivilegedBrokerRefused.into())
        }
        SocketClass::PublicDaemon | SocketClass::Other => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::ensure_allowed_socket;
    use d2b_toolkit_core::SocketClass;

    #[test]
    fn refuses_privileged_broker_without_echoing_paths() {
        let err = ensure_allowed_socket(SocketClass::PrivilegedBroker).unwrap_err();
        let debug = format!("{err:?}");
        assert!(debug.contains("PrivilegedBrokerRefused"));
        assert!(!debug.contains("/run/d2b"));
        assert!(!debug.contains("broker.sock"));
    }
}
