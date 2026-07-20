#![forbid(unsafe_code)]

//! Distribution facade for the canonical d2b v2 client crates.
//!
//! Protocol, service, session, and transport definitions are re-exported from
//! the exact pinned d2b source. This crate defines no wire representation.

mod tokio_adapter;

pub use d2b_client as client;
pub use d2b_client::*;
pub use d2b_contracts as contracts;
pub use d2b_session as session;
#[cfg(feature = "host-socket")]
pub use d2b_session_unix as unix_session;
pub use tokio_adapter::{TokioAdapterError, TokioClientAdapter, TokioClientTask};

pub const D2B_SOURCE_REVISION: &str = "9dc902243cdd7aba7ef269988b96f0aae6e037da";
pub const D2B_SOURCE_FINGERPRINT: &str =
    "5a20cef3a64281df819eeb76bdfe385999755479b467b559653011582fb9c043";

#[cfg(test)]
mod tests {
    use super::*;
    use std::any::TypeId;

    #[test]
    fn reexports_canonical_types_without_wrappers() {
        assert_eq!(
            TypeId::of::<ClientError>(),
            TypeId::of::<d2b_client::ClientError>()
        );
        assert_eq!(
            TypeId::of::<session::TransportPacket>(),
            TypeId::of::<d2b_session::TransportPacket>()
        );
        assert_eq!(
            TypeId::of::<contracts::v2_component_session::SessionErrorCode>(),
            TypeId::of::<d2b_contracts::v2_component_session::SessionErrorCode>()
        );
    }

    #[test]
    fn source_pin_is_full_and_exact() {
        assert_eq!(D2B_SOURCE_REVISION.len(), 40);
        assert_eq!(D2B_SOURCE_FINGERPRINT.len(), 64);
    }

    #[test]
    fn exposes_content_frozen_service_clients() {
        assert_eq!(
            TypeId::of::<DaemonClient>(),
            TypeId::of::<d2b_client::DaemonClient>()
        );
        assert_eq!(
            TypeId::of::<GuestClient>(),
            TypeId::of::<d2b_client::GuestClient>()
        );
        assert!(matches!(
            ServiceKind::Daemon,
            d2b_client::ServiceKind::Daemon
        ));
        assert!(matches!(ServiceKind::User, d2b_client::ServiceKind::User));
        assert!(matches!(ServiceKind::Shell, d2b_client::ServiceKind::Shell));
        assert!(matches!(
            ServiceKind::Notify,
            d2b_client::ServiceKind::Notify
        ));
        assert!(matches!(
            ServiceKind::Wayland,
            d2b_client::ServiceKind::Wayland
        ));
    }
}
