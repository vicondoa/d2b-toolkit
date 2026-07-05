# d2b-toolkit

Shared Rust/Nix toolkit crates for d2b desktop integrations. The workspace is intentionally small and library-first so downstream projects can use it as a path dependency while the protocol and UI contracts settle.

## Crates

- `d2b-toolkit-core`: shared DTOs, redaction wrappers, daemon hello shape, socket classification, and public shell messages.
- `d2b-client`: runtime-agnostic framed public-socket client helpers over `futures::io::{AsyncRead, AsyncWrite}`.
- `d2b-wayland-core`: safe Wayland client metadata DTOs and UI color types.
- `d2b-wayland-colors`: d2b UI color artifact parsing, fallback reporting, and CSS variable helpers.
- `d2b-wayland-waybar`: Waybar custom-module serialization helpers.
- `d2b-wayland-proxy`: Unix-only ancillary FD transport trait seams for proxy integrations.

## Development

```bash
cargo fmt --all -- --check
cargo test --workspace
nix flake check
```

## Flake usage

Downstream desktop clients should share the consumer host's `nixpkgs` input and
use the packaged source output when building from flakes:

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    d2b-toolkit = {
      url = "github:vicondoa/d2b-toolkit";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
}
```

`packages.${system}.default` is a source package containing `Cargo.toml`,
`Cargo.lock`, `crates/`, `README.md`, and `docs/`. Sibling clients such as
`d2b-wlterm`, `d2b-wlcontrol`, and WeezTerm use it to rewrite local Cargo path
dependencies in Nix builds without vendoring another toolkit copy.

The client crate does not open sockets directly. Runtime integrations must pass concrete transports implementing the futures async I/O traits and must use the public daemon socket, never the privileged broker socket. Wayland FD passing stays isolated in `d2b-wayland-proxy`; shared client crates only model safe metadata and presentation data.
