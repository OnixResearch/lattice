//! Pure capability-conditional prompt composition.
//!
//! A prompt fragment renders only when every tool or skill it names is in the
//! resolved availability set for the current turn. Availability is an explicit
//! composition input, never an inference from prompt text. Compositions with
//! unresolvable capability inputs fail closed before any prompt is emitted.

#![allow(unknown_lints)]
#![cfg_attr(
    animus_octet,
    allow(
        tigerstyle::assertion_density,
        tigerstyle::function_length,
        reason = "capability composition rejects untrusted labels with typed errors instead of panicking assertions"
    )
)]
#![cfg_attr(
    animus_octet,
    allow(
        tigerstyle::non_trait_imports,
        reason = "the pure composition core uses an explicit ordered-set dependency for deterministic identities"
    )
)]
#![cfg_attr(
    animus_octet,
    allow(
        tigerstyle::bool_naming,
        tigerstyle::numeric_units,
        reason = "published capability vocabulary uses field names fixed by its versioned contract"
    )
)]

use std::collections::BTreeSet;

/// Capability-conditional composition schema identity.
pub const CAPABILITY_CONDITIONAL_SCHEMA: &str = "animus.capability-conditional.v1";
/// Receipt hash domain.
pub const CAPABILITY_CONDITIONAL_RECEIPT_DOMAIN: &str = "animus/capability-conditional-receipt";
/// Maximum safe label length in bytes.
pub const MAX_CAPABILITY_LABEL_BYTES: usize = 160;

/// One validated capability label (a tool or skill name).
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CapabilityLabel(String);

impl CapabilityLabel {
    /// Validates a bounded safe capability label.
    ///
    /// # Errors
    ///
    /// Returns [`CapabilityCompositionError::InvalidLabel`] for malformed text.
    pub fn parse(value: &str) -> Result<Self, CapabilityCompositionError> {
        if value.is_empty()
            || value.len() > MAX_CAPABILITY_LABEL_BYTES
            || value.chars().any(char::is_control)
        {
            return Err(CapabilityCompositionError::InvalidLabel);
        }
        Ok(Self(value.to_string()))
    }

    /// Returns the bounded label text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

/// Capability family a fragment name participates in.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FragmentKind {
    /// Native tool capability.
    Tool,
    /// Reviewed skill capability.
    Skill,
}

impl FragmentKind {
    /// Returns the stable kind label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Tool => "tool",
            Self::Skill => "skill",
        }
    }
}

/// One conditional prompt fragment.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PromptFragment {
    /// Stable fragment name.
    pub name: String,
    /// Fragment capability family.
    pub kind: FragmentKind,
    /// Every tool or skill this fragment requires across all families.
    pub requires: Vec<CapabilityLabel>,
}

/// Explicit resolved availability set for the current turn.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AvailabilitySet {
    /// Tools available for the turn.
    pub tools: BTreeSet<CapabilityLabel>,
    /// Skills available for the turn.
    pub skills: BTreeSet<CapabilityLabel>,
}

impl AvailabilitySet {
    /// Returns whether the set is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.tools.is_empty() && self.skills.is_empty()
    }

    /// Returns a deterministic BLAKE3 identity over the sorted availability.
    #[must_use]
    pub fn identity(&self) -> String {
        let mut parts = Vec::with_capacity(self.tools.len().saturating_add(self.skills.len()));
        for label in &self.tools {
            parts.push(format!("tool:{}", label.as_str()));
        }
        for label in &self.skills {
            parts.push(format!("skill:{}", label.as_str()));
        }
        hash_parts(
            "animus/capability-availability",
            &parts.iter().map(String::as_str).collect::<Vec<_>>(),
        )
    }
}

/// Bounded omission reason.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OmissionReason {
    /// A required tool is not in the availability set.
    ToolUnavailable,
    /// A required skill is not in the availability set.
    SkillUnavailable,
}

impl OmissionReason {
    /// Returns the stable reason label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ToolUnavailable => "tool_unavailable",
            Self::SkillUnavailable => "skill_unavailable",
        }
    }
}

/// One dropped fragment recorded without raw instruction text.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompositionOmission {
    /// Fragment name.
    pub fragment_name: String,
    /// Omission reason.
    pub reason: OmissionReason,
}

/// Fully composed conditional prompt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComposedPrompt {
    /// Fragments whose every required capability is available.
    pub included: Vec<PromptFragment>,
    /// Fragments dropped because a required capability is unavailable.
    pub omitted: Vec<CompositionOmission>,
    /// BLAKE3 identity of the resolved availability set.
    pub availability_identity: String,
}

