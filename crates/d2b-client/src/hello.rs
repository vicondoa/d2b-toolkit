use crate::{read_json_frame, write_json_frame, ClientError, FrameBounds};
use d2b_toolkit_core::{Hello, HelloResponse, CURRENT_PROTOCOL_VERSION};
use futures::io::{AsyncRead, AsyncWrite};

pub async fn send_hello<W>(
    writer: &mut W,
    hello: &Hello,
    bounds: FrameBounds,
) -> Result<(), ClientError>
where
    W: AsyncWrite + Unpin,
{
    write_json_frame(writer, hello, bounds).await
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
        HelloResponse::HelloOk {
            protocol_version, ..
        } if *protocol_version == CURRENT_PROTOCOL_VERSION => Ok(()),
        HelloResponse::HelloOk { .. } => Err(ClientError::Hello { reason: "version" }),
        HelloResponse::HelloRejected { .. } => Err(ClientError::Hello { reason: "rejected" }),
    }
}

#[cfg(test)]
mod tests {
    use super::validate_hello_response;
    use d2b_toolkit_core::HelloResponse;

    #[test]
    fn rejects_rejected_hello() {
        let response = HelloResponse::HelloRejected {
            reason: "unsupported".into(),
        };
        let err = validate_hello_response(&response).unwrap_err();
        assert!(err.to_string().contains("hello negotiation failed"));
    }

    #[test]
    fn accepts_matching_protocol_version() {
        let response = HelloResponse::HelloOk {
            protocol_version: 1,
            accepted_features: vec![],
        };
        validate_hello_response(&response).expect("valid hello ok");
    }
}
