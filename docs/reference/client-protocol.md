# Client protocol skeleton

`d2b-client` provides bounded, length-prefixed JSON frame helpers over `futures::io::{AsyncRead, AsyncWrite}`. The crate does not own socket creation; runtime-specific callers adapt their own transport.

Initial negotiation uses `Hello` and `HelloResponse` from `d2b-toolkit-core`. The current skeleton validates the accepted protocol version and leaves feature-specific negotiation for later implementation.

The privileged broker socket is not a client transport. Socket classification helpers fail closed with a typed refusal error and do not echo filesystem paths.