/// Metadata-only composition receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CapabilityCompositionReceipt {
    /// Canonical BLAKE3 receipt identity.
    pub identity: String,
    /// Schema identity.
    pub schema: &'static str,
    /// Included fragment count.
    pub included_count: u32,
    /// Omitted fragment count.
    pub omitted_count: u32,
    /// Availability-set identity.
    pub availability_identity: String,
}

/// Typed composition error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CapabilityCompositionError {
    /// A label is empty, oversized, or contains control characters.
    InvalidLabel,
    /// The availability set is empty and no capability resolves.
    EmptyAvailability,
}

impl CapabilityCompositionError {
    /// Returns the stable error label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InvalidLabel => "invalid_label",
            Self::EmptyAvailability => "empty_availability",
        }
    }
}

/// Composes fragments conditionally against an explicit availability set.
///
/// Every fragment renders only when all of its required capabilities are
/// present. An empty availability set is an unresolvable capability input and
/// fails closed.
///
/// # Errors
///
/// Returns [`CapabilityCompositionError::EmptyAvailability`] or
/// [`CapabilityCompositionError::InvalidLabel`] before any prompt is emitted.
pub fn compose_capability_conditional(
    availability: &AvailabilitySet,
    fragments: &[PromptFragment],
) -> Result<ComposedPrompt, CapabilityCompositionError> {
    if availability.is_empty() {
        return Err(CapabilityCompositionError::EmptyAvailability);
    }
    let mut included = Vec::with_capacity(fragments.len());
    let mut omitted = Vec::with_capacity(fragments.len());
    for fragment in fragments {
        if fragment.name.is_empty()
            || fragment.name.len() > MAX_CAPABILITY_LABEL_BYTES
            || fragments
                .iter()
                .filter(|candidate| candidate.name == fragment.name)
                .count()
                > 1
        {
            return Err(CapabilityCompositionError::InvalidLabel);
        }
        let mut is_missing = false;
        for capability in &fragment.requires {
            let available =
                availability.tools.contains(capability) || availability.skills.contains(capability);
            if !available {
                is_missing = true;
                break;
            }
        }
        if is_missing {
            omitted.push(CompositionOmission {
                fragment_name: fragment.name.clone(),
                reason: match fragment.kind {
                    FragmentKind::Tool => OmissionReason::ToolUnavailable,
                    FragmentKind::Skill => OmissionReason::SkillUnavailable,
                },
            });
        } else {
            included.push(fragment.clone());
        }
    }
    Ok(ComposedPrompt {
        included,
        omitted,
        availability_identity: availability.identity(),
    })
}

/// Builds a metadata-only composition receipt.
///
/// # Errors
///
/// Returns [`CapabilityCompositionError::InvalidLabel`] for an empty
/// availability identity.
pub fn build_capability_composition_receipt(
    composed: &ComposedPrompt,
) -> Result<CapabilityCompositionReceipt, CapabilityCompositionError> {
    if composed.availability_identity.is_empty() {
        return Err(CapabilityCompositionError::InvalidLabel);
    }
    let included_count = u32::try_from(composed.included.len())
        .map_err(|_| CapabilityCompositionError::InvalidLabel)?;
    let omitted_count = u32::try_from(composed.omitted.len())
        .map_err(|_| CapabilityCompositionError::InvalidLabel)?;
    let identity = hash_parts(
        CAPABILITY_CONDITIONAL_RECEIPT_DOMAIN,
        &[
            CAPABILITY_CONDITIONAL_SCHEMA,
            &included_count.to_string(),
            &omitted_count.to_string(),
            composed.availability_identity.as_str(),
        ],
    );
    Ok(CapabilityCompositionReceipt {
        identity,
        schema: CAPABILITY_CONDITIONAL_SCHEMA,
        included_count,
        omitted_count,
        availability_identity: composed.availability_identity.clone(),
    })
}

/// Rejects raw material in composition evidence text.
///
/// # Errors
///
/// Returns [`CapabilityCompositionError::InvalidLabel`] when a leak sentinel
/// appears.
pub fn validate_capability_evidence_text(value: &str) -> Result<(), CapabilityCompositionError> {
    let lower = value.to_ascii_lowercase();
    for forbidden in [
        "raw prompt",
        "provider payload",
        "instruction text",
        "/home/",
    ] {
        if lower.contains(forbidden) {
            return Err(CapabilityCompositionError::InvalidLabel);
        }
    }
    Ok(())
}

