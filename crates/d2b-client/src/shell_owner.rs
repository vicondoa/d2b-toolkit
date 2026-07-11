use crate::{read_json_frame, write_json_frame, ClientError, FrameBounds};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use d2b_toolkit_core::{
    CorrelationId, ErrorEnvelope, HelloOk, KnownFeatureFlag, NegotiatedCapabilities, OpaqueHandle,
    PublicRequest, PublicResponse, Redacted, ShellAttachArgs, ShellAttachResult,
    ShellCloseAttachArgs, ShellDetachArgs, ShellDetachResult, ShellKillArgs, ShellKillResult,
    ShellListArgs, ShellListResult, ShellName, ShellOp, ShellOpResponse, TerminalClose,
    TerminalCloseResult, TerminalControlResult, TerminalReadOutput, TerminalReadOutputChunk,
    TerminalResize, TerminalSize, TerminalStream, TerminalWait, TerminalWaitResult,
    TerminalWriteStdin, TerminalWriteStdinResult,
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
    pub(crate) transport: T,
    pub(crate) bounds: FrameBounds,
    pub(crate) next_op_id: u64,
    pub(crate) negotiated_capabilities: Option<NegotiatedCapabilities>,
    require_unsafe_local_shell: bool,
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
            negotiated_capabilities: None,
            require_unsafe_local_shell: false,
        }
    }

    pub fn with_negotiated_capabilities(
        transport: T,
        capabilities: NegotiatedCapabilities,
    ) -> Self {
        Self::with_bounds_and_negotiated_capabilities(
            transport,
            FrameBounds::default(),
            capabilities,
        )
    }

    pub fn with_bounds_and_negotiated_capabilities(
        transport: T,
        bounds: FrameBounds,
        capabilities: NegotiatedCapabilities,
    ) -> Self {
        Self {
            transport,
            bounds,
            next_op_id: 1,
            negotiated_capabilities: Some(capabilities),
            require_unsafe_local_shell: false,
        }
    }

    pub fn with_hello_ok(transport: T, hello: &HelloOk) -> Self {
        Self::with_negotiated_capabilities(transport, hello.negotiated_capabilities())
    }

    pub fn negotiated_capabilities(&self) -> Option<&NegotiatedCapabilities> {
        self.negotiated_capabilities.as_ref()
    }

    pub fn require_unsafe_local_shell(&mut self) -> Result<(), ClientError> {
        self.require_feature(KnownFeatureFlag::UnsafeLocalShellV1)?;
        self.require_unsafe_local_shell = true;
        Ok(())
    }

    pub fn requiring_unsafe_local_shell(mut self) -> Result<Self, ClientError> {
        self.require_unsafe_local_shell()?;
        Ok(self)
    }

    pub fn into_inner(self) -> T {
        self.transport
    }

    pub async fn shell_list(
        &mut self,
        target: impl Into<String>,
    ) -> Result<ShellListResult, ClientError> {
        let target = checked_shell_target(target.into())?;
        let response = self
            .round_trip(ShellOp::List(ShellListArgs { vm: target }))
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
        target: impl Into<String>,
        name: Option<ShellName>,
        force: bool,
        size: TerminalSize,
    ) -> Result<AttachedShell<T>, ClientError> {
        let target = checked_shell_target(target.into())?;
        let response = self
            .round_trip(ShellOp::Attach(ShellAttachArgs {
                vm: target,
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
        target: impl Into<String>,
        name: Option<ShellName>,
    ) -> Result<ShellDetachResult, ClientError> {
        let target = checked_shell_target(target.into())?;
        let response = self
            .round_trip(ShellOp::Detach(ShellDetachArgs { vm: target, name }))
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
        target: impl Into<String>,
        name: ShellName,
    ) -> Result<ShellKillResult, ClientError> {
        let target = checked_shell_target(target.into())?;
        let response = self
            .round_trip(ShellOp::Kill(ShellKillArgs { vm: target, name }))
            .await?;
        match response {
            ShellOpResponse::Kill(result) => Ok(result),
            _ => Err(ClientError::UnexpectedResponse {
                context: "killing shell",
            }),
        }
    }

    async fn round_trip(&mut self, op: ShellOp) -> Result<ShellOpResponse, ClientError> {
        self.preflight_shell_features()?;
        let op_id = self.reserve_op_id();
        let request = PublicRequest::shell(Some(op_id), op);
        write_json_frame(&mut self.transport, &request, self.bounds).await?;
        let response: PublicResponse = read_json_frame(&mut self.transport, self.bounds).await?;
        shell_response_for_op(response, op_id)
    }

    pub(crate) fn reserve_op_id(&mut self) -> u64 {
        let op_id = self.next_op_id.max(1);
        self.next_op_id = op_id.wrapping_add(1).max(1);
        op_id
    }

    fn preflight_shell_features(&self) -> Result<(), ClientError> {
        if self.require_unsafe_local_shell {
            self.require_feature(KnownFeatureFlag::UnsafeLocalShellV1)?;
        }
        Ok(())
    }

    pub(crate) fn require_feature(&self, feature: KnownFeatureFlag) -> Result<(), ClientError> {
        match self.negotiated_capabilities.as_ref() {
            Some(capabilities) => capabilities.require(feature).map_err(ClientError::from),
            None => Err(d2b_toolkit_core::ToolkitError::FeatureUnavailable { feature }.into()),
        }
    }
}

fn checked_shell_target(target: String) -> Result<String, ClientError> {
    d2b_toolkit_core::workload::validate_shell_target(&target)?;
    Ok(target)
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
        PublicResponse::Shell { op_id, response }
            if op_id.is_none() || op_id == Some(expected_op_id) =>
        {
            Ok(response)
        }
        PublicResponse::Shell { .. } => Err(ClientError::CorrelationMismatch),
        PublicResponse::Error { op_id, error } if op_id == Some(expected_op_id) => {
            daemon_error(error)
        }
        PublicResponse::Error { op_id: None, error } => daemon_error(error),
        PublicResponse::Error { .. } => Err(ClientError::CorrelationMismatch),
        PublicResponse::Workload { .. } => Err(ClientError::UnexpectedResponse {
            context: "decoding shell response type",
        }),
    }
}

fn daemon_error(error: ErrorEnvelope) -> Result<ShellOpResponse, ClientError> {
    Err(ClientError::daemon_kind(error.kind))
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

    #[test]
    fn shared_operation_ids_wrap_without_returning_zero() {
        let mut client = PublicSocketClient::new(FakePublicSocket::default());
        client.next_op_id = u64::MAX - 1;

        let ids = [
            client.reserve_op_id(),
            client.reserve_op_id(),
            client.reserve_op_id(),
            client.reserve_op_id(),
        ];

        assert_eq!(ids, [u64::MAX - 1, u64::MAX, 1, 2]);
        assert_eq!(
            ids.iter()
                .copied()
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            4
        );
        assert!(!ids.contains(&0));
    }

    #[test]
    fn shell_and_workload_operations_share_wrapping_id_namespace() {
        block_on(async {
            let responses = vec![
                default_list_response(u64::MAX - 1),
                PublicResponse::Workload {
                    op_id: Some(u64::MAX),
                    response: d2b_toolkit_core::WorkloadOpResponse::List(
                        d2b_toolkit_core::WorkloadListResult { workloads: vec![] },
                    ),
                },
                default_list_response(1),
                PublicResponse::Workload {
                    op_id: Some(2),
                    response: d2b_toolkit_core::WorkloadOpResponse::List(
                        d2b_toolkit_core::WorkloadListResult { workloads: vec![] },
                    ),
                },
            ];
            let capabilities = NegotiatedCapabilities::from_features([
                KnownFeatureFlag::ConfiguredLaunchV1.wire_value(),
                KnownFeatureFlag::UnsafeLocalProviderV1.wire_value(),
            ]);
            let mut client = PublicSocketClient::with_negotiated_capabilities(
                FakePublicSocket::with_responses(responses),
                capabilities,
            );
            client.next_op_id = u64::MAX - 1;

            client.shell_list("corp-vm").await.unwrap();
            client.workload_inventory().await.unwrap();
            client.shell_list("corp-vm").await.unwrap();
            client.workload_inventory().await.unwrap();

            let op_ids = client
                .into_inner()
                .written_json_frames()
                .into_iter()
                .map(|frame| frame["opId"].as_u64().unwrap())
                .collect::<Vec<_>>();
            assert_eq!(op_ids, [u64::MAX - 1, u64::MAX, 1, 2]);
        });
    }

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
                default_name: ShellName::new("default").unwrap(),
                sessions: vec![d2b_toolkit_core::ShellListEntry {
                    name: ShellName::new("customer-project").unwrap(),
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
            assert_eq!(frames[0]["type"], "shell");
            assert_eq!(frames[0]["op"], "list");
            assert_eq!(frames[0]["args"]["vm"], "corp-vm");
            assert_eq!(frames[0]["opId"], 1);
        });
    }

    #[test]
    fn shell_list_accepts_stateless_response_without_op_id() {
        block_on(async {
            let response = PublicResponse::Shell {
                op_id: None,
                response: ShellOpResponse::List(ShellListResult {
                    default_name: ShellName::new("default").unwrap(),
                    sessions: vec![],
                }),
            };
            let mut client =
                PublicSocketClient::new(FakePublicSocket::with_responses(vec![response]));
            let result = client.shell_list("corp-vm").await.unwrap();
            assert!(result.sessions.is_empty());
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
                        resolved_name: ShellName::new("default").unwrap(),
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
                        resolved_name: ShellName::new("default").unwrap(),
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
            assert_eq!(frames[0]["op"], "attach");
            assert!(frames[0]["args"].get("session").is_none());
            assert_eq!(frames[1]["op"], "writeStdin");
            assert_eq!(frames[1]["args"]["session"], "opaque-session-handle");
            assert_eq!(frames[1]["args"]["chunkBase64"], "aGVsbG8=");
            assert_eq!(frames[2]["op"], "readOutput");
            assert_eq!(frames[2]["args"]["offset"], 0);
            assert_eq!(frames[3]["op"], "resize");
            assert_eq!(frames[4]["op"], "closeAttach");
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
                        resolved_name: ShellName::new("default").unwrap(),
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
                        resolved_name: ShellName::new("default").unwrap(),
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
            ShellName::new("customer-project")
                .unwrap()
                .metrics_label_value(),
            "shell"
        );
        let error = ErrorEnvelope {
            kind: "guest-control-shell-timeout".into(),
            exit_code: 69,
            message: "message".into(),
            remediation: "retry".into(),
        };
        assert_eq!(error.metrics_label_value(), "daemon-error");
    }
}
