# Client protocol

`d2b-client` provides bounded, length-prefixed JSON frame helpers over `futures::io::{AsyncRead, AsyncWrite}`. The crate does not own socket creation; runtime-specific callers adapt their own public daemon socket transport.

Frames use the daemon public socket convention: a 4-byte little-endian body length followed by a JSON object. The default public-daemon frame bound is 1 MiB, and reads reject an over-bound declared length before allocating the payload buffer.

Initial negotiation sends a `{"type":"hello", ...}` frame with the d2b public `clientVersion` and `supportedFeatures` fields, then accepts `helloOk` or propagates `helloRejected` as a typed client error.

Shell routing uses the daemon shape directly: requests are `{"type":"shell","op":...,"args":...,"opId":...}` and responses are `shellResponse` or `error` with the same `opId`. `d2b-client` exposes list, attach, detach, and kill helpers plus an attached-shell handle that injects the opaque session into write, read, resize, wait, close-stdin, and close-attach operations. Consumers do not send raw `ShellOp` values for attached sessions.

The toolkit uses byte read/write only for the public socket path. It does not use `recvmsg`, ancillary file descriptors, runtime timers, or task spawning in core client code.

The privileged broker socket is not a client transport. Socket classification helpers fail closed with a typed refusal error and do not echo filesystem paths.

## Packaging contract

Flake consumers should depend on `d2b-toolkit.packages.${system}.default` and
rewrite Cargo path dependencies to the packaged source tree under
`share/d2b-toolkit/crates/`. The package includes the workspace lockfile and
docs so downstream `nix flake check` runs can build against the exact toolkit
revision selected by their flake lock.
