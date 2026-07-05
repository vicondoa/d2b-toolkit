# Design notes

The toolkit is split into small crates so protocol DTOs, runtime-agnostic client framing, and Wayland presentation helpers can evolve independently.

The client layer separates wire/framing from concrete sockets. This keeps daemon clients testable with in-memory transports and avoids binding every downstream user to one async runtime.

Redaction is part of the type model. Sensitive shell-owner fields use wrappers whose diagnostics are redacted by default, and tests assert that representative secrets do not appear in debug output.

## Wayland helper boundaries

The Wayland helper crates separate safe presentation metadata from Unix ancillary-data transport. `d2b-wayland-core` contains color values and ordinary client metadata such as app id, title, display name, surface kind, output name, and scale. It does not parse Wayland proxy wire messages, own file descriptors, or expose platform transport details.

`d2b-wayland-proxy` is the only crate in this workspace that models SCM_RIGHTS-style FD transport. It exposes bounded message DTOs and traits so downstream proxy implementations can provide their own Unix socket integration without moving unsafe or platform-specific seams into shared client crates.
