# Use as a path dependency

During local development, point downstream workspaces at this checkout:

```toml
[dependencies]
d2b-client = { path = "../d2b-toolkit/crates/d2b-client" }
d2b-toolkit-core = { path = "../d2b-toolkit/crates/d2b-toolkit-core" }
```

Run `cargo test --workspace` in this repository before updating downstream pins.
