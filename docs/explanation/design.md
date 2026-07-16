# Distribution design

`d2b-client-toolkit` is an integration and distribution repository, not a
second protocol owner.

The canonical d2b repository owns serialized identities, ComponentSession
records, generated service bindings, target resolution, retries, attachments,
named streams, Unix descriptor handling, and client errors. This repository
pins one immutable d2b revision and re-exports those crates with type identity
preserved. A source fingerprint gate verifies the complete upstream closure.

## Runtime boundary

The canonical client and session crates are Tokio-compatible. Desktop
applications using another UI executor keep that executor outside the client
boundary and submit client futures through an explicit Tokio handle. The
toolkit does not add a futures-I/O compatibility protocol or a second transport
implementation.

The current canonical foundation accepts an already-owned route, endpoint
policy, credentials, and transport. Endpoint discovery and route acquisition
belong to the control-service API and are not guessed here.

## Presentation boundary

Color parsing and Waybar JSON are local presentation models. They do not carry
service requests, session records, credentials, target identifiers, terminal
bytes, or opaque handles.

Authenticated Wayland control belongs to the user/desktop service API. The
former ancillary-FD helper was removed so this distribution does not collide
with or bypass the canonical `d2b-wayland-proxy`.
