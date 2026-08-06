//! Optional Preserves codec over the serde wire DTO.

use alloc::string::String;
use alloc::vec::Vec;

use crate::wire::SelectorWire;
use crate::HostControlError;
use crate::HostSelector;

/// Bounded codec diagnostic that does not grant transport authority.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PreservesWireError {
    /// Preserves syntax or structural decoding failed.
    Malformed,
    /// Decoded data failed semantic core validation.
    Invalid(HostControlError),
    /// Text encoding did not produce UTF-8.
    InvalidUtf8,
}

/// Decode Preserves text and revalidate selector semantics.
///
/// # Errors
/// Returns syntax or semantic validation diagnostics.
pub fn decode_selector_text(text: &str) -> Result<HostSelector, PreservesWireError> {
    let wire: SelectorWire = preserves::de::from_text(text).map_err(|_error| PreservesWireError::Malformed)?;
    wire.validate().map_err(PreservesWireError::Invalid)
}

/// Encode a structurally valid selector wire value as Preserves text.
///
/// # Errors
/// Returns semantic, codec, or UTF-8 diagnostics.
pub fn encode_selector_text(wire: &SelectorWire) -> Result<String, PreservesWireError> {
    wire.validate().map_err(PreservesWireError::Invalid)?;
    let mut buffer = Vec::new();
    let mut writer = preserves::value::TextWriter::new(&mut buffer);
    preserves::ser::to_writer(&mut writer, wire).map_err(|_error| PreservesWireError::Malformed)?;
    String::from_utf8(buffer).map_err(|_error| PreservesWireError::InvalidUtf8)
}

#[cfg(test)]
mod tests {
    use alloc::string::ToString;

    use super::*;

    const REVISION: u64 = 3;

    fn valid_wire() -> SelectorWire {
        SelectorWire {
            host_id: "host:1".to_string(),
            resource_id: "pane:1".to_string(),
            kind: "pane".to_string(),
            lifetime: "live".to_string(),
            revision: REVISION,
        }
    }

    #[test]
    fn preserves_round_trip_revalidates() {
        let text = encode_selector_text(&valid_wire()).expect("valid selector should encode");
        let selector = decode_selector_text(&text).expect("encoded selector should decode");
        assert_eq!(selector.revision(), REVISION);
    }

    #[test]
    fn malformed_and_semantically_invalid_preserves_fail_closed() {
        assert_eq!(decode_selector_text("<"), Err(PreservesWireError::Malformed));
        let mut invalid = valid_wire();
        invalid.host_id = String::new();
        assert!(matches!(encode_selector_text(&invalid), Err(PreservesWireError::Invalid(_))));
    }
}
