# d2b public workload v3 fixtures v1

Contract name: `d2b-public-workload-v3-fixtures-v1`.

These files mirror d2b's public protocol-v3 workload serde shapes. They are
shared inputs for toolkit crates and downstream Rust or JSON desktop clients.

- `local-vm-list-response.json`: legacy-backed local VM.
- `first-class-local-vm-list-response.json`: local VM without
  `legacyVmName`.
- `unsafe-local-list-response.json`: no-isolation host tools with ordinary
  Firefox `exec` and terminal `shell` items, helper-unavailable state, and an
  unknown capability token.
- `workload-frames.json`: flattened list, status, launch, and response frames
  with optional envelope correlation.
- `all-enums.json`: every provider, posture, availability, state, launcher
  kind, disposition, and known capability token.
- `malformed-secret-injections.json`: fields and values that public decoders
  must reject without reflecting sensitive content.

Valid public launcher records never contain argv, environment, cwd, executable
paths, command output, opaque sessions, or private helper messages.
