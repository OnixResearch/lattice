//! Optional serde wire DTOs that always re-enter core validation.

use alloc::string::String;

use crate::HostControlError;
use crate::HostSelector;
use crate::HostSelectorDraft;

/// Structurally decoded selector fields.
#[derive(Clone, Debug, Eq, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(deny_unknown_fields)]
pub struct SelectorWire {
    /// Host identity.
    pub host_id: String,
    /// Resource identity.
    pub resource_id: String,
    /// Resource kind label.
    pub kind: String,
    /// Lifetime label.
    pub lifetime: String,
    /// Observed topology revision.
    pub revision: u64,
}

impl SelectorWire {
    /// Revalidate decoded fields through the semantic core.
    ///
    /// # Errors
    /// Returns the core selector diagnostic for malformed decoded values.
    pub fn validate(&self) -> Result<HostSelector, HostControlError> {
        HostSelector::validate(HostSelectorDraft {
            host_id: &self.host_id,
            resource_id: &self.resource_id,
            kind: &self.kind,
            lifetime: &self.lifetime,
            revision: self.revision,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn decoded_selector_revalidates() {
        let value = SelectorWire {
            host_id: "host:1".into(),
            resource_id: "pane:1".into(),
            kind: "pane".into(),
            lifetime: "live".into(),
            revision: 1,
        };
        assert!(value.validate().is_ok());
    }
    #[test]
    fn malformed_decoded_selector_is_rejected() {
        let value = SelectorWire {
            host_id: String::new(),
            resource_id: "pane:1".into(),
            kind: "pane".into(),
            lifetime: "live".into(),
            revision: 1,
        };
        assert!(value.validate().is_err());
    }
}
