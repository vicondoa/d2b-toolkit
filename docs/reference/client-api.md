# Client API

The `d2b-client-toolkit` crate is a zero-copy facade over the canonical d2b
client foundation.

| Path | Canonical owner | Contents |
| --- | --- | --- |
| crate root and `client` | `d2b-client` | typed targets, services, client calls, retry/cancellation, attachments, and named streams |
| `contracts` | `d2b-contracts` | v2 identities, ComponentSession records, provider/state records, and generated service bindings |
| `session` | `d2b-session` | authenticated session runtime, owned transports, packets, and streams |
| `unix_session` | `d2b-session-unix` | Linux socket and descriptor substrate; available only with `host-socket` |

The re-exports preserve Rust type identity. There are no toolkit request,
response, hello, frame, error-envelope, shell, workload, or compatibility
types.

## Features

- `default`: empty.
- `host-socket`: selects `d2b-client/host-socket` and exposes the canonical Unix
  session crate.

The direct `d2b-contracts` dependency selects only `v2-services`; legacy public
JSON features are not enabled.

## Tokio adapter

`TokioClientAdapter::new` accepts an explicit `tokio::runtime::Handle`.
`TokioClientAdapter::current` fails with `RuntimeUnavailable` outside an
entered Tokio runtime. `spawn` returns a typed task whose `join` method maps
runtime task failure to the closed `TaskFailed` error.

The adapter schedules canonical client futures only. It does not select or
open an endpoint, construct credentials, acquire a route, or convert between
wire formats.

## Deferred integrations

Live daemon endpoint and route examples are unavailable until the
control-service API is content-frozen. Persistent shell, notification, desktop
action, and authenticated Wayland examples additionally depend on the
user/desktop service API. This distribution intentionally provides no
placeholder implementation or fallback.

Canonical API details and generated contracts are in the bundled
`share/d2b-client-toolkit/d2b/docs/reference/` tree.
