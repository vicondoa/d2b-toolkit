use crate::{read_json_frame, write_json_frame, ClientError};
use d2b_toolkit_core::{
    ErrorEnvelope, KnownFeatureFlag, LauncherExecArgs, LauncherExecResult, OperationId,
    ProtocolToken, PublicRequest, PublicResponse, WorkloadListArgs, WorkloadListResult, WorkloadOp,
    WorkloadOpResponse, WorkloadStatusArgs, WorkloadStatusResult, WorkloadTarget,
};
use futures::io::{AsyncRead, AsyncWrite};
use serde_json::Value;

use crate::PublicSocketClient;

impl<T> PublicSocketClient<T>
where
    T: AsyncRead + AsyncWrite + Unpin,
{
    pub async fn workload_inventory(&mut self) -> Result<WorkloadListResult, ClientError> {
        self.workload_list(None).await
    }

    pub async fn workload_list(
        &mut self,
        realm: Option<String>,
    ) -> Result<WorkloadListResult, ClientError> {
        let args = WorkloadListArgs::new(realm)?;
        let response = self.workload_round_trip(WorkloadOp::List(args)).await?;
        match response {
            WorkloadOpResponse::List(result) => Ok(result),
            _ => Err(ClientError::UnexpectedResponse {
                context: "listing workloads",
            }),
        }
    }

    pub async fn workload_status(
        &mut self,
        target: WorkloadTarget,
    ) -> Result<WorkloadStatusResult, ClientError> {
        let expected_target = target.clone();
        let response = self
            .workload_round_trip(WorkloadOp::Status(WorkloadStatusArgs { target }))
            .await?;
        match response {
            WorkloadOpResponse::Status(result)
                if result.workload.identity.canonical_target == expected_target =>
            {
                Ok(*result)
            }
            WorkloadOpResponse::Status(_) => Err(ClientError::CorrelationMismatch),
            _ => Err(ClientError::UnexpectedResponse {
                context: "reading workload status",
            }),
        }
    }

    pub async fn workload_status_str(
        &mut self,
        target: &str,
    ) -> Result<WorkloadStatusResult, ClientError> {
        self.workload_status(WorkloadTarget::parse(target)?).await
    }

    pub async fn launcher_exec(
        &mut self,
        target: WorkloadTarget,
        item_id: ProtocolToken,
        operation_id: OperationId,
    ) -> Result<LauncherExecResult, ClientError> {
        let expected_target = target.clone();
        let expected_item_id = item_id.clone();
        let expected_operation_id = operation_id.clone();
        let response = self
            .workload_round_trip(WorkloadOp::LauncherExec(LauncherExecArgs {
                target,
                item_id,
                operation_id,
            }))
            .await?;
        match response {
            WorkloadOpResponse::LauncherExec(result)
                if result.target == expected_target
                    && result.item_id == expected_item_id
                    && result.operation_id == expected_operation_id =>
            {
                Ok(result)
            }
            WorkloadOpResponse::LauncherExec(_) => Err(ClientError::CorrelationMismatch),
            _ => Err(ClientError::UnexpectedResponse {
                context: "executing launcher item",
            }),
        }
    }

    async fn workload_round_trip(
        &mut self,
        op: WorkloadOp,
    ) -> Result<WorkloadOpResponse, ClientError> {
        self.require_workload_features()?;
        let op_id = self.reserve_op_id();
        let request = PublicRequest::workload(Some(op_id), op);
        write_json_frame(&mut self.transport, &request, self.bounds).await?;
        let value: Value = read_json_frame(&mut self.transport, self.bounds).await?;
        let response = decode_workload_response(value)?;
        workload_response_for_op(response, op_id)
    }

    fn require_workload_features(&self) -> Result<(), ClientError> {
        self.require_feature(KnownFeatureFlag::ConfiguredLaunchV1)?;
        self.require_feature(KnownFeatureFlag::UnsafeLocalProviderV1)
    }
}

fn decode_workload_response(value: Value) -> Result<PublicResponse, ClientError> {
    match value.get("type").and_then(Value::as_str) {
        Some("workloadResponse" | "error") => {
            serde_json::from_value(value).map_err(|source| ClientError::Codec {
                context: "decoding public response",
                source,
            })
        }
        _ => Err(ClientError::UnexpectedResponse {
            context: "decoding workload response type",
        }),
    }
}

fn workload_response_for_op(
    response: PublicResponse,
    expected_op_id: u64,
) -> Result<WorkloadOpResponse, ClientError> {
    match response {
        PublicResponse::Workload { op_id, response }
            if op_id.is_none() || op_id == Some(expected_op_id) =>
        {
            Ok(response)
        }
        PublicResponse::Workload { .. } => Err(ClientError::CorrelationMismatch),
        PublicResponse::Error { op_id, error } if op_id == Some(expected_op_id) => {
            daemon_error(error)
        }
        PublicResponse::Error { op_id: None, error } => daemon_error(error),
        PublicResponse::Error { .. } => Err(ClientError::CorrelationMismatch),
        PublicResponse::Shell { .. } => Err(ClientError::UnexpectedResponse {
            context: "decoding workload response type",
        }),
    }
}

fn daemon_error(error: ErrorEnvelope) -> Result<WorkloadOpResponse, ClientError> {
    Err(ClientError::daemon_kind(error.kind))
}
