use crate::ToolkitError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocketClass {
    PublicDaemon,
    PrivilegedBroker,
    Other,
}

pub fn ensure_client_socket(class: SocketClass) -> Result<(), ToolkitError> {
    match class {
        SocketClass::PrivilegedBroker => Err(ToolkitError::PrivilegedBrokerRefused),
        SocketClass::PublicDaemon | SocketClass::Other => Ok(()),
    }
}
