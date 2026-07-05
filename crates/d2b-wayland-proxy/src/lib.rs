#![forbid(unsafe_code)]

//! Wayland proxy transport seams.
//!
//! Public JSON-frame clients remain byte-stream oriented. Wayland proxy clients
//! need a Unix-specific side channel for ancillary file descriptors such as
//! `wl_shm` memfds, so this crate owns that extension point.

#[derive(Debug, thiserror::Error)]
pub enum WaylandProxyTransportError {
    #[error("ancillary fd transport is unsupported on this platform")]
    UnsupportedPlatform,
    #[error("ancillary fd transport failed while {context}")]
    Transport { context: &'static str },
}

#[cfg(unix)]
pub mod unix {
    use super::WaylandProxyTransportError;
    use std::os::fd::OwnedFd;

    pub struct AncillaryMessage {
        pub bytes: Vec<u8>,
        pub fds: Vec<OwnedFd>,
    }

    pub trait AncillaryFdTransport {
        fn send_with_fds(
            &mut self,
            message: AncillaryMessage,
        ) -> Result<(), WaylandProxyTransportError>;
        fn recv_with_fds(&mut self) -> Result<AncillaryMessage, WaylandProxyTransportError>;
    }
}

#[cfg(not(unix))]
pub mod unix {
    use super::WaylandProxyTransportError;

    pub struct AncillaryMessage {
        pub bytes: Vec<u8>,
    }

    pub trait AncillaryFdTransport {
        fn send_with_fds(
            &mut self,
            _message: AncillaryMessage,
        ) -> Result<(), WaylandProxyTransportError> {
            Err(WaylandProxyTransportError::UnsupportedPlatform)
        }

        fn recv_with_fds(&mut self) -> Result<AncillaryMessage, WaylandProxyTransportError> {
            Err(WaylandProxyTransportError::UnsupportedPlatform)
        }
    }
}
