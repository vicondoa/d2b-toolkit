use crate::{read_json_frame, write_json_frame, ClientError, FrameBounds};
use d2b_toolkit_core::{Hello, HelloFrame, HelloResponse};
use futures::io::{AsyncRead, AsyncWrite};

pub async fn send_hello<W>(
    writer: &mut W,
    hello: &Hello,
    bounds: FrameBounds,
) -> Result<(), ClientError>
where
    W: AsyncWrite + Unpin,
{
    write_json_frame(writer, &HelloFrame::new(hello.clone()), bounds).await
}

pub async fn read_hello_response<R>(
    reader: &mut R,
    bounds: FrameBounds,
) -> Result<HelloResponse, ClientError>
where
    R: AsyncRead + Unpin,
{
    let response = read_json_frame(reader, bounds).await?;
    validate_hello_response(&response)?;
    Ok(response)
}

pub fn validate_hello_response(response: &HelloResponse) -> Result<(), ClientError> {
    match response {
        HelloResponse::HelloOk(_) => Ok(()),
        HelloResponse::HelloRejected(_) => Err(ClientError::Hello { reason: "rejected" }),
    }
}

#[cfg(test)]
mod tests {
    use super::{read_hello_response, send_hello, validate_hello_response};
    use d2b_toolkit_core::{
        ErrorEnvelope, Hello, HelloOk, HelloRejected, HelloRejectedReason, HelloResponse,
        KnownFeatureFlag, Version,
    };
    use futures::executor::block_on;
    use futures::io::Cursor;
    use serde::Serialize;
    use serde_json::Value;

    #[test]
    fn rejects_rejected_hello() {
        let response = HelloResponse::HelloRejected(HelloRejected {
            reason: HelloRejectedReason::VersionMismatch,
            error: ErrorEnvelope {
                kind: "wire-version-mismatch".into(),
                exit_code: 52,
                message: "version mismatch".into(),
                remediation: "upgrade client or daemon".into(),
            },
        });
        let err = validate_hello_response(&response).unwrap_err();
        assert!(err.to_string().contains("hello negotiation failed"));
    }

    #[test]
    fn accepts_hello_ok() {
        let response = HelloResponse::HelloOk(HelloOk {
            server_version: Version::new("0.4.0"),
            selected_version: Version::new("0.4.0"),
            capabilities: vec![],
        });
        validate_hello_response(&response).expect("valid hello ok");
    }

    #[test]
    fn hello_uses_daemon_frame_shape() {
        block_on(async {
            let mut client_to_daemon = Cursor::new(Vec::new());
            send_hello(
                &mut client_to_daemon,
                &Hello::toolkit_client(vec![KnownFeatureFlag::TypedErrors.wire_value()]),
                crate::FrameBounds::new(256),
            )
            .await
            .unwrap();
            let bytes = client_to_daemon.into_inner();
            let len = u32::from_le_bytes(bytes[..4].try_into().unwrap()) as usize;
            let payload: Value = serde_json::from_slice(&bytes[4..4 + len]).unwrap();
            assert_eq!(payload["type"], "hello");
            assert_eq!(payload["supportedFeatures"][0], "typed-errors");

            let response = HelloResponse::HelloOk(HelloOk {
                server_version: Version::new("0.4.0"),
                selected_version: Version::new("0.4.0"),
                capabilities: vec![KnownFeatureFlag::TypedErrors.wire_value()],
            });
            let mut daemon_to_client = Cursor::new(frame(&response));
            read_hello_response(&mut daemon_to_client, crate::FrameBounds::new(256))
                .await
                .unwrap();
        });
    }

    fn frame<T: Serialize>(value: &T) -> Vec<u8> {
        let payload = serde_json::to_vec(value).unwrap();
        let mut frame = Vec::with_capacity(payload.len() + 4);
        frame.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        frame.extend_from_slice(&payload);
        frame
    }
}
