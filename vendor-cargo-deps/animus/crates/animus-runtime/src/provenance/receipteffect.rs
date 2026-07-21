/// Borrowed safe receipt fields before validation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvenanceReceiptDraft<'a> {
    /// Receipt entry identity.
    pub id: &'a str,
    /// Parent entry identities.
    pub parents: &'a [&'a str],
    /// Receipt status label.
    pub status: &'a str,
    /// Optional safe issue code.
    pub issue_code: Option<&'a str>,
    /// Safe actor label.
    pub actor_label: &'a str,
    /// Safe session label.
    pub session_label: &'a str,
    /// Safe trace label.
    pub trace_label: &'a str,
    /// Bounded event count.
    pub event_count: u64,
    /// BLAKE3 artifact hashes.
    pub artifact_hashes: &'a [&'a str],
    /// Optional mutation kind.
    pub mutation_kind: Option<&'a str>,
    /// Optional effect kind.
    pub effect_kind: Option<&'a str>,
    /// Optional backend label.
    pub backend_label: Option<&'a str>,
    /// Optional interchange label.
    pub interchange_label: Option<&'a str>,
    /// Redaction markers.
    pub redaction_markers: &'a [&'a str],
}

/// Safe runtime/provenance receipt metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvenanceReceipt {
    /// Receipt entry identity.
    pub id: Text,
    /// Parent entry identities.
    pub parents: Collection<Text>,
    /// Receipt status label.
    pub status: Text,
    /// Optional safe issue code.
    pub issue_code: Option<Text>,
    /// Safe actor label.
    pub actor_label: Text,
    /// Safe session label.
    pub session_label: Text,
    /// Safe trace label.
    pub trace_label: Text,
    /// Bounded event count.
    pub event_count: u64,
    /// BLAKE3 artifact hashes.
    pub artifact_hashes: Collection<Text>,
    /// Optional mutation kind.
    pub mutation_kind: Option<Text>,
    /// Optional effect kind.
    pub effect_kind: Option<Text>,
    /// Optional backend label.
    pub backend_label: Option<Text>,
    /// Optional interchange label.
    pub interchange_label: Option<Text>,
    /// Redaction markers.
    pub redaction_markers: Collection<Text>,
    /// Deterministic BLAKE3 hash over safe fields.
    pub receipt_hash: Text,
}

impl ProvenanceReceipt {
    /// Validates a safe receipt.
    ///
    /// # Errors
    ///
    /// Returns [`ProvenanceError`] when metadata is malformed, unbounded, or
    /// contains raw prompt, provider payload, credential, full-path, raw tool,
    /// raw model, raw diff, or unbounded-output markers.
    pub fn validate(draft: &ProvenanceReceiptDraft<'_>) -> Result<Self, ProvenanceError> {
        let id = validate_safe_text("receipt_id", draft.id.to_owned())?;
        let parents = validate_parent_ids(draft.parents)?;
        let status = validate_safe_text("receipt_status", draft.status.to_owned())?;
        let issue_code = validate_optional_safe("issue_code", draft.issue_code)?;
        let actor_label = validate_safe_text("actor_label", draft.actor_label.to_owned())?;
        let session_label = validate_safe_text("session_label", draft.session_label.to_owned())?;
        let trace_label = validate_safe_text("trace_label", draft.trace_label.to_owned())?;
        validate_count("event_count", draft.event_count)?;
        let artifact_hashes = validate_hashes("artifact_hash", draft.artifact_hashes)?;
        let mutation_kind = validate_optional_safe("mutation_kind", draft.mutation_kind)?;
        let effect_kind = validate_optional_safe("effect_kind", draft.effect_kind)?;
        let backend_label = validate_optional_safe("backend_label", draft.backend_label)?;
        let interchange_label =
            validate_optional_safe("interchange_label", draft.interchange_label)?;
        let redaction_markers = validate_safe_list("redaction_marker", draft.redaction_markers)?;
        let receipt_hash = receipt_hash(ReceiptHashInput {
            id: id.as_str(),
            status: status.as_str(),
            issue_code: issue_code.as_deref(),
            actor_label: actor_label.as_str(),
            session_label: session_label.as_str(),
            trace_label: trace_label.as_str(),
            event_count: draft.event_count,
            artifact_hashes: artifact_hashes.as_slice(),
            mutation_kind: mutation_kind.as_deref(),
            effect_kind: effect_kind.as_deref(),
            backend_label: backend_label.as_deref(),
            interchange_label: interchange_label.as_deref(),
            redaction_markers: redaction_markers.as_slice(),
        });
        Ok(Self {
            id,
            parents,
            status,
            issue_code,
            actor_label,
            session_label,
            trace_label,
            event_count: draft.event_count,
            artifact_hashes,
            mutation_kind,
            effect_kind,
            backend_label,
            interchange_label,
            redaction_markers,
            receipt_hash,
        })
    }
}

#[derive(Clone, Copy)]
struct ReceiptHashInput<'a> {
    id: &'a str,
    status: &'a str,
    issue_code: Option<&'a str>,
    actor_label: &'a str,
    session_label: &'a str,
    trace_label: &'a str,
    event_count: u64,
    artifact_hashes: &'a [Text],
    mutation_kind: Option<&'a str>,
    effect_kind: Option<&'a str>,
    backend_label: Option<&'a str>,
    interchange_label: Option<&'a str>,
    redaction_markers: &'a [Text],
}

