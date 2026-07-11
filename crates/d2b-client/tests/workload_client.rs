use d2b_client::{ClientError, PublicSocketClient};
use d2b_toolkit_core::{
    ErrorEnvelope, KnownFeatureFlag, LauncherExecDisposition, LauncherExecResult,
    NegotiatedCapabilities, OperationId, ProtocolToken, PublicResponse, ShellListResult, ShellName,
    ShellOpResponse, ToolkitError, WorkloadListResult, WorkloadOpResponse, WorkloadPublicSummary,
    WorkloadStatusResult, WorkloadTarget,
};
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
        let mut position = 0;
        while position < self.writes.len() {
            let length = u32::from_le_bytes(self.writes[position..position + 4].try_into().unwrap())
                as usize;
            position += 4;
            frames.push(serde_json::from_slice(&self.writes[position..position + length]).unwrap());
            position += length;
        }
        frames
    }
}

impl AsyncRead for FakePublicSocket {
    fn poll_read(
        mut self: Pin<&mut Self>,
        _context: &mut Context<'_>,
        buffer: &mut [u8],
    ) -> Poll<std::io::Result<usize>> {
        if self.read_pos >= self.reads.len() {
            return Poll::Ready(Ok(0));
        }
        let count = buffer.len().min(self.reads.len() - self.read_pos);
        let start = self.read_pos;
        let end = start + count;
        buffer[..count].copy_from_slice(&self.reads[start..end]);
        self.read_pos = end;
        Poll::Ready(Ok(count))
    }
}

impl AsyncWrite for FakePublicSocket {
    fn poll_write(
        mut self: Pin<&mut Self>,
        _context: &mut Context<'_>,
        buffer: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        self.writes.extend_from_slice(buffer);
        Poll::Ready(Ok(buffer.len()))
    }

