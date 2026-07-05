use crate::ClientError;
use d2b_toolkit_core::{ShellOp, ShellOpResponse};
use futures::{Sink, Stream};

pub trait ShellOwnerSink: Sink<ShellOp, Error = ClientError> + Send + Unpin {}
impl<T> ShellOwnerSink for T where T: Sink<ShellOp, Error = ClientError> + Send + Unpin {}

pub trait ShellOwnerStream: Stream<Item = ShellOpResponse> + Send + Unpin {}
impl<T> ShellOwnerStream for T where T: Stream<Item = ShellOpResponse> + Send + Unpin {}

/// Type-safe shell-owner half-duplex boundary. Concrete transports can be
/// in-memory tests, framed public-daemon sessions, or runtime-specific sockets.
pub struct ShellOwnerBoundary<S, R> {
    sink: S,
    stream: R,
}

impl<S, R> ShellOwnerBoundary<S, R>
where
    S: ShellOwnerSink,
    R: ShellOwnerStream,
{
    pub fn new(sink: S, stream: R) -> Self {
        Self { sink, stream }
    }

    pub fn split(self) -> (S, R) {
        (self.sink, self.stream)
    }
}
