# Source distribution contract

The canonical source pin is
`4018d9c9652bd826c2e6a9abccdcdcafb832d944` from
`https://github.com/vicondoa/d2b`. The client distribution fingerprint is
`c2c99bdd77ba66948fce81161dcc3efde608eefefb96f28fa934c9f58d96d838`.

[`toolkit-source-contract.json`](./toolkit-source-contract.json) is the exact
source/ownership inventory frozen at d2b inventory revision
`c645a769f50b8283c1eddeb12f2a9bf0a1f397bd`.
[`source-pin.json`](./source-pin.json) binds this repository to the canonical
revision, distribution fingerprint, and inventory digest.

The W9 inventory added its own source-contract reference and updated the
foundation-crate reference after the W4 code freeze. Those two byte-exact,
non-wire supplements live under `canonical-source-artifacts/`; all Rust,
protobuf, generated binding, manifest, test, schema, vector, and other
reference bytes come directly from the exact W4 source revision. The drift
gate permits no other overlay path.

For each selected source group, the drift gate verifies every file SHA-256 and
then hashes the sorted paths and bytes with the inventory's domain-separated,
length-prefixed encoding. It also verifies the sorted union as the client
distribution fingerprint.

The gate checks the same revision in:

- Cargo workspace dependency declarations;
- `Cargo.lock`;
- the non-flake `d2b-src` node in `flake.lock`;
- the facade constants; and
- the source-pin artifact.

Run:

```bash
python3 scripts/check-source-fingerprint.py
```

Set `D2B_CANONICAL_SOURCE` or pass `--source` to check an explicit source tree.
Without either, the script resolves the already-fetched Cargo Git checkout in
offline mode.

Release archives contain `distribution/`, including the two audited reference
supplements, and the exact W4 `d2b/` tree. They do not include d2b's lockfile as
this repository's lockfile; each distribution owns its dependency resolution.
