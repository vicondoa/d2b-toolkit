# d2b-client-toolkit

GitHub/flake source distribution for d2b desktop clients. Version 2.0.0 is a
clean break from the former public-JSON toolkit: client, contract, and session
APIs are the canonical non-publishable crates from
[`vicondoa/d2b`](https://github.com/vicondoa/d2b).

The distribution is pinned to d2b revision
`7e94327951d30913a1a6e0e7a47d4a24b462deff` and source fingerprint
`0401d1f463d9dad49efd663d1493184e42954f624a029fbfd41c49f0323e5708`.
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
