# d2b-client-toolkit agent guide

This repository distributes canonical d2b client sources and owns only its
facade, presentation helpers, packaging, and documentation.

## Invariants

- `d2b-client`, `d2b-contracts`, `d2b-session`, and `d2b-session-unix` come
  from the one exact d2b revision declared in `Cargo.toml`, `flake.lock`, and
  `docs/reference/source-pin.json`.
- Do not copy protocol DTOs, generated bindings, framing, handshakes, errors,
  resolvers, routes, or fixtures. Re-export canonical types directly.
- Canonical client work runs on Tokio. Cross-runtime consumers use an explicit
  runtime boundary; they do not copy or translate protocol code.
- Keep the color and Waybar crates presentation-only.
- Do not claim live endpoint discovery, route acquisition, persistent-shell,
  notification, desktop-action, or authenticated Wayland behavior before the
  owning service contracts are frozen.
- Never connect normal client code to a privileged broker socket.
- All crates remain `publish = false`; distribution is through flakes and
  GitHub source archives.
- Keep one `nixpkgs` input and make downstream toolkit inputs follow the
  consumer's `nixpkgs`.

## Validation

```bash
cargo fmt --all -- --check
cargo build --workspace --all-features --locked
cargo clippy --workspace --all-features --all-targets --locked -- -D warnings
cargo test --workspace --all-features --locked
python3 scripts/check-source-fingerprint.py
nix flake check
```
