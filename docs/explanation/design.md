# Design notes

The toolkit is split into small crates so public protocol DTOs,
runtime-agnostic framing, and Wayland presentation helpers can evolve
independently.

The client layer separates wire framing from concrete sockets. In-memory and
runtime-specific transports implement `futures::io::{AsyncRead, AsyncWrite}`;
Tokio, async-std, and smol are not dependencies of the public client path.

## Provider-neutral workloads

`d2b-toolkit-core::workload` mirrors d2b's public protocol-v3 workload
contract. Stable realm-scoped identities and canonical targets are separate
from provider kind, lifecycle state, execution posture, availability, and
launcher presentation metadata. Capability sets preserve bounded unknown
tokens in deterministic order for forward compatibility.

Launcher summaries describe item-owned id, name, icon, kind, graphical state,
and capabilities only. Runtime argv, environment, cwd, executable paths, uid,
and proxy details stay provider-private and are resolved by the daemon.

`unsafe-local` is an explicit provider and isolation posture. It means **no
isolation**: execution occurs as the authenticated host user under that user's
manager. Clients must show that posture rather than presenting it as a VM or
provider-managed boundary.

Feature negotiation allows protocol-v3 peers to reject version skew before
dispatch. Workload operations require both configured-launch and unsafe-local
provider features, while unsafe-local shell support is negotiated separately.
There is no local execution or shell fallback.

## Trust boundaries

Every normal client operation crosses only the public d2b daemon socket. The
daemon owns helper and broker interaction. Toolkit crates neither connect to
nor model the private helper protocol or privileged broker protocol.

Redaction is part of the type model. Opaque session and operation identifiers
are serializable for wire use but hidden from diagnostics. Presentation fields
are bounded, and metric labels come only from closed enums.

## Wayland helper boundaries

`d2b-wayland-core` contains color values and ordinary client metadata such as
app id, title, display name, surface kind, output name, and scale. It does not
parse proxy wire messages, own file descriptors, or expose platform transport
details.

`d2b-wayland-proxy` alone models SCM_RIGHTS-style FD transport. It exposes
bounded message DTOs and traits so downstream proxy implementations can supply
Unix socket integration without moving platform-specific seams into shared
client crates.
