# d2b-toolkit agent guide

This repository contains shared toolkit crates for d2b desktop integrations.

## Invariants

- Keep `d2b-client` async-runtime-agnostic. It may depend on `futures::io::{AsyncRead, AsyncWrite}` but must not depend on Tokio, async-std, or smol in the core public-socket path.
- Wayland/proxy helpers that need Unix ancillary data should live behind Unix-specific transport traits/extensions rather than weakening the public JSON-frame path.
- Never connect to or model normal client access through the privileged broker socket. Client-facing code targets the public d2b daemon socket only.
- Do not put terminal bytes, argv, environment values, cwd values, or opaque handles into `Debug`, logs, metrics, or error strings.
- Shell names are user-controlled presentation data and must not become metrics labels.
- Keep Nix flake checks useful for downstream path-dependency development.

## Validation

Run the smallest relevant commands first:

```bash
cargo fmt --all -- --check
cargo test --workspace
nix flake check
```