/// Tool/effect outcome class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProvenanceToolEffectStatus {
    /// Effect completed.
    Completed,
    /// Required service was unavailable.
    ServiceUnavailable,
    /// Capability policy denied the effect.
    CapabilityDenied,
    /// Effect was cancelled before completion.
    Cancelled,
    /// Effect completed with bounded model-visible output.
    Truncated,
}

impl ProvenanceToolEffectStatus {
    /// Returns the stable status label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Completed => COMPLETED_STATUS,
            Self::ServiceUnavailable => UNAVAILABLE_STATUS,
            Self::CapabilityDenied => DENIED_STATUS,
            Self::Cancelled => CANCELLED_STATUS,
            Self::Truncated => TRUNCATED_STATUS,
        }
    }

    const fn claims_completed_effect(self) -> bool {
        matches!(self, Self::Completed | Self::Truncated)
    }
}

/// Borrowed adapter-safe tool/effect outcome draft.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProvenanceToolEffectDraft<'a> {
    /// Safe tool kind.
    pub tool_kind: &'a str,
    /// Safe service label.
    pub service_label: &'a str,
    /// Effect ability label.
    pub effect_ability: &'a str,
    /// Effect resource label.
    pub effect_resource: &'a str,
    /// Outcome status.
    pub status: ProvenanceToolEffectStatus,
    /// Original output byte count when known.
    pub original_output_bytes: Option<u64>,
    /// Retained output byte count when known.
    pub retained_output_bytes: Option<u64>,
}

/// Adapter-safe tool/effect outcome DTO.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProvenanceToolEffect {
    /// Safe tool kind.
    pub tool_kind: Text,
    /// Safe service label.
    pub service_label: Text,
    /// Effect ability label.
    pub effect_ability: Text,
    /// Effect resource label.
    pub effect_resource: Text,
    /// Outcome status.
    pub status: ProvenanceToolEffectStatus,
    /// Original output byte count when known.
    pub original_output_bytes: Option<u64>,
    /// Retained output byte count when known.
    pub retained_output_bytes: Option<u64>,
}

impl ProvenanceToolEffect {
    /// Validates an adapter-safe effect outcome.
    ///
    /// # Errors
    ///
    /// Returns [`ProvenanceError`] if labels are unsafe or counts are unbounded.
    pub fn validate(draft: &ProvenanceToolEffectDraft<'_>) -> Result<Self, ProvenanceError> {
        if let Some(count) = draft.original_output_bytes {
            validate_count("original_output_bytes", count)?;
        }
        if let Some(count) = draft.retained_output_bytes {
            validate_count("retained_output_bytes", count)?;
        }
        Ok(Self {
            tool_kind: validate_safe_text("tool_kind", draft.tool_kind.to_owned())?,
            service_label: validate_safe_text("service_label", draft.service_label.to_owned())?,
            effect_ability: validate_safe_text("effect_ability", draft.effect_ability.to_owned())?,
            effect_resource: validate_safe_text(
                "effect_resource",
                draft.effect_resource.to_owned(),
            )?,
            status: draft.status,
            original_output_bytes: draft.original_output_bytes,
            retained_output_bytes: draft.retained_output_bytes,
        })
    }

    /// Converts a completed effect into fact and receipt evidence.
    ///
    /// # Errors
    ///
    /// Returns [`ProvenanceError`] if the outcome did not complete and therefore
    /// must not claim completed effect evidence.
    pub fn completed_evidence(
        &self,
        receipt_id: &str,
        artifact_hash: &str,
    ) -> Result<(ProvenanceFact, ProvenanceReceipt), ProvenanceError> {
        if !self.status.claims_completed_effect() {
            return Err(ProvenanceError::new(
                ProvenanceErrorKind::IncompleteEffect,
                "tool_effect_status",
                self.status.as_str(),
                "effect did not complete before evidence request",
            ));
        }
        let fact = ProvenanceFact::ToolActivity(ProvenanceFactToolActivity {
            fact_id: validate_safe_text("fact_id", format!("{receipt_id}-fact"))?,
            tool_kind: self.tool_kind.clone(),
            artifact_hash: validate_hash("artifact_hash", artifact_hash)?.to_owned(),
            effect_ability: self.effect_ability.clone(),
            effect_resource: self.effect_resource.clone(),
        });
        let mut redaction_markers = vec![REDACTED_PAYLOAD_MARKER];
        if self.status == ProvenanceToolEffectStatus::Truncated {
            redaction_markers.push(TRUNCATED_STATUS);
        }
        let receipt = ProvenanceReceipt::validate(&ProvenanceReceiptDraft {
            id: receipt_id,
            parents: &[],
            status: self.status.as_str(),
            issue_code: None,
            actor_label: self.tool_kind.as_str(),
            session_label: self.service_label.as_str(),
            trace_label: PROVENANCE_TOOL_EFFECT_SCHEMA,
            event_count: self.original_output_bytes.unwrap_or_default(),
            artifact_hashes: &[artifact_hash],
            mutation_kind: None,
            effect_kind: Some(self.effect_ability.as_str()),
            backend_label: Some(self.service_label.as_str()),
            interchange_label: None,
            redaction_markers: redaction_markers.as_slice(),
        })?;
        Ok((fact, receipt))
    }
}
