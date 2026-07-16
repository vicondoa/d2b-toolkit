# d2b-client-toolkit

GitHub/flake source distribution for d2b desktop clients. Version 2.0.0 is a
clean break from the former public-JSON toolkit: client, contract, and session
APIs are the canonical non-publishable crates from
[`vicondoa/d2b`](https://github.com/vicondoa/d2b).

The distribution is pinned to d2b revision
`9183b45c6505cfd496e5d537bf6376f884fb16c7` and source fingerprint
`6f63e19042fb60bc2626e566321f5cf73574ed03caf6971de0c4c739f3ed5dd6`.
CI verifies every file in the upstream client distribution inventory.

## Crates

- `d2b-client-toolkit` re-exports `d2b-client`, `d2b-contracts`, and
  `d2b-session` without wrapping their types. Its `host-socket` feature also
  re-exports `d2b-session-unix`.
- `d2b-client-toolkit-colors` parses presentation-only UI color artifacts and
  emits the stable CSS color names.
- `d2b-client-toolkit-waybar` serializes generic Waybar presentation models.

All crates are `publish = false`. Releases are source archives and flake
outputs, not crates.io packages.

## Flake usage

Use one `nixpkgs` revision across the consumer and toolkit:

```nix
{
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

    d2b-client-toolkit = {
      url = "github:vicondoa/d2b-client-toolkit";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
}
```

`packages.${system}.d2b-client-toolkit` (also `default`) contains:

```text
share/d2b-client-toolkit/
├── distribution/  # facade and presentation crates, docs, and lockfile
└── d2b/            # exact canonical d2b source
```

This lets Nix builds rewrite dependencies to ordinary paths in one immutable
source artifact. See
[`docs/how-to/use-as-path-dependency.md`](docs/how-to/use-as-path-dependency.md).

## Runtime boundary

Canonical client operations are Tokio-compatible. `TokioClientAdapter` makes
the selected runtime handle explicit when a desktop application uses another
executor for its UI. It does not discover an endpoint, synthesize a route, or
translate an older protocol.

Live endpoint/route examples remain intentionally absent until the owning
control-service API is content-frozen. An authenticated Wayland control helper
is likewise not implemented until the user/desktop service API is frozen.

## Development

```bash
cargo fmt --all -- --check
cargo build --workspace --all-features --locked
cargo clippy --workspace --all-features --all-targets --locked -- -D warnings
cargo test --workspace --all-features --locked
python3 scripts/check-source-fingerprint.py
nix flake check
```

The fingerprint command uses `D2B_CANONICAL_SOURCE` when set; otherwise it
locates the exact Cargo Git checkout without network access.
