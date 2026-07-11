# Changelog

All notable changes to this project will be documented in this file.

## [Unreleased]

## [0.2.0] - 2026-07-11

### Added

- Added bounded public protocol-v3 workload identity, provider, posture,
  availability, capability, launcher-item, inventory, status, and configured
  launch DTOs.
- Added runtime-agnostic workload inventory, status, and launcher execution
  methods with negotiated feature checks and strict response correlation.
- Added canonical-target shell support and explicit
  `unsafe-local-shell-v1` negotiation without changing legacy VM shell
  compatibility.
- Added deterministic unknown capability preservation, launcher selection
  helpers, typed validation and ambiguity errors, and closed metrics labels.
- Added the shared `d2b-public-workload-v3-fixtures-v1` conformance contract
  for Rust and JSON desktop clients.
- Added safe Wayland metadata, color artifact fallback parsing, Waybar JSON
  serialization, and proxy-only ancillary FD transport traits.

### Changed

- Updated every toolkit crate and Nix package/check version to 0.2.0.
- Aligned hello, shell, workload, and error frames with d2b's flattened public
  daemon protocol while retaining optional operation correlation ids.
- Validated shell names and workload-facing identifiers, targets, tokens,
  collections, and presentation fields during decode.

### Security

- Kept launcher argv, environment, cwd, executable paths, output, user data,
  opaque handles, and helper protocol messages out of public workload DTOs.
- Redacted caller operation ids and opaque session values from diagnostics and
  restricted metrics helpers to closed non-identifying classes.
- Enforced public-socket-only client boundaries; toolkit clients do not connect
  to the privileged broker or private unsafe-local helper.
- Documented `unsafe-local` as an explicit no-isolation posture with no local
  execution or shell fallback.
