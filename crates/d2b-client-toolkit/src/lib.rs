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

pub const D2B_SOURCE_REVISION: &str = "4018d9c9652bd826c2e6a9abccdcdcafb832d944";
pub const D2B_SOURCE_FINGERPRINT: &str =
    "c2c99bdd77ba66948fce81161dcc3efde608eefefb96f28fa934c9f58d96d838";

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
}
