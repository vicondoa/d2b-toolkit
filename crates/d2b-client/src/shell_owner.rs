use crate::{read_json_frame, write_json_frame, ClientError, FrameBounds};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use d2b_toolkit_core::{
    CorrelationId, ErrorEnvelope, OpaqueHandle, PublicRequest, PublicResponse, Redacted,
    ShellAttachArgs, ShellAttachResult, ShellCloseAttachArgs, ShellDetachArgs, ShellDetachResult,
    ShellKillArgs, ShellKillResult, ShellListArgs, ShellListResult, ShellName, ShellOp,
    ShellOpResponse, TerminalClose, TerminalCloseResult, TerminalControlResult, TerminalReadOutput,
    TerminalReadOutputChunk, TerminalResize, TerminalSize, TerminalStream, TerminalWait,
    TerminalWaitResult, TerminalWriteStdin, TerminalWriteStdinResult,
};
use futures::io::{AsyncRead, AsyncWrite};
use futures::{Sink, Stream};

pub trait AttachedShellCommandSink:
    Sink<AttachedShellCommand, Error = ClientError> + Send + Unpin
{
}
impl<T> AttachedShellCommandSink for T where
    T: Sink<AttachedShellCommand, Error = ClientError> + Send + Unpin
{
}

pub trait AttachedShellEventStream:
    Stream<Item = Result<AttachedShellEvent, ClientError>> + Send + Unpin
{
}
impl<T> AttachedShellEventStream for T where
    T: Stream<Item = Result<AttachedShellEvent, ClientError>> + Send + Unpin
{
}

/// Type-safe attached-shell boundary. Commands deliberately omit the daemon
/// session handle and stateless list/attach/detach/kill variants.
pub struct AttachedShellBoundary<S, R> {
    sink: S,
    stream: R,
}

