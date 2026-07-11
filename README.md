# d2b-toolkit

Shared Rust/Nix toolkit crates for d2b desktop integrations. Version 0.2.0
provides the public protocol-v3 workload, launcher, posture, and persistent
shell contracts used by desktop clients.

## Crates

- `d2b-toolkit-core`: bounded public workload, capability, launcher, shell,
  hello, redaction, and socket-classification DTOs.
- `d2b-client`: feature-aware, runtime-agnostic public-socket workload and
  shell helpers over `futures::io::{AsyncRead, AsyncWrite}`.
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

The client crate does not open sockets directly. Runtime integrations pass a
concrete futures async-I/O transport connected only to the public daemon
socket. Toolkit clients never connect to the privileged broker or the private
unsafe-local helper.

Workload launch is reference-only: callers provide a canonical target, item id,
and stable operation id. Public DTOs contain no argv, environment, cwd,
executable path, uid, output, or helper message. Unknown capability tokens are
preserved for forward compatibility.

`unsafe-local` is explicitly **not isolated**. It runs as the authenticated host
user and must not be presented as equivalent to `local-vm` or a
provider-managed boundary. Workload methods require
`configured-launch-v1` and `unsafe-local-provider-v1`; unsafe-local shell
clients additionally opt into `unsafe-local-shell-v1`.

The reusable `d2b-public-workload-v3-fixtures-v1` conformance contract is under
`crates/d2b-toolkit-core/tests/fixtures/public-workload-v3-v1/`. See
[`docs/reference/client-protocol.md`](docs/reference/client-protocol.md) for
frame shapes, version-skew behavior, and launcher selection.

Wayland FD passing stays isolated in `d2b-wayland-proxy`; shared client crates
model only safe metadata and presentation data.