    fn poll_flush(self: Pin<&mut Self>, _context: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, _context: &mut Context<'_>) -> Poll<std::io::Result<()>> {
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

fn workload_capabilities() -> NegotiatedCapabilities {
    NegotiatedCapabilities::from_features([
        KnownFeatureFlag::ConfiguredLaunchV1.wire_value(),
        KnownFeatureFlag::UnsafeLocalProviderV1.wire_value(),
    ])
}

fn unsafe_shell_capabilities() -> NegotiatedCapabilities {
    NegotiatedCapabilities::from_features([KnownFeatureFlag::UnsafeLocalShellV1.wire_value()])
}

fn unsafe_local_workload() -> WorkloadPublicSummary {
    let value: Value = serde_json::from_str(include_str!(
        "../../d2b-toolkit-core/tests/fixtures/public-workload-v3-v1/unsafe-local-list-response.json"
    ))
    .unwrap();
    let response: PublicResponse = serde_json::from_value(value).unwrap();
    match response {
        PublicResponse::Workload {
            response: WorkloadOpResponse::List(result),
            ..
        } => result.workloads.into_iter().next().unwrap(),
        other => panic!("unexpected fixture: {other:?}"),
    }
}

fn list_response(op_id: Option<u64>) -> PublicResponse {
    PublicResponse::Workload {
        op_id,
        response: WorkloadOpResponse::List(WorkloadListResult {
            workloads: vec![unsafe_local_workload()],
        }),
    }
}

fn launcher_response(op_id: Option<u64>, operation_id: &str) -> PublicResponse {
    PublicResponse::Workload {
        op_id,
        response: WorkloadOpResponse::LauncherExec(LauncherExecResult {
            target: WorkloadTarget::parse("tools.host.d2b").unwrap(),
            item_id: ProtocolToken::parse("firefox").unwrap(),
            operation_id: OperationId::parse(operation_id).unwrap(),
            disposition: LauncherExecDisposition::Committed,
        }),
    }
}

fn daemon_error(op_id: Option<u64>, kind: &str) -> PublicResponse {
    PublicResponse::Error {
        op_id,
        error: ErrorEnvelope {
            kind: kind.into(),
            exit_code: 69,
            message: "OUTPUT_SECRET_CANARY".into(),
            remediation: "/home/alice/CWD_SECRET_CANARY".into(),
        },
    }
}

#[test]
fn workload_methods_require_both_features_before_writing() {
    block_on(async {
        let mut unnegotiated = PublicSocketClient::new(FakePublicSocket::default());
        let error = unnegotiated.workload_inventory().await.unwrap_err();
        assert!(matches!(
            error,
            ClientError::Core(ToolkitError::FeatureUnavailable {
                feature: KnownFeatureFlag::ConfiguredLaunchV1
            })
        ));
        assert!(unnegotiated.into_inner().writes.is_empty());

        let configured_only = NegotiatedCapabilities::from_features([
            KnownFeatureFlag::ConfiguredLaunchV1.wire_value(),
        ]);
        let mut skewed = PublicSocketClient::with_negotiated_capabilities(
            FakePublicSocket::default(),
            configured_only,
        );
        let error = skewed.workload_inventory().await.unwrap_err();
        assert!(matches!(
            error,
            ClientError::Core(ToolkitError::FeatureUnavailable {
                feature: KnownFeatureFlag::UnsafeLocalProviderV1
            })
        ));
        assert!(skewed.into_inner().writes.is_empty());
    });
}

#[test]
fn workload_list_writes_exact_flattened_frame() {
    block_on(async {
        let transport = FakePublicSocket::with_responses(vec![list_response(Some(1))]);
        let mut client =
            PublicSocketClient::with_negotiated_capabilities(transport, workload_capabilities());
        let result = client.workload_list(Some("host".into())).await.unwrap();
        assert_eq!(result.workloads.len(), 1);
        let frames = client.into_inner().written_json_frames();
        assert_eq!(
            frames,
            vec![serde_json::json!({
                "type": "workload",
                "op": "list",
                "args": {"realm": "host"},
                "opId": 1
            })]
        );
    });
}

#[test]
fn launcher_exec_uses_caller_operation_id_and_never_accepts_argv() {
    block_on(async {
        let transport =
            FakePublicSocket::with_responses(vec![launcher_response(Some(1), "launch-42")]);
        let mut client =
            PublicSocketClient::with_negotiated_capabilities(transport, workload_capabilities());
        let result = client
            .launcher_exec(
                WorkloadTarget::parse("tools.host.d2b").unwrap(),
                ProtocolToken::parse("firefox").unwrap(),
                OperationId::parse("launch-42").unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(result.disposition, LauncherExecDisposition::Committed);
        let frames = client.into_inner().written_json_frames();
        assert_eq!(frames[0]["args"]["operationId"], "launch-42");
        assert_eq!(frames[0]["args"]["itemId"], "firefox");
        assert!(frames[0]["args"].get("argv").is_none());
        assert!(frames[0]["args"].get("env").is_none());
        assert!(frames[0]["args"].get("cwd").is_none());
    });
}

#[test]
fn status_and_launcher_payloads_are_correlated() {
    block_on(async {
        let workload = unsafe_local_workload();
        let response = PublicResponse::Workload {
            op_id: Some(1),
            response: WorkloadOpResponse::Status(Box::new(WorkloadStatusResult { workload })),
        };
        let transport = FakePublicSocket::with_responses(vec![response]);
        let mut client =
            PublicSocketClient::with_negotiated_capabilities(transport, workload_capabilities());
        let status = client
            .workload_status(WorkloadTarget::parse("tools.host.d2b").unwrap())
            .await
            .unwrap();
        assert_eq!(
            status.workload.identity.canonical_target.as_str(),
            "tools.host.d2b"
        );

        let transport =
            FakePublicSocket::with_responses(vec![launcher_response(Some(1), "launch-other")]);
        let mut client =
            PublicSocketClient::with_negotiated_capabilities(transport, workload_capabilities());
        let error = client
            .launcher_exec(
                WorkloadTarget::parse("tools.host.d2b").unwrap(),
                ProtocolToken::parse("firefox").unwrap(),
                OperationId::parse("launch-42").unwrap(),
            )
            .await
            .unwrap_err();
        assert!(matches!(error, ClientError::CorrelationMismatch));
        assert!(!format!("{error:?}").contains("launch-42"));
        assert!(!format!("{error:?}").contains("launch-other"));
    });
}

#[test]
fn envelope_correlation_accepts_absent_ids_and_rejects_wrong_ids() {
    block_on(async {
        let transport = FakePublicSocket::with_responses(vec![list_response(None)]);
        let mut client =
            PublicSocketClient::with_negotiated_capabilities(transport, workload_capabilities());
        client.workload_inventory().await.unwrap();

        let transport = FakePublicSocket::with_responses(vec![list_response(Some(99))]);
        let mut client =
            PublicSocketClient::with_negotiated_capabilities(transport, workload_capabilities());
        let error = client.workload_inventory().await.unwrap_err();
        assert!(matches!(error, ClientError::CorrelationMismatch));
    });
}

#[test]
fn wrong_response_type_and_operation_are_typed() {
    block_on(async {
        let shell_response = PublicResponse::Shell {
            op_id: Some(1),
            response: ShellOpResponse::List(ShellListResult {
                default_name: ShellName::new("default").unwrap(),
                sessions: vec![],
            }),
        };
        let transport = FakePublicSocket::with_responses(vec![shell_response]);
        let mut client =
            PublicSocketClient::with_negotiated_capabilities(transport, workload_capabilities());
        let error = client.workload_inventory().await.unwrap_err();
        assert!(matches!(error, ClientError::UnexpectedResponse { .. }));

        let transport = FakePublicSocket {
            reads: frame(&serde_json::json!({
                "type": "audioResponse",
                "result": {}
            })),
            ..FakePublicSocket::default()
        };
        let mut client =
            PublicSocketClient::with_negotiated_capabilities(transport, workload_capabilities());
        let error = client.workload_inventory().await.unwrap_err();
        assert!(matches!(error, ClientError::UnexpectedResponse { .. }));

        let transport = FakePublicSocket {
            reads: frame(&serde_json::json!({
                "type": "workloadResponse",
                "op": "list",
                "result": {
                    "workloads": [],
                    "argv": ["ARG_SECRET_CANARY"]
                },
                "opId": 1
            })),
            ..FakePublicSocket::default()
        };
        let mut client =
            PublicSocketClient::with_negotiated_capabilities(transport, workload_capabilities());
        let error = client.workload_inventory().await.unwrap_err();
        assert!(matches!(error, ClientError::Codec { .. }));
        assert!(!format!("{error:?} {error}").contains("ARG_SECRET_CANARY"));

        let transport = FakePublicSocket::with_responses(vec![list_response(Some(1))]);
        let mut client =
            PublicSocketClient::with_negotiated_capabilities(transport, workload_capabilities());
        let error = client
            .workload_status(WorkloadTarget::parse("tools.host.d2b").unwrap())
            .await
            .unwrap_err();
        assert!(matches!(error, ClientError::UnexpectedResponse { .. }));
    });
}

#[test]
fn daemon_errors_map_by_kind_without_secret_text() {
    block_on(async {
        let transport =
            FakePublicSocket::with_responses(vec![daemon_error(Some(1), "helper-unavailable")]);
        let mut client =
            PublicSocketClient::with_negotiated_capabilities(transport, workload_capabilities());
        let error = client.workload_inventory().await.unwrap_err();
        assert!(matches!(
            error,
            ClientError::Daemon { ref kind } if kind == "helper-unavailable"
        ));
        let rendered = format!("{error:?} {error}");
        assert!(!rendered.contains("OUTPUT_SECRET_CANARY"));
        assert!(!rendered.contains("CWD_SECRET_CANARY"));

        let transport =
            FakePublicSocket::with_responses(vec![daemon_error(Some(1), "OUTPUT_SECRET_CANARY")]);
        let mut client =
            PublicSocketClient::with_negotiated_capabilities(transport, workload_capabilities());
        let error = client.workload_inventory().await.unwrap_err();
        assert!(matches!(
            error,
            ClientError::Daemon { ref kind } if kind == "invalid-daemon-error"
        ));
        assert!(!format!("{error:?} {error}").contains("OUTPUT_SECRET_CANARY"));
    });
}

#[test]
fn canonical_shell_targets_remain_available_to_legacy_clients() {
    block_on(async {
        let response = PublicResponse::Shell {
            op_id: None,
            response: ShellOpResponse::List(ShellListResult {
                default_name: ShellName::new("default").unwrap(),
                sessions: vec![],
            }),
        };
        let transport = FakePublicSocket::with_responses(vec![response]);
        let mut client = PublicSocketClient::new(transport);
        client.shell_list("tools.host.d2b").await.unwrap();
        let frames = client.into_inner().written_json_frames();
        assert_eq!(frames[0]["args"]["vm"], "tools.host.d2b");
    });
}

#[test]
fn unsafe_local_shell_requirement_is_explicit() {
    block_on(async {
        let client = PublicSocketClient::new(FakePublicSocket::default());
        let error = match client.requiring_unsafe_local_shell() {
            Ok(_) => panic!("unnegotiated client must fail"),
            Err(error) => error,
        };
        assert!(matches!(
            error,
            ClientError::Core(ToolkitError::FeatureUnavailable {
                feature: KnownFeatureFlag::UnsafeLocalShellV1
            })
        ));

        let response = PublicResponse::Shell {
            op_id: Some(1),
            response: ShellOpResponse::List(ShellListResult {
                default_name: ShellName::new("default").unwrap(),
                sessions: vec![],
            }),
        };
        let transport = FakePublicSocket::with_responses(vec![response]);
        let mut client = PublicSocketClient::with_negotiated_capabilities(
            transport,
            unsafe_shell_capabilities(),
        )
        .requiring_unsafe_local_shell()
        .unwrap();
        client.shell_list("tools.host.d2b").await.unwrap();
    });
}

#[test]
fn core_client_has_no_async_runtime_or_private_transport_dependency() {
    let manifest = include_str!("../Cargo.toml");
    for dependency in [
        "to".to_owned() + "kio",
        "async-".to_owned() + "std",
        "sm".to_owned() + "ol",
    ] {
        assert!(!manifest.contains(&dependency));
    }
    assert!(!manifest.contains(&("d2b-unsafe-local-".to_owned() + "helper")));
    assert!(!manifest.contains(&("d2b-priv-".to_owned() + "broker")));
}