fn hash_parts(domain: &str, parts: &[&str]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain.as_bytes());
    for part in parts {
        hasher.update(&[0]);
        hasher.update(part.as_bytes());
    }
    format!("b3:{}", hasher.finalize().to_hex())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn label(value: &str) -> CapabilityLabel {
        CapabilityLabel::parse(value).expect("valid label")
    }

    fn tools(values: &[&str]) -> BTreeSet<CapabilityLabel> {
        values.iter().map(|value| label(value)).collect()
    }

    fn skills(values: &[&str]) -> BTreeSet<CapabilityLabel> {
        values.iter().map(|value| label(value)).collect()
    }

    fn fragment(name: &str, kind: FragmentKind, requires: &[&str]) -> PromptFragment {
        PromptFragment {
            name: name.to_string(),
            kind,
            requires: requires.iter().map(|value| label(value)).collect(),
        }
    }

    fn availability() -> AvailabilitySet {
        AvailabilitySet {
            tools: tools(&["web-search"]),
            skills: skills(&["review-skill"]),
        }
    }

    #[test]
    fn positive_available_fragments_render_and_missing_are_omitted() {
        // r[verify agent_harness_runtime.capability_conditional_prompt.verification]
        let fragments = vec![
            fragment("search-fragment", FragmentKind::Tool, &["web-search"]),
            fragment("review-fragment", FragmentKind::Skill, &["review-skill"]),
            fragment("ghost-tool-fragment", FragmentKind::Tool, &["missing-tool"]),
        ];
        let composed =
            compose_capability_conditional(&availability(), &fragments).expect("composition");
        assert_eq!(composed.included.len(), 2);
        assert_eq!(composed.omitted.len(), 1);
        assert_eq!(composed.omitted[0].fragment_name, "ghost-tool-fragment");
        assert_eq!(composed.omitted[0].reason, OmissionReason::ToolUnavailable);
        assert!(!composed.availability_identity.is_empty());
    }

    #[test]
    fn positive_fragment_can_require_capabilities_from_either_family() {
        // r[verify agent_harness_runtime.capability_conditional_prompt.conditional]
        let fragments = vec![fragment(
            "combined",
            FragmentKind::Tool,
            &["web-search", "review-skill"],
        )];
        let composed =
            compose_capability_conditional(&availability(), &fragments).expect("composition");
        assert_eq!(composed.included.len(), 1);
    }

    #[test]
    fn positive_receipt_is_metadata_only() {
        // r[verify agent_harness_runtime.capability_conditional_prompt.receipts]
        let fragments = vec![fragment(
            "search-fragment",
            FragmentKind::Tool,
            &["web-search"],
        )];
        let composed =
            compose_capability_conditional(&availability(), &fragments).expect("composition");
        let receipt = build_capability_composition_receipt(&composed).expect("receipt");
        assert_eq!(receipt.included_count, 1);
        assert_eq!(receipt.omitted_count, 0);
        assert!(receipt.identity.starts_with('b'));
        assert!(validate_capability_evidence_text(&receipt.identity).is_ok());
        assert!(
            validate_capability_evidence_text("raw prompt and instruction text leaked").is_err(),
            "evidence with raw material must be rejected"
        );
    }

    #[test]
    fn negative_missing_tool_never_produces_instructions() {
        // r[verify agent_harness_runtime.capability_conditional_prompt.missing]
        // r[verify agent_harness_runtime.capability_conditional_prompt.fail_closed]
        let fragments = vec![fragment(
            "role-with-missing-tool",
            FragmentKind::Tool,
            &["tool-that-does-not-exist"],
        )];
        let composed =
            compose_capability_conditional(&availability(), &fragments).expect("composition");
        assert!(composed.included.is_empty());
        assert_eq!(composed.omitted.len(), 1);
        assert_eq!(composed.omitted[0].reason, OmissionReason::ToolUnavailable);
        // No instruction is produced for the missing tool by construction.
        assert!(
            composed
                .omitted
                .iter()
                .all(|omission| omission.fragment_name == "role-with-missing-tool")
        );
    }

    #[test]
    fn negative_empty_availability_and_invalid_labels_fail_closed() {
        // r[verify agent_harness_runtime.capability_conditional_prompt.fail_closed]
        let empty = AvailabilitySet::default();
        assert_eq!(
            compose_capability_conditional(&empty, &[]),
            Err(CapabilityCompositionError::EmptyAvailability)
        );
        let duplicated = vec![
            fragment("dup", FragmentKind::Tool, &["web-search"]),
            fragment("dup", FragmentKind::Tool, &["web-search"]),
        ];
        assert_eq!(
            compose_capability_conditional(&availability(), &duplicated),
            Err(CapabilityCompositionError::InvalidLabel)
        );
        assert_eq!(
            CapabilityLabel::parse(""),
            Err(CapabilityCompositionError::InvalidLabel)
        );
    }
}
