//! Private bounded-worker protocol.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use zeroize::Zeroizing;

const FRAME_MAGIC: &[u8] = b"ANIMUS_SECRET_V1\0";
const LENGTH_BYTES: usize = size_of::<u64>();
/// Maximum direct worker request accepted independently of caller policy.
pub const WORKER_REQUEST_HARD_MAX_BYTES: usize = 65_536;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WorkerRequest {
    pub(crate) manifest_path: PathBuf,
    pub(crate) manifest_identity: String,
    pub(crate) profile: String,
    pub(crate) scope: String,
    pub(crate) declaration: String,
    pub(crate) provider_selector: String,
    pub(crate) provider_scheme: String,
    pub(crate) purpose: String,
    pub(crate) value_max_bytes: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ProtocolError {
    Serialize,
    InvalidFrame,
    ValueTooLarge,
}

pub(crate) fn encode_request(request: &WorkerRequest) -> Result<Vec<u8>, ProtocolError> {
    serde_json::to_vec(request).map_err(|_| ProtocolError::Serialize)
}

pub(crate) fn decode_request(bytes: &[u8]) -> Result<WorkerRequest, ProtocolError> {
    if bytes.is_empty() || bytes.len() > WORKER_REQUEST_HARD_MAX_BYTES {
        return Err(ProtocolError::InvalidFrame);
    }
    serde_json::from_slice(bytes).map_err(|_| ProtocolError::InvalidFrame)
}

pub(crate) fn encode_secret(value: &[u8]) -> Result<Zeroizing<Vec<u8>>, ProtocolError> {
    let length = u64::try_from(value.len()).map_err(|_| ProtocolError::ValueTooLarge)?;
    let capacity = FRAME_MAGIC
        .len()
        .checked_add(LENGTH_BYTES)
        .and_then(|size| size.checked_add(value.len()))
        .ok_or(ProtocolError::ValueTooLarge)?;
    let mut frame = Zeroizing::new(Vec::with_capacity(capacity));
    frame.extend_from_slice(FRAME_MAGIC);
    frame.extend_from_slice(&length.to_le_bytes());
    frame.extend_from_slice(value);
    Ok(frame)
}

pub(crate) fn decode_secret(
    frame: Zeroizing<Vec<u8>>,
    value_max_bytes: usize,
) -> Result<Zeroizing<Vec<u8>>, ProtocolError> {
    let header_bytes = FRAME_MAGIC.len() + LENGTH_BYTES;
    if frame.len() < header_bytes || &frame[..FRAME_MAGIC.len()] != FRAME_MAGIC {
        return Err(ProtocolError::InvalidFrame);
    }
    let mut encoded_length = [0_u8; LENGTH_BYTES];
    encoded_length.copy_from_slice(&frame[FRAME_MAGIC.len()..header_bytes]);
    let declared_length = usize::try_from(u64::from_le_bytes(encoded_length))
        .map_err(|_| ProtocolError::ValueTooLarge)?;
    let payload = &frame[header_bytes..];
    if declared_length != payload.len() || payload.len() > value_max_bytes {
        return Err(ProtocolError::ValueTooLarge);
    }
    let result = Zeroizing::new(payload.to_vec());
    drop(frame);
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALUE_MAX_BYTES: usize = 64;

    #[test]
    fn secret_frame_round_trips() {
        let frame = encode_secret(b"fixture-value").expect("encode");
        let decoded = decode_secret(frame, VALUE_MAX_BYTES).expect("decode");
        assert_eq!(decoded.as_slice(), b"fixture-value");
    }

    #[test]
    fn malformed_and_oversized_frames_fail_closed() {
        assert_eq!(
            decode_secret(Zeroizing::new(b"not-a-frame".to_vec()), VALUE_MAX_BYTES),
            Err(ProtocolError::InvalidFrame)
        );
        let frame = encode_secret(&[b'x'; VALUE_MAX_BYTES + 1]).expect("encode fixture");
        assert_eq!(
            decode_secret(frame, VALUE_MAX_BYTES),
            Err(ProtocolError::ValueTooLarge)
        );
    }
}
