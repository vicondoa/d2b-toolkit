# d2b-toolkit

Shared Rust/Nix toolkit crates for d2b desktop integrations. The workspace is intentionally small and library-first so downstream projects can use it as a path dependency while the protocol and UI contracts settle.

## Crates

- `d2b-toolkit-core`: shared DTOs, redaction wrappers, hello shape, socket classification, shell owner messages.
- `d2b-client`: runtime-agnostic framed client helpers over `futures::io::{AsyncRead, AsyncWrite}`.
- `d2b-wayland-core`: common Wayland/UI color types.
- `d2b-wayland-colors`: d2b UI color artifact parsing and CSS variable helpers.
- `d2b-wayland-waybar`: Waybar-facing module scaffolding.

## Development

```bash
cargo fmt --all -- --check
cargo test --workspace
nix flake check
```

The client crate does not open sockets directly. Runtime integrations must pass concrete transports implementing the futures async I/O traits and must use the public daemon socket, never the privileged broker socket.
