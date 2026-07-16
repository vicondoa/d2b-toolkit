# Redaction boundary

Client, target, session, attachment, identity, provider, and service errors use
their canonical d2b `Debug` and `Display` implementations. The toolkit does not
wrap them in a wire-serializable generic redaction type.

Presentation helpers contain only UI colors, fixed CSS names, and caller-owned
Waybar text. They must not be extended with credentials, endpoints, target
identifiers, terminal bytes, command arguments, environment values, current
working directories, opaque handles, or generated service messages.

When a missing redaction primitive is required by more than one consumer, add
it to the canonical d2b owner first and consume it through the pinned source.
Do not create a serialized sibling DTO.
