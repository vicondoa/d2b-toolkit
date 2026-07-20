# Source distribution contract

The canonical source pin is
`9dc902243cdd7aba7ef269988b96f0aae6e037da` from
`https://github.com/vicondoa/d2b`. The client distribution fingerprint is
`5a20cef3a64281df819eeb76bdfe385999755479b467b559653011582fb9c043`.

[`toolkit-source-contract.json`](./toolkit-source-contract.json) is the exact
source/ownership inventory frozen at d2b revision
`9dc902243cdd7aba7ef269988b96f0aae6e037da`.
[`source-pin.json`](./source-pin.json) binds this repository to the canonical
revision, distribution fingerprint, and inventory digest.

The inventory carries updated source-contract and foundation-crate references as
two byte-exact, non-wire supplements under `canonical-source-artifacts/`. All
Rust, protobuf, generated binding, manifest, test, schema, vector, and other
reference bytes come directly from the exact canonical revision. The drift gate
permits no other overlay path.

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
supplements, and the exact pinned `d2b/` tree. They do not include d2b's lockfile
as this repository's lockfile; each distribution owns its dependency resolution.
