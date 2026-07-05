# Design notes

The toolkit is split into small crates so protocol DTOs, runtime-agnostic client framing, and Wayland presentation helpers can evolve independently.

The client layer separates wire/framing from concrete sockets. This keeps daemon clients testable with in-memory transports and avoids binding every downstream user to one async runtime.

Redaction is part of the type model. Sensitive shell-owner fields use wrappers whose diagnostics are redacted by default, and tests assert that representative secrets do not appear in debug output.
