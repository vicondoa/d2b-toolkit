# Client protocol

`d2b-client` implements bounded, length-prefixed JSON framing over
`futures::io::{AsyncRead, AsyncWrite}`. It does not create sockets or select an
async runtime. Callers provide a transport connected to the d2b public daemon
socket.

Frames use a four-byte little-endian body length followed by one JSON object.
The public frame limit is 1 MiB. Collection, identifier, token, target, and
presentation-text fields also have decode-time bounds.

## Negotiation

The public protocol version remains 3. Workload clients advertise and require:

- `configured-launch-v1`
- `unsafe-local-provider-v1`

Unsafe-local shell clients additionally require `unsafe-local-shell-v1`.
Unknown feature tokens are retained so newer daemons can be inspected without
losing information.

Construct a workload client with `PublicSocketClient::with_hello_ok` or
`with_negotiated_capabilities`. Workload methods on an unnegotiated client, or
one missing either required workload feature, return `FeatureUnavailable`
before writing. `PublicSocketClient::new` remains valid for legacy VM shell
operations. Call `require_unsafe_local_shell` or
`requiring_unsafe_local_shell` when a canonical unsafe-local target will use
the shell API.

## Workload frames

Workload requests and responses use the daemon's flattened public shape:

```json
{"type":"workload","op":"status","args":{"target":"builder.dev.d2b"},"opId":1}
```

```json
{"type":"workloadResponse","op":"status","result":{"workload":{}},"opId":1}
```

`opId` is optional for compatibility. When present, the client requires an
exact envelope correlation match. It also checks response operation and
launcher result identity. Unknown fields, response types, and operation
mismatches fail with typed errors.

`workload_inventory`, `workload_list`, `workload_status`, and `launcher_exec`
cover public inventory, status, and configured launch. `launcher_exec` accepts
only a canonical target, item id, and caller-generated operation id. Reuse the
same operation id when retrying; the toolkit never creates one. Public launcher
metadata and launch requests contain no argv, environment, cwd, executable
path, or session value.

Launcher selection is explicit item, then configured default, then the sole
item. Multiple remaining items return bounded `AmbiguousLauncherItems`
candidates containing only ids and presentation names. Firefox has no special
meaning; it is an ordinary `exec` item.

## Shell frames

Shell requests retain
`{"type":"shell","op":...,"args":...,"opId":...}` and `shellResponse` shapes.
Existing list, attach, detach, kill, and attached-session helpers are
unchanged. Their `vm` wire field accepts either a validated legacy VM name or a
canonical workload target without converting the target back to a VM.

## Transport boundary

Toolkit clients never connect to the privileged broker or the per-user
unsafe-local helper. Both are private implementation transports behind `d2bd`;
their messages and types are absent from the public toolkit API. There is no
direct helper fallback.

## Shared fixture contract

`d2b-public-workload-v3-fixtures-v1` lives at
`crates/d2b-toolkit-core/tests/fixtures/public-workload-v3-v1/`. It is the
shared conformance input for Rust, Waybar, wlcontrol, wlterm, and JSON clients.
It covers legacy and first-class local VMs, unsafe-local generic exec and shell
items, every posture enum, feature-forward capability tokens, launch results,
and rejected secret-bearing fields.

## Packaging contract

Flake consumers depend on `d2b-toolkit.packages.${system}.default` and rewrite
Cargo path dependencies to `share/d2b-toolkit/crates/`. The source package
includes the workspace lockfile, fixtures, and docs for the exact selected
revision.