impl<S, R> AttachedShellBoundary<S, R>
where
    S: AttachedShellCommandSink,
    R: AttachedShellEventStream,
{
    pub fn new(sink: S, stream: R) -> Self {
        Self { sink, stream }
    }

    pub fn split(self) -> (S, R) {
        (self.sink, self.stream)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttachedShellCommand {
    Write {
        bytes: Redacted<Vec<u8>>,
        eof: bool,
    },
    Read {
        stream: TerminalStream,
        max_len: u64,
        wait: bool,
        timeout_ms: u64,
    },
    Resize {
        rows: u32,
        cols: u32,
    },
    Wait {
        timeout_ms: u64,
    },
    CloseStdin,
    CloseAttach,
}

impl AttachedShellCommand {
    pub const fn metrics_label_value(&self) -> &'static str {
        match self {
            Self::Write { .. } => "write",
            Self::Read { .. } => "read",
            Self::Resize { .. } => "resize",
            Self::Wait { .. } => "wait",
            Self::CloseStdin => "close-stdin",
            Self::CloseAttach => "close-attach",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttachedShellEvent {
    WriteAccepted(TerminalWriteStdinResult),
    Output(TerminalReadOutputChunk),
    Resized(TerminalControlResult),
    Wait(TerminalWaitResult),
    StdinClosed(TerminalCloseResult),
    Detached(ShellDetachResult),
}

pub struct PublicSocketClient<T> {
    transport: T,
    bounds: FrameBounds,
    next_op_id: u64,
}

impl<T> PublicSocketClient<T>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    pub fn new(transport: T) -> Self {
        Self::with_bounds(transport, FrameBounds::default())
    }

    pub fn with_bounds(transport: T, bounds: FrameBounds) -> Self {
        Self {
            transport,
            bounds,
            next_op_id: 1,
        }
    }

    pub fn into_inner(self) -> T {
        self.transport
    }

    pub async fn shell_list(
        &mut self,
        vm: impl Into<String>,
    ) -> Result<ShellListResult, ClientError> {
        let response = self
            .round_trip(ShellOp::List(ShellListArgs { vm: vm.into() }))
            .await?;
        match response {
            ShellOpResponse::List(result) => Ok(result),
            _ => Err(ClientError::UnexpectedResponse {
                context: "listing shells",
            }),
        }
    }

    pub async fn attach_shell(
        mut self,
        vm: impl Into<String>,
        name: Option<ShellName>,
        force: bool,
        size: TerminalSize,
    ) -> Result<AttachedShell<T>, ClientError> {
        let response = self
            .round_trip(ShellOp::Attach(ShellAttachArgs {
                vm: vm.into(),
                name,
                force,
                initial_terminal_size: size,
            }))
            .await?;
        match response {
            ShellOpResponse::Attach(result) => Ok(AttachedShell::new(self, result)),
            _ => Err(ClientError::UnexpectedResponse {
                context: "attaching shell",
            }),
        }
    }

    pub async fn shell_detach(
        &mut self,
        vm: impl Into<String>,
        name: Option<ShellName>,
    ) -> Result<ShellDetachResult, ClientError> {
        let response = self
            .round_trip(ShellOp::Detach(ShellDetachArgs {
                vm: vm.into(),
                name,
            }))
            .await?;
        match response {
            ShellOpResponse::Detach(result) => Ok(result),
            _ => Err(ClientError::UnexpectedResponse {
                context: "detaching shell",
            }),
        }
    }

    pub async fn shell_kill(
        &mut self,
        vm: impl Into<String>,
        name: ShellName,
    ) -> Result<ShellKillResult, ClientError> {
        let response = self
            .round_trip(ShellOp::Kill(ShellKillArgs {
                vm: vm.into(),
                name,
            }))
            .await?;
        match response {
            ShellOpResponse::Kill(result) => Ok(result),
            _ => Err(ClientError::UnexpectedResponse {
                context: "killing shell",
            }),
        }
    }

    async fn round_trip(&mut self, op: ShellOp) -> Result<ShellOpResponse, ClientError> {
        let op_id = self.reserve_op_id();
        let request = PublicRequest::shell(Some(op_id), op);
        write_json_frame(&mut self.transport, &request, self.bounds).await?;
        let response: PublicResponse = read_json_frame(&mut self.transport, self.bounds).await?;
        shell_response_for_op(response, op_id)
    }

    fn reserve_op_id(&mut self) -> u64 {
        let op_id = self.next_op_id;
        self.next_op_id = self.next_op_id.saturating_add(1);
        op_id
    }
}

pub struct AttachedShell<T> {
    client: PublicSocketClient<T>,
    session: OpaqueHandle,
    resolved_name: ShellName,
    stdin_offset: u64,
    stdout_offset: u64,
    stderr_offset: u64,
}

impl<T> AttachedShell<T>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    fn new(client: PublicSocketClient<T>, result: ShellAttachResult) -> Self {
        Self {
            client,
            session: result.session,
            resolved_name: result.resolved_name,
            stdin_offset: 0,
            stdout_offset: 0,
            stderr_offset: 0,
        }
    }

    pub fn resolved_name(&self) -> &ShellName {
        &self.resolved_name
    }

    pub fn correlation_id(&self) -> CorrelationId {
        CorrelationId::from_opaque_handle(&self.session)
    }

    pub fn into_inner(self) -> PublicSocketClient<T> {
        self.client
    }

    pub async fn write_bytes(
        &mut self,
        bytes: Redacted<Vec<u8>>,
        eof: bool,
    ) -> Result<TerminalWriteStdinResult, ClientError> {
        let raw = bytes.into_inner_for_wire();
        let encoded = STANDARD.encode(&raw);
        let response = self
            .client
            .round_trip(ShellOp::WriteStdin(TerminalWriteStdin {
                session: self.session.clone(),
                offset: self.stdin_offset,
                chunk_base64: Redacted::new(encoded),
                eof,
            }))
            .await?;
        match response {
            ShellOpResponse::WriteStdin(result) => {
                self.stdin_offset = result.next_offset;
                Ok(result)
            }
            _ => Err(ClientError::UnexpectedResponse {
                context: "writing shell stdin",
            }),
        }
    }

    pub async fn read_output(
        &mut self,
        stream: TerminalStream,
        max_len: u64,
        wait: bool,
        timeout_ms: u64,
    ) -> Result<TerminalReadOutputChunk, ClientError> {
        let offset = match stream {
            TerminalStream::Stdout => self.stdout_offset,
            TerminalStream::Stderr => self.stderr_offset,
        };
        let response = self
            .client
            .round_trip(ShellOp::ReadOutput(TerminalReadOutput {
                session: self.session.clone(),
                stream,
                offset,
                max_len,
                wait,
                timeout_ms,
            }))
            .await?;
        match response {
            ShellOpResponse::ReadOutput(chunk) => {
                match stream {
                    TerminalStream::Stdout => self.stdout_offset = chunk.next_offset,
                    TerminalStream::Stderr => self.stderr_offset = chunk.next_offset,
                }
                Ok(chunk)
            }
            _ => Err(ClientError::UnexpectedResponse {
                context: "reading shell output",
            }),
        }
    }

    pub async fn resize(
        &mut self,
        rows: u32,
        cols: u32,
    ) -> Result<TerminalControlResult, ClientError> {
        let response = self
            .client
            .round_trip(ShellOp::Resize(TerminalResize {
                session: self.session.clone(),
                rows,
                cols,
                op_id: 0,
            }))
            .await?;
        match response {
            ShellOpResponse::Resize(result) => Ok(result),
            _ => Err(ClientError::UnexpectedResponse {
                context: "resizing shell",
            }),
        }
    }

    pub async fn wait(&mut self, timeout_ms: u64) -> Result<TerminalWaitResult, ClientError> {
        let response = self
            .client
            .round_trip(ShellOp::Wait(TerminalWait {
                session: self.session.clone(),
                timeout_ms,
            }))
            .await?;
        match response {
            ShellOpResponse::Wait(result) => Ok(result),
            _ => Err(ClientError::UnexpectedResponse {
                context: "waiting for shell",
            }),
        }
    }

    pub async fn close_stdin(&mut self) -> Result<TerminalCloseResult, ClientError> {
        let response = self
            .client
            .round_trip(ShellOp::CloseStdin(TerminalClose {
                session: self.session.clone(),
            }))
            .await?;
        match response {
            ShellOpResponse::CloseStdin(result) => Ok(result),
            _ => Err(ClientError::UnexpectedResponse {
                context: "closing shell stdin",
            }),
        }
    }

    pub async fn close_attach(
        mut self,
    ) -> Result<(PublicSocketClient<T>, ShellDetachResult), ClientError> {
        let response = self
            .client
            .round_trip(ShellOp::CloseAttach(ShellCloseAttachArgs {
                session: self.session.clone(),
            }))
            .await?;
        match response {
            ShellOpResponse::CloseAttach(result) => Ok((self.client, result)),
            _ => Err(ClientError::UnexpectedResponse {
                context: "closing shell attach",
            }),
        }
    }
}

fn shell_response_for_op(
    response: PublicResponse,
    expected_op_id: u64,
) -> Result<ShellOpResponse, ClientError> {
    match response {
        PublicResponse::Shell { op_id, response } if op_id == Some(expected_op_id) => Ok(response),
        PublicResponse::Shell { .. } => Err(ClientError::CorrelationMismatch),
        PublicResponse::Error { op_id, error } if op_id == Some(expected_op_id) => {
            daemon_error(error)
        }
        PublicResponse::Error { .. } => Err(ClientError::CorrelationMismatch),
    }
}

fn daemon_error(error: ErrorEnvelope) -> Result<ShellOpResponse, ClientError> {
    Err(ClientError::Daemon { kind: error.kind })
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor::block_on;
    use futures::io::{AsyncRead, AsyncWrite};
    use serde::Serialize;
    use serde_json::Value;
    use std::pin::Pin;
    use std::task::{Context, Poll};

    #[derive(Default)]
    struct FakePublicSocket {
        reads: Vec<u8>,
        read_pos: usize,
        writes: Vec<u8>,
    }

    impl FakePublicSocket {
        fn with_responses(responses: Vec<PublicResponse>) -> Self {
            let mut reads = Vec::new();
            for response in responses {
                reads.extend(frame(&response));
            }
            Self {
                reads,
                read_pos: 0,
                writes: Vec::new(),
            }
        }

        fn written_json_frames(&self) -> Vec<Value> {
            let mut frames = Vec::new();
            let mut pos = 0;
            while pos < self.writes.len() {
                let len =
                    u32::from_le_bytes(self.writes[pos..pos + 4].try_into().unwrap()) as usize;
                pos += 4;
                frames.push(serde_json::from_slice(&self.writes[pos..pos + len]).unwrap());
                pos += len;
            }
            frames
        }
    }

    impl AsyncRead for FakePublicSocket {
        fn poll_read(
            mut self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
            buf: &mut [u8],
        ) -> Poll<std::io::Result<usize>> {
            if self.read_pos >= self.reads.len() {
                return Poll::Ready(Ok(0));
            }
            let available = self.reads.len() - self.read_pos;
            let n = available.min(buf.len());
            let start = self.read_pos;
            let end = start + n;
            buf[..n].copy_from_slice(&self.reads[start..end]);
            self.read_pos = end;
            Poll::Ready(Ok(n))
        }
    }

    impl AsyncWrite for FakePublicSocket {
        fn poll_write(
            mut self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
            buf: &[u8],
        ) -> Poll<std::io::Result<usize>> {
            self.writes.extend_from_slice(buf);
            Poll::Ready(Ok(buf.len()))
        }

        fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
            Poll::Ready(Ok(()))
        }

        fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
            Poll::Ready(Ok(()))
        }
    }

    fn frame<T: Serialize>(value: &T) -> Vec<u8> {
        let payload = serde_json::to_vec(value).unwrap();
        let mut frame = Vec::with_capacity(payload.len() + 4);
        frame.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        frame.extend_from_slice(&payload);
        frame
    }

    fn shell_error(op_id: u64, kind: &str) -> PublicResponse {
        PublicResponse::Error {
            op_id: Some(op_id),
            error: ErrorEnvelope {
                kind: kind.into(),
                exit_code: 69,
                message: "redacted daemon message".into(),
                remediation: "retry".into(),
            },
        }
    }

    fn default_list_response(op_id: u64) -> PublicResponse {
        PublicResponse::Shell {
            op_id: Some(op_id),
            response: ShellOpResponse::List(ShellListResult {
                default_name: ShellName::new("default"),
                sessions: vec![d2b_toolkit_core::ShellListEntry {
                    name: ShellName::new("customer-project"),
                    state: d2b_toolkit_core::ShellSessionState::Detached,
                    attached: false,
                    is_default: true,
                }],
            }),
        }
    }

    #[test]
    fn shell_list_uses_real_public_socket_shape() {
        block_on(async {
            let transport = FakePublicSocket::with_responses(vec![default_list_response(1)]);
            let mut client = PublicSocketClient::new(transport);
            let result = client.shell_list("corp-vm").await.unwrap();
            assert_eq!(result.sessions.len(), 1);

            let transport = client.into_inner();
            let frames = transport.written_json_frames();
            assert_eq!(frames[0]["kind"], "shell");
            assert_eq!(frames[0]["payload"]["op"], "list");
            assert_eq!(frames[0]["payload"]["args"]["vm"], "corp-vm");
            assert_eq!(frames[0]["opId"], 1);
        });
    }

    #[test]
    fn attached_shell_api_injects_session_for_stateful_ops() {
        block_on(async {
            let responses = vec![
                PublicResponse::Shell {
                    op_id: Some(1),
                    response: ShellOpResponse::Attach(ShellAttachResult {
                        session: OpaqueHandle::new("opaque-session-handle"),
                        resolved_name: ShellName::new("default"),
                        state: d2b_toolkit_core::ShellSessionState::Attached,
                        force_evicted: false,
                    }),
                },
                PublicResponse::Shell {
                    op_id: Some(2),
                    response: ShellOpResponse::WriteStdin(TerminalWriteStdinResult {
                        accepted_len: 5,
                        next_offset: 5,
                        backpressured: false,
                        stdin_closed: false,
                    }),
                },
                PublicResponse::Shell {
                    op_id: Some(3),
                    response: ShellOpResponse::ReadOutput(TerminalReadOutputChunk {
                        data_base64: Redacted::new("b2s=".into()),
                        next_offset: 2,
                        eof: false,
                        dropped_bytes: 0,
                        truncated: false,
                        timed_out: false,
                    }),
                },
                PublicResponse::Shell {
                    op_id: Some(4),
                    response: ShellOpResponse::Resize(TerminalControlResult { delivered: true }),
                },
                PublicResponse::Shell {
                    op_id: Some(5),
                    response: ShellOpResponse::CloseAttach(ShellDetachResult {
                        resolved_name: ShellName::new("default"),
                        detached: true,
                        cause: None,
                    }),
                },
            ];
            let transport = FakePublicSocket::with_responses(responses);
            let client = PublicSocketClient::new(transport);
            let mut attached = client
                .attach_shell("corp-vm", None, false, TerminalSize { rows: 24, cols: 80 })
                .await
                .unwrap();
            assert!(!format!("{:?}", attached.correlation_id()).contains("opaque-session-handle"));
            attached
                .write_bytes(Redacted::new(b"hello".to_vec()), false)
                .await
                .unwrap();
            attached
                .read_output(TerminalStream::Stdout, 1024, true, 25)
                .await
                .unwrap();
            attached.resize(30, 100).await.unwrap();
            let (client, detach) = attached.close_attach().await.unwrap();
            assert!(detach.detached);

            let frames = client.into_inner().written_json_frames();
            assert_eq!(frames[0]["payload"]["op"], "attach");
            assert!(frames[0]["payload"]["args"].get("session").is_none());
            assert_eq!(frames[1]["payload"]["op"], "writeStdin");
            assert_eq!(
                frames[1]["payload"]["args"]["session"],
                "opaque-session-handle"
            );
            assert_eq!(frames[1]["payload"]["args"]["chunkBase64"], "aGVsbG8=");
            assert_eq!(frames[2]["payload"]["op"], "readOutput");
            assert_eq!(frames[2]["payload"]["args"]["offset"], 0);
            assert_eq!(frames[3]["payload"]["op"], "resize");
            assert_eq!(frames[4]["payload"]["op"], "closeAttach");
        });
    }

    #[test]
    fn typed_shell_errors_propagate_by_kind_only() {
        block_on(async {
            let transport = FakePublicSocket::with_responses(vec![shell_error(
                1,
                "guest-control-shell-timeout",
            )]);
            let mut client = PublicSocketClient::new(transport);
            let err = client.shell_list("corp-vm").await.unwrap_err();
            assert!(
                matches!(err, ClientError::Daemon { ref kind } if kind == "guest-control-shell-timeout")
            );
            let rendered = format!("{err:?}");
            assert!(!rendered.contains("opaque-session-handle"));
        });
    }

    #[test]
    fn stale_attached_session_is_typed_error() {
        block_on(async {
            let responses = vec![
                PublicResponse::Shell {
                    op_id: Some(1),
                    response: ShellOpResponse::Attach(ShellAttachResult {
                        session: OpaqueHandle::new("opaque-session-handle"),
                        resolved_name: ShellName::new("default"),
                        state: d2b_toolkit_core::ShellSessionState::Attached,
                        force_evicted: false,
                    }),
                },
                shell_error(2, "guest-control-shell-stale-session"),
            ];
            let transport = FakePublicSocket::with_responses(responses);
            let client = PublicSocketClient::new(transport);
            let mut attached = client
                .attach_shell("corp-vm", None, false, TerminalSize { rows: 24, cols: 80 })
                .await
                .unwrap();
            let err = attached
                .write_bytes(Redacted::new(b"hello".to_vec()), false)
                .await
                .unwrap_err();
            assert!(
                matches!(err, ClientError::Daemon { ref kind } if kind == "guest-control-shell-stale-session")
            );
        });
    }

    #[test]
    fn timed_out_read_chunk_is_not_a_runtime_timer() {
        block_on(async {
            let responses = vec![
                PublicResponse::Shell {
                    op_id: Some(1),
                    response: ShellOpResponse::Attach(ShellAttachResult {
                        session: OpaqueHandle::new("opaque-session-handle"),
                        resolved_name: ShellName::new("default"),
                        state: d2b_toolkit_core::ShellSessionState::Attached,
                        force_evicted: false,
                    }),
                },
                PublicResponse::Shell {
                    op_id: Some(2),
                    response: ShellOpResponse::ReadOutput(TerminalReadOutputChunk {
                        data_base64: Redacted::new(String::new()),
                        next_offset: 0,
                        eof: false,
                        dropped_bytes: 0,
                        truncated: false,
                        timed_out: true,
                    }),
                },
            ];
            let transport = FakePublicSocket::with_responses(responses);
            let client = PublicSocketClient::new(transport);
            let mut attached = client
                .attach_shell("corp-vm", None, false, TerminalSize { rows: 24, cols: 80 })
                .await
                .unwrap();
            let chunk = attached
                .read_output(TerminalStream::Stdout, 1024, true, 1)
                .await
                .unwrap();
            assert!(chunk.timed_out);
        });
    }

    #[test]
    fn metric_labels_are_bounded() {
        let command = AttachedShellCommand::Resize {
            rows: 40,
            cols: 120,
        };
        assert_eq!(command.metrics_label_value(), "resize");
        assert_eq!(
            ShellName::new("customer-project").metrics_label_value(),
            "shell"
        );
        let error = ErrorEnvelope {
            kind: "guest-control-shell-timeout".into(),
            exit_code: 69,
            message: "message".into(),
            remediation: "retry".into(),
        };
        assert_eq!(error.metrics_label_value(), "guest-control-shell-timeout");
    }
}
