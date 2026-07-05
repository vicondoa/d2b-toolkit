#![forbid(unsafe_code)]

//! Wayland proxy transport seams.
//!
//! Public JSON-frame clients remain byte-stream oriented. Wayland proxy clients
//! need a Unix-specific side channel for ancillary file descriptors such as
//! `wl_shm` memfds, so this crate owns that extension point. It deliberately
//! exposes transport traits and owned-message DTOs only; raw proxy wire parsing
//! stays out of shared client crates.

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum WaylandProxyTransportError {
    #[error("ancillary fd transport is unsupported on this platform")]
    UnsupportedPlatform,
    #[error("ancillary fd transport failed while {context}")]
    Transport { context: &'static str },
    #[error("ancillary fd message exceeded configured bounds")]
    BoundsExceeded,
}

pub const DEFAULT_MAX_MESSAGE_BYTES: usize = 64 * 1024;
pub const DEFAULT_MAX_FDS: usize = 28;

#[cfg(unix)]
pub mod unix {
    use super::{WaylandProxyTransportError, DEFAULT_MAX_FDS, DEFAULT_MAX_MESSAGE_BYTES};
    use std::os::fd::OwnedFd;

    #[derive(Debug)]
    pub struct AncillaryMessage {
        pub bytes: Vec<u8>,
        pub fds: Vec<OwnedFd>,
    }

    impl AncillaryMessage {
        pub fn new(bytes: Vec<u8>, fds: Vec<OwnedFd>) -> Result<Self, WaylandProxyTransportError> {
            Self::with_bounds(bytes, fds, AncillaryBounds::default())
        }

        pub fn with_bounds(
            bytes: Vec<u8>,
            fds: Vec<OwnedFd>,
            bounds: AncillaryBounds,
        ) -> Result<Self, WaylandProxyTransportError> {
            if bytes.len() > bounds.max_message_bytes || fds.len() > bounds.max_fds {
                return Err(WaylandProxyTransportError::BoundsExceeded);
            }
            Ok(Self { bytes, fds })
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct AncillaryBounds {
        pub max_message_bytes: usize,
        pub max_fds: usize,
    }

    impl Default for AncillaryBounds {
        fn default() -> Self {
            Self {
                max_message_bytes: DEFAULT_MAX_MESSAGE_BYTES,
                max_fds: DEFAULT_MAX_FDS,
            }
        }
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

    #[derive(Debug)]
    pub struct AncillaryMessage {
        pub bytes: Vec<u8>,
    }

    impl AncillaryMessage {
        pub fn new(bytes: Vec<u8>) -> Result<Self, WaylandProxyTransportError> {
            Ok(Self { bytes })
        }
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

#[cfg(all(test, unix))]
mod tests {
    use super::unix::{AncillaryBounds, AncillaryFdTransport, AncillaryMessage};
    use super::WaylandProxyTransportError;
    use std::{collections::VecDeque, fs::File, os::fd::OwnedFd};

    #[derive(Default)]
    struct MemoryFdTransport {
        queue: VecDeque<AncillaryMessage>,
    }

    impl AncillaryFdTransport for MemoryFdTransport {
        fn send_with_fds(
            &mut self,
            message: AncillaryMessage,
        ) -> Result<(), WaylandProxyTransportError> {
            self.queue.push_back(message);
            Ok(())
        }

        fn recv_with_fds(&mut self) -> Result<AncillaryMessage, WaylandProxyTransportError> {
            self.queue
                .pop_front()
                .ok_or(WaylandProxyTransportError::Transport {
                    context: "reading memory fd transport",
                })
        }
    }

    #[test]
    fn fd_transport_trait_owns_ancillary_fds_in_proxy_crate() {
        let fd: OwnedFd = File::open("/dev/null").unwrap().into();
        let message = AncillaryMessage::new(b"wayland-bytes".to_vec(), vec![fd]).unwrap();
        let mut transport = MemoryFdTransport::default();

        transport.send_with_fds(message).unwrap();
        let received = transport.recv_with_fds().unwrap();

        assert_eq!(received.bytes, b"wayland-bytes");
        assert_eq!(received.fds.len(), 1);
    }

    #[test]
    fn ancillary_message_enforces_bounds_before_transport() {
        let too_large = AncillaryMessage::with_bounds(
            vec![0; 4],
            Vec::new(),
            AncillaryBounds {
                max_message_bytes: 3,
                max_fds: 0,
            },
        );
        assert_eq!(
            too_large.unwrap_err(),
            WaylandProxyTransportError::BoundsExceeded
        );
    }
}
