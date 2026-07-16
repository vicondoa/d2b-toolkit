# Source distribution contract

The canonical source pin is
`7e94327951d30913a1a6e0e7a47d4a24b462deff` from
`https://github.com/vicondoa/d2b`. The client distribution fingerprint is
`0401d1f463d9dad49efd663d1493184e42954f624a029fbfd41c49f0323e5708`.

[`toolkit-source-contract.json`](./toolkit-source-contract.json) is the exact
source/ownership inventory frozen at d2b inventory revision
`d5a913922eb019ed83a16e4e64f562303b31ecf7`.
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
