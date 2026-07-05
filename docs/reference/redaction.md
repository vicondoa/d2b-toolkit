# Redaction contract

The toolkit treats terminal bytes, command arguments, environment keys/values, current working directories, and opaque session handles as sensitive. They must not appear in `Debug`, logs, metrics, or error strings.

Use `Redacted<T>`, `SensitiveString`, `TerminalBytes`, and `OpaqueHandle` for DTO fields that cross the shell owner boundary. Serialization is for protocol use only; diagnostics must rely on the redacted `Debug`/`Display` implementations.

Shell names may be shown as presentation text, but they are not metrics labels. Metrics should use bounded labels such as `surface="shell"` rather than the concrete shell name.
