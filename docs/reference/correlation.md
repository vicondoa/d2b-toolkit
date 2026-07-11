# Correlation and redaction

`d2b-toolkit` clients may correlate operations with an ephemeral trace id or a
non-reversible digest supplied by the caller. Raw shell names, terminal bytes,
argv, environment values, cwd values, and opaque handles must never be used as
log fields, metric labels, span attributes, or user-visible diagnostics.

Configured launch also carries a caller-generated `operationId` for stable
retries. It is protocol data, not a trace or metric label: reuse it for the
same retry, but never print it through `Debug`, `Display`, errors, logs, or
metrics. Envelope `opId` remains an optional numeric request/response
correlator and is checked independently.

Consumers should prefer bounded fields such as operation kind, result, and error
kind. When a cross-component investigation needs correlation, pass an ephemeral
trace id through the request context and keep the raw session handle confined to
the wire payload.
