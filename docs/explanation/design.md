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

The canonical client now distributes the content-frozen daemon, guest, and
user/desktop service clients. Its local connector still accepts an already-owned
route, endpoint policy, credentials, and transport. Live acquisition and
integrated routing are runtime behavior and are not guessed here.

## Presentation boundary

Color parsing and Waybar JSON are local presentation models. They do not carry
service requests, session records, credentials, target identifiers, terminal
bytes, or opaque handles.

Authenticated Wayland contracts come from the canonical user/desktop service
API. The former ancillary-FD helper remains removed so this distribution does
not collide with or bypass the canonical `d2b-wayland-proxy`.
