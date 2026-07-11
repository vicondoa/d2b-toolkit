# Redaction contract

Terminal bytes, command arguments, environment keys and values, current
working directories, executable paths, command output, opaque handles, and
caller operation ids must not appear in `Debug`, `Display`, logs, metrics, or
error strings.

Use `Redacted<T>`, `SensitiveString`, `TerminalBytes`, and `OpaqueHandle` for
shell-owner payloads. `OperationId` is wire-serializable but its `Debug` and
`Display` are always redacted. Correlation errors never print expected or
received values.

Workload targets, workload and launcher display names, and icon identifiers are
presentation data and may be rendered in UI. Free-form presentation strings
are bounded by the public frame, omitted from metadata `Debug`, and must not
become metrics labels. Ambiguous launcher errors expose only sanitized,
field-bounded item ids and names; they never include provider-private execution
data.

Metrics helpers return closed provider, posture, availability, state, item
kind, disposition, or operation classes. They never return a target, workload
id, item id, name, user id, shell name, or daemon-supplied free-form text.

Public launcher summaries reject fields such as `argv`, `env`, `cwd`, `path`,
`output`, and `session`. Daemon error messages and remediations are omitted
from diagnostics; clients propagate only the bounded error kind.
