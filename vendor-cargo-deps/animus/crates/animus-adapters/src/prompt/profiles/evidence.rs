/// Redaction marker used by profile receipts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReceiptRedaction {
    /// Profile body is represented by a BLAKE3 hash, not raw text.
    BodyHashed,
}

/// Safe profile assembly receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Receipt {
    profile_id: super::support::Text,
    source_identity: super::support::Text,
    body_hash: super::support::Text,
    redaction: ReceiptRedaction,
    directive_mode: Option<super::types::Mode>,
    directive_chain_target: Option<super::types::Mode>,
    directive_issue_count: u64,
    chain_decision: Option<super::continuity::ChainDecision>,
    omitted_reason: Option<super::compose::OmittedReason>,
}

impl Receipt {
    /// Builds a safe receipt for profile assembly.
    #[must_use]
    pub fn from_profile(
        profile: &super::types::Descriptor,
        parsed_directives: &super::directives::Report,
        chain_plan: Option<&super::continuity::ChainPlan>,
        omitted_reason: Option<super::compose::OmittedReason>,
    ) -> Self {
        Self {
            profile_id: super::support::Text::from(profile.profile_id()),
            source_identity: super::support::Text::from(profile.source_identity()),
            body_hash: profile.body_hash(),
            redaction: ReceiptRedaction::BodyHashed,
            directive_mode: parsed_directives.mode(),
            directive_chain_target: parsed_directives.chain_target(),
            directive_issue_count: super::support::count_to_u64(
                parsed_directives.diagnostics().len(),
            ),
            chain_decision: chain_plan.map(super::continuity::ChainPlan::decision),
            omitted_reason,
        }
    }

    /// Returns the profile identifier.
    #[must_use]
    pub fn profile_id(&self) -> &str {
        self.profile_id.as_str()
    }

    /// Returns the safe source identity label.
    #[must_use]
    pub fn source_identity(&self) -> &str {
        self.source_identity.as_str()
    }

    /// Returns the BLAKE3 body hash as lowercase hexadecimal.
    #[must_use]
    pub fn body_hash(&self) -> &str {
        self.body_hash.as_str()
    }

    /// Returns the receipt redaction marker.
    #[must_use]
    pub const fn redaction(&self) -> ReceiptRedaction {
        self.redaction
    }

    /// Returns the parsed directive mode when valid.
    #[must_use]
    pub const fn directive_mode(&self) -> Option<super::types::Mode> {
        self.directive_mode
    }

    /// Returns the parsed chain target when valid.
    #[must_use]
    pub const fn directive_chain_target(&self) -> Option<super::types::Mode> {
        self.directive_chain_target
    }

    /// Returns the number of directive diagnostics without raw directive bodies.
    #[must_use]
    pub const fn directive_issue_count(&self) -> u64 {
        self.directive_issue_count
    }

    /// Returns the safe chain decision when a chain was planned.
    #[must_use]
    pub const fn chain_decision(&self) -> Option<super::continuity::ChainDecision> {
        self.chain_decision
    }

    /// Returns the safe omission reason when model-visible injection was withheld.
    #[must_use]
    pub const fn omitted_reason(&self) -> Option<super::compose::OmittedReason> {
        self.omitted_reason
    }
}
