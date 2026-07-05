# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

### Added

- Documented the flake input follow policy and packaged-source contract used by
  downstream d2b desktop clients.
- Matured the Wayland toolkit crates with safe client metadata DTOs, color artifact fallback parsing, Waybar JSON serialization helpers, and proxy-only ancillary FD transport traits.
- Initialized the Rust/Nix workspace skeleton with core, client, Wayland color, and Waybar crates.
- Added redaction scaffolding for terminal bytes, argv/env/cwd, and opaque handles.
- Added bounded frame I/O and hello negotiation stubs for a runtime-agnostic d2b client.
- Added daemon-shaped public socket hello, shell request/response routing, typed error propagation, and attached-shell client helpers for downstream terminal clients.
