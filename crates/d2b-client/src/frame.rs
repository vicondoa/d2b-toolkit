use crate::ClientError;
use futures::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use serde::{de::DeserializeOwned, Serialize};

pub const DEFAULT_MAX_FRAME_LEN: usize = 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FrameBounds {
    max_len: usize,
}

impl FrameBounds {
    pub const fn new(max_len: usize) -> Self {
        Self { max_len }
    }

    pub const fn default_public_daemon() -> Self {
        Self::new(DEFAULT_MAX_FRAME_LEN)
    }

    pub const fn max_len(self) -> usize {
        self.max_len
    }

    fn check(self, len: usize) -> Result<(), ClientError> {
        if len > self.max_len {
            return Err(d2b_toolkit_core::ToolkitError::FrameTooLarge {
                len,
                max: self.max_len,
            }
            .into());
        }
        Ok(())
    }
}

impl Default for FrameBounds {
    fn default() -> Self {
        Self::default_public_daemon()
    }
}

pub async fn read_frame<R>(reader: &mut R, bounds: FrameBounds) -> Result<Vec<u8>, ClientError>
where
    R: AsyncRead + Unpin,
{
    let mut prefix = [0_u8; 4];
    reader
        .read_exact(&mut prefix)
        .await
        .map_err(|source| d2b_toolkit_core::ToolkitError::Io {
            context: "reading frame length",
            source,
        })?;
    let len = u32::from_le_bytes(prefix) as usize;
    bounds.check(len)?;

    let mut payload = vec![0_u8; len];
    reader
        .read_exact(&mut payload)
        .await
        .map_err(|source| d2b_toolkit_core::ToolkitError::Io {
            context: "reading frame payload",
            source,
        })?;
    Ok(payload)
}

pub async fn write_frame<W>(
    writer: &mut W,
    payload: &[u8],
    bounds: FrameBounds,
) -> Result<(), ClientError>
where
    W: AsyncWrite + Unpin,
{
    bounds.check(payload.len())?;
    let len = u32::try_from(payload.len()).map_err(|_| {
        d2b_toolkit_core::ToolkitError::FrameTooLarge {
            len: payload.len(),
            max: u32::MAX as usize,
        }
    })?;
    writer
        .write_all(&len.to_le_bytes())
        .await
        .map_err(|source| d2b_toolkit_core::ToolkitError::Io {
            context: "writing frame length",
            source,
        })?;
    writer
        .write_all(payload)
        .await
        .map_err(|source| d2b_toolkit_core::ToolkitError::Io {
            context: "writing frame payload",
            source,
        })?;
    writer
        .flush()
        .await
        .map_err(|source| d2b_toolkit_core::ToolkitError::Io {
            context: "flushing frame",
            source,
        })?;
    Ok(())
}

pub async fn read_json_frame<R, T>(reader: &mut R, bounds: FrameBounds) -> Result<T, ClientError>
where
    R: AsyncRead + Unpin,
    T: DeserializeOwned,
{
    let payload = read_frame(reader, bounds).await?;
    serde_json::from_slice(&payload).map_err(|source| ClientError::Codec {
        context: "decoding json frame",
        source,
    })
}

pub async fn write_json_frame<W, T>(
    writer: &mut W,
    value: &T,
    bounds: FrameBounds,
) -> Result<(), ClientError>
where
    W: AsyncWrite + Unpin,
    T: Serialize,
{
    let payload = serde_json::to_vec(value).map_err(|source| ClientError::Codec {
        context: "encoding json frame",
        source,
    })?;
    write_frame(writer, &payload, bounds).await
}

#[cfg(test)]
mod tests {
    use super::{read_frame, read_json_frame, write_json_frame, FrameBounds};
    use futures::executor::block_on;
    use futures::io::Cursor;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
    struct Message {
        kind: String,
    }

    #[test]
    fn json_frame_round_trip_is_runtime_agnostic() {
        block_on(async {
            let mut transport = Cursor::new(Vec::new());
            write_json_frame(
                &mut transport,
                &Message {
                    kind: "hello".into(),
                },
                FrameBounds::new(128),
            )
            .await
            .unwrap();
            transport.set_position(0);
            let decoded: Message = read_json_frame(&mut transport, FrameBounds::new(128))
                .await
                .unwrap();
            assert_eq!(decoded.kind, "hello");
        });
    }

    #[test]
    fn oversize_frame_is_rejected_before_write() {
        block_on(async {
            let mut transport = Cursor::new(Vec::new());
            let err = write_json_frame(
                &mut transport,
                &Message {
                    kind: "this payload is intentionally too long".into(),
                },
                FrameBounds::new(8),
            )
            .await
            .unwrap_err();
            let debug = format!("{err:?}");
            assert!(debug.contains("FrameTooLarge"));
            assert!(transport.get_ref().is_empty());
        });
    }

    #[test]
    fn frame_length_prefix_is_little_endian() {
        block_on(async {
            let mut transport = Cursor::new(Vec::new());
            write_json_frame(
                &mut transport,
                &Message {
                    kind: "little".into(),
                },
                FrameBounds::new(128),
            )
            .await
            .unwrap();
            let bytes = transport.into_inner();
            let declared = u32::from_le_bytes(bytes[..4].try_into().unwrap()) as usize;
            assert_eq!(declared, bytes.len() - 4);
        });
    }

    #[test]
    fn read_rejects_oversize_declared_length_before_payload() {
        block_on(async {
            let mut bytes = Vec::new();
            bytes.extend_from_slice(&9_u32.to_le_bytes());
            bytes.extend_from_slice(b"short");
            let mut transport = Cursor::new(bytes);
            let err = read_frame(&mut transport, FrameBounds::new(8))
                .await
                .unwrap_err();
            assert!(format!("{err:?}").contains("FrameTooLarge"));
        });
    }
}
