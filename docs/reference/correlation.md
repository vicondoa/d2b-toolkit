# Correlation and redaction

`d2b-toolkit` clients may correlate operations with an ephemeral trace id or a
non-reversible digest supplied by the caller. Raw shell names, terminal bytes,
argv, environment values, cwd values, and opaque handles must never be used as
log fields, metric labels, span attributes, or user-visible diagnostics.

Consumers should prefer bounded fields such as operation kind, result, and error
kind. When a cross-component investigation needs correlation, pass an ephemeral
trace id through the request context and keep the raw session handle confined to
the wire payload.
