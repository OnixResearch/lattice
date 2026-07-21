#![allow(unknown_lints)]
#![cfg_attr(animus_octet, allow(tigerstyle::excessive_file_length))]

use super::{
    DESCRIPTOR_ID_FIELD, MAX_MEMORY_CONFIDENCE_PERCENT, MAX_MEMORY_DESCRIPTORS,
    MEMORY_CATEGORY_FIELD, MEMORY_EVIDENCE_FIELD, MEMORY_SOURCE_FIELD, MemoryAdmissionStatus,
    MemoryAvailabilityStatus, MemoryConfidence, MemoryError, MemoryFreshness,
    MemoryInjectionBudget, MemoryReceiptLeak, MemoryRedactionClass, MemoryTrustScope, NonZeroU32,
    String, Vec, memory_leak, normalize, reject_expired_descriptor,
};

const MEMORY_ADAPTER_LABEL_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("memory_adapter_label");
const MEMORY_ADAPTER_OWNER_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("memory_adapter_owner");
const MEMORY_CONFLICT_GROUP_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("memory_conflict_group");
const MEMORY_EXPIRATION_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("memory_expiration_label");
const MEMORY_SUPERSESSION_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("memory_supersession_id");
const MEMORY_QUERY_LABEL_FIELD: crate::validation::FieldName =
    crate::validation::FieldName::new("memory_query_label");
const ZERO_MEMORY_UNITS: u32 = 0;
const RANK_OFFSET: u32 = 1;
const RANK_OFFSET_INDEX: usize = 1;
const MAX_MEMORY_ADAPTERS: usize = 16;
const PERCENT_SCALE: u32 = 100;

/// Swappable generic memory adapter role.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum GenericMemoryAdapterRole {
    /// Imports already-admitted instruction or memory source descriptors.
    SourceImport,
    /// Persists admitted records in a host-owned durable store.
    DurableStore,
    /// Retrieves or searches normalized memory records.
    RetrievalSearch,
    /// Extracts candidate memories from already-admitted context.
    Extraction,
    /// Supplies embedding, ranking, or hybrid-search support.
    EmbeddingSearchSupport,
    /// Reviews candidate memories for admission.
    Admission,
    /// Applies update, delete, tombstone, or supersession requests.
    UpdateDelete,
    /// Supplies instruction-source memory behavior.
    InstructionSource,
}

impl GenericMemoryAdapterRole {
    /// Returns the stable role label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SourceImport => "source-import",
            Self::DurableStore => "durable-store",
            Self::RetrievalSearch => "retrieval-search",
            Self::Extraction => "extraction",
            Self::EmbeddingSearchSupport => "embedding-search-support",
            Self::Admission => "admission",
            Self::UpdateDelete => "update-delete",
            Self::InstructionSource => "instruction-source",
        }
    }
}

/// Capability declared by a generic memory adapter.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum GenericMemoryAdapterCapability {
    /// Adapter can import sources into normalized records.
    SourceImport,
    /// Adapter can durably store admitted records.
    DurableStore,
    /// Adapter can retrieve or search normalized records.
    RetrievalSearch,
    /// Adapter can propose extracted memory candidates.
    Extraction,
    /// Adapter can provide embedding, ranking, or search assistance.
    EmbeddingSearch,
    /// Adapter can review admission decisions.
    AdmissionReview,
    /// Adapter can update or delete existing records.
    UpdateDelete,
    /// Adapter can supply instruction-source memories.
    InstructionSource,
    /// Adapter can emit safe operation receipts.
    SafeReceipts,
}

impl GenericMemoryAdapterCapability {
    /// Returns the stable capability label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SourceImport => "source-import",
            Self::DurableStore => "durable-store",
            Self::RetrievalSearch => "retrieval-search",
            Self::Extraction => "extraction",
            Self::EmbeddingSearch => "embedding-search",
            Self::AdmissionReview => "admission-review",
            Self::UpdateDelete => "update-delete",
            Self::InstructionSource => "instruction-source",
            Self::SafeReceipts => "safe-receipts",
        }
    }
}

/// Borrowed descriptor for a host-injected memory adapter family member.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MemoryAdapterFamilyDescriptorDraft<'a> {
    /// Safe adapter label.
    pub adapter_label: &'a str,
    /// Adapter role.
    pub role: GenericMemoryAdapterRole,
    /// Declared adapter capabilities.
    pub capabilities: &'a [GenericMemoryAdapterCapability],
    /// Adapter availability.
    pub availability: MemoryAvailabilityStatus,
    /// Safe owner label.
    pub owner_label: &'a str,
}

/// Validated descriptor for a host-injected memory adapter family member.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemoryAdapterFamilyDescriptor {
    adapter_label: String,
    role: GenericMemoryAdapterRole,
    capabilities: Vec<GenericMemoryAdapterCapability>,
    availability: MemoryAvailabilityStatus,
    owner_label: String,
}

impl MemoryAdapterFamilyDescriptor {
    /// Validates safe adapter metadata before reusable logic depends on it.
    ///
    /// # Errors
    ///
    /// Returns [`MemoryError`] when safe labels fail validation or capabilities are absent.
    // r[impl agent_harness_runtime.memory_adapters.family]
    pub fn validate(draft: &MemoryAdapterFamilyDescriptorDraft<'_>) -> Result<Self, MemoryError> {
        let adapter_label = normalize(MEMORY_ADAPTER_LABEL_FIELD, draft.adapter_label)?;
        if draft.capabilities.is_empty() {
            return Err(MemoryError::MissingAdapterCapabilities { adapter_label });
        }
        Ok(Self {
            adapter_label,
            role: draft.role,
            capabilities: draft.capabilities.to_vec(),
            availability: draft.availability,
            owner_label: normalize(MEMORY_ADAPTER_OWNER_FIELD, draft.owner_label)?,
        })
    }

    /// Returns the safe adapter label.
    #[must_use]
    pub fn adapter_label(&self) -> &str {
        self.adapter_label.as_str()
    }

    /// Returns the adapter role.
    #[must_use]
    pub const fn role(&self) -> GenericMemoryAdapterRole {
        self.role
    }

    /// Returns declared capabilities.
    #[must_use]
    pub fn capabilities(&self) -> &[GenericMemoryAdapterCapability] {
        self.capabilities.as_slice()
    }

    /// Returns adapter availability.
    #[must_use]
    pub const fn availability(&self) -> MemoryAvailabilityStatus {
        self.availability
    }

    /// Returns the safe owner label.
    #[must_use]
    pub fn owner_label(&self) -> &str {
        self.owner_label.as_str()
    }
}

/// Input for fail-closed memory adapter availability evaluation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MemoryAdapterOperationInput<'a> {
    /// Whether memory use or generation is enabled for this operation.
    pub enabled: bool,
    /// Roles required by the operation.
    pub required_roles: &'a [GenericMemoryAdapterRole],
    /// Validated adapters injected by the host.
    pub adapters: &'a [MemoryAdapterFamilyDescriptor],
}

/// Fail-closed availability outcome for a memory operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MemoryAdapterOperationAvailability {
    /// All required adapters are present, available, and capability-compatible.
    Ready,
    /// Memory is disabled before any adapter or ambient fallback can run.
    Disabled {
        /// Role that was not evaluated because memory is disabled.
        role: Option<GenericMemoryAdapterRole>,
    },
    /// A required adapter was absent or unavailable.
    Unavailable {
        /// Missing or unavailable role.
        role: GenericMemoryAdapterRole,
    },
    /// A required adapter did not declare the capability needed by its role.
    Unsupported {
        /// Unsupported role.
        role: GenericMemoryAdapterRole,
    },
}

/// Evaluates whether a memory operation may dispatch to injected adapters only.
///
/// The function performs no filesystem, database, MCP, network, provider, or vector-index fallback.
///
/// # Panics
///
/// Panics if callers provide more required roles or adapters than the memory adapter bound.
#[must_use]
// r[impl agent_harness_runtime.memory_adapters.family.missing]
pub fn evaluate_memory_adapter_operation(
    input: MemoryAdapterOperationInput<'_>,
) -> MemoryAdapterOperationAvailability {
    assert!(input.required_roles.len() <= MAX_MEMORY_ADAPTERS);
    assert!(input.adapters.len() <= MAX_MEMORY_ADAPTERS);
    if !input.enabled {
        return MemoryAdapterOperationAvailability::Disabled {
            role: input.required_roles.first().copied(),
        };
    }
    for required_role in input.required_roles {
        let Some(adapter) = input
            .adapters
            .iter()
            .find(|adapter| adapter.role() == *required_role)
        else {
            return MemoryAdapterOperationAvailability::Unavailable {
                role: *required_role,
            };
        };
        if adapter.availability() != MemoryAvailabilityStatus::Available {
            return MemoryAdapterOperationAvailability::Unavailable {
                role: *required_role,
            };
        }
        if !adapter
            .capabilities()
            .contains(&role_capability(*required_role))
        {
            return MemoryAdapterOperationAvailability::Unsupported {
                role: *required_role,
            };
        }
    }
    MemoryAdapterOperationAvailability::Ready
}

/// Generic normalized memory record kind.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum GenericMemoryRecordKind {
    /// Instruction-like record.
    Instruction,
    /// Durable fact.
    Fact,
    /// User, project, or host preference.
    Preference,
    /// Summary record.
    Summary,
    /// Pitfall or recurring warning.
    Pitfall,
    /// Extracted candidate awaiting review.
    Candidate,
}

impl GenericMemoryRecordKind {
    /// Returns the stable record kind label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Instruction => "instruction",
            Self::Fact => "fact",
            Self::Preference => "preference",
            Self::Summary => "summary",
            Self::Pitfall => "pitfall",
            Self::Candidate => "candidate",
        }
    }
}

/// Category-aware memory taxonomy label.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum MemoryRecordCategory {
    /// Timestamped or event-like episodic memory.
    Episodic,
    /// Distilled semantic knowledge.
    Semantic,
    /// Reusable procedure or workflow memory.
    Procedural,
    /// Reflective lesson learned.
    Reflection,
    /// Pitfall or recurring warning.
    Pitfall,
    /// Reviewed instruction or guidance.
    Instruction,
    /// Durable fact.
    Fact,
    /// User, project, or host preference.
    Preference,
    /// Compact summary.
    Summary,
    /// Candidate awaiting review.
    Candidate,
    /// Explicitly unsupported memory category.
    Unsupported,
}

impl MemoryRecordCategory {
    /// Returns the stable category label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Episodic => "episodic",
            Self::Semantic => "semantic",
            Self::Procedural => "procedural",
            Self::Reflection => "reflection",
            Self::Pitfall => "pitfall",
            Self::Instruction => "instruction",
            Self::Fact => "fact",
            Self::Preference => "preference",
            Self::Summary => "summary",
            Self::Candidate => "candidate",
            Self::Unsupported => "unsupported",
        }
    }

    #[must_use]
    fn from_label(label: &str) -> Option<Self> {
        match label {
            "episodic" => Some(Self::Episodic),
            "semantic" => Some(Self::Semantic),
            "procedural" => Some(Self::Procedural),
            "reflection" => Some(Self::Reflection),
            "pitfall" => Some(Self::Pitfall),
            "instruction" => Some(Self::Instruction),
            "fact" => Some(Self::Fact),
            "preference" => Some(Self::Preference),
            "summary" => Some(Self::Summary),
            "candidate" => Some(Self::Candidate),
            "unsupported" => Some(Self::Unsupported),
            _ => None,
        }
    }
}

/// Maps existing generic record kinds into the category-aware taxonomy.
#[must_use]
// r[impl agent_harness_runtime.memory_taxonomy.compatibility]
pub const fn memory_category_for_generic_kind(
    kind: GenericMemoryRecordKind,
) -> MemoryRecordCategory {
    match kind {
        GenericMemoryRecordKind::Instruction => MemoryRecordCategory::Instruction,
        GenericMemoryRecordKind::Fact => MemoryRecordCategory::Fact,
        GenericMemoryRecordKind::Preference => MemoryRecordCategory::Preference,
        GenericMemoryRecordKind::Summary => MemoryRecordCategory::Summary,
        GenericMemoryRecordKind::Pitfall => MemoryRecordCategory::Pitfall,
        GenericMemoryRecordKind::Candidate => MemoryRecordCategory::Candidate,
    }
}

/// Borrowed normalized record-envelope metadata before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MemoryRecordEnvelopeDraft<'a> {
    /// Stable record identity.
    pub id: &'a str,
    /// Generic record kind.
    pub kind: GenericMemoryRecordKind,
    /// Optional explicit cognitive taxonomy category label.
    pub category_label: Option<&'a str>,
    /// Whether callers require an explicit category instead of compatibility mapping.
    pub require_explicit_category: bool,
    /// Memory scope; missing scope is rejected.
    pub scope: Option<MemoryTrustScope>,
    /// Safe source label.
    pub source_label: &'a str,
    /// Evidence handle; missing evidence is rejected.
    pub evidence_handle: Option<&'a str>,
    /// Host admission status.
    pub admission_status: MemoryAdmissionStatus,
    /// Freshness state.
    pub freshness: MemoryFreshness,
    /// Bounded confidence percentage.
    pub confidence_percent: u8,
    /// Sensitivity or redaction class.
    pub redaction_class: MemoryRedactionClass,
    /// Lowercase BLAKE3 hash for the hidden body or safe summary source.
    pub body_hash: &'a str,
    /// Prompt budget units requested by this record.
    pub budget_units: u32,
    /// Optional safe expiration label.
    pub expiration_label: Option<&'a str>,
    /// Optional record superseded by this record.
    pub supersedes_id: Option<&'a str>,
    /// Optional newer record that supersedes this record.
    pub superseded_by_id: Option<&'a str>,
    /// Optional conflict group label.
    pub conflict_group: Option<&'a str>,
    /// Raw body marker, forbidden at this boundary.
    pub raw_body_marker: Option<&'a str>,
    /// Hidden body marker, forbidden at this boundary.
    pub hidden_body_marker: Option<&'a str>,
}

/// Validated normalized memory record envelope.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemoryRecordEnvelope {
    id: String,
    kind: GenericMemoryRecordKind,
    category: MemoryRecordCategory,
    scope: MemoryTrustScope,
    source_label: String,
    evidence_handle: String,
    admission_status: MemoryAdmissionStatus,
    freshness: MemoryFreshness,
    confidence: MemoryConfidence,
    redaction_class: MemoryRedactionClass,
    body_hash: crate::artifact::Hash,
    budget_units: NonZeroU32,
    expiration_label: Option<String>,
    supersedes_id: Option<String>,
    superseded_by_id: Option<String>,
    conflict_group: Option<String>,
}

impl MemoryRecordEnvelope {
    /// Validates a backend-normalized memory record before planning or prompt influence.
    ///
    /// # Errors
    ///
    /// Returns [`MemoryError`] when required metadata is missing, malformed, unsafe, expired, or unbudgeted.
    // r[impl agent_harness_runtime.memory_adapters.record_envelope]
    // r[impl agent_harness_runtime.memory_taxonomy.record_categories]
    pub fn validate(draft: &MemoryRecordEnvelopeDraft<'_>) -> Result<Self, MemoryError> {
        let id = normalize(DESCRIPTOR_ID_FIELD, draft.id)?;
        reject_record_leaks(draft)?;
        let category = memory_record_category(
            id.as_str(),
            draft.kind,
            draft.category_label,
            draft.require_explicit_category,
        )?;
        let scope = draft.scope.ok_or_else(|| MemoryError::MissingRecordScope {
            record_id: id.clone(),
        })?;
        let evidence_handle = draft
            .evidence_handle
            .ok_or_else(|| MemoryError::MissingEvidence {
                record_id: id.clone(),
            })
            .and_then(|value| normalize(MEMORY_EVIDENCE_FIELD, value))?;
        reject_expired_descriptor(&id, draft.freshness)?;
        let confidence = MemoryConfidence::new(draft.confidence_percent)?;
        let body_hash =
            crate::artifact::Hash::new(draft.body_hash).map_err(MemoryError::InvalidHash)?;
        let budget_units =
            NonZeroU32::new(draft.budget_units).ok_or_else(|| MemoryError::ZeroBudgetUnits {
                descriptor_id: id.clone(),
            })?;
        Ok(Self {
            id,
            kind: draft.kind,
            category,
            scope,
            source_label: normalize(MEMORY_SOURCE_FIELD, draft.source_label)?,
            evidence_handle,
            admission_status: draft.admission_status,
            freshness: draft.freshness,
            confidence,
            redaction_class: draft.redaction_class,
            body_hash,
            budget_units,
            expiration_label: normalize_optional(MEMORY_EXPIRATION_FIELD, draft.expiration_label)?,
            supersedes_id: normalize_optional(MEMORY_SUPERSESSION_FIELD, draft.supersedes_id)?,
            superseded_by_id: normalize_optional(
                MEMORY_SUPERSESSION_FIELD,
                draft.superseded_by_id,
            )?,
            conflict_group: normalize_optional(MEMORY_CONFLICT_GROUP_FIELD, draft.conflict_group)?,
        })
    }

    /// Returns record identity.
    #[must_use]
    pub fn id(&self) -> &str {
        self.id.as_str()
    }

    /// Returns record kind.
    #[must_use]
    pub const fn kind(&self) -> GenericMemoryRecordKind {
        self.kind
    }

    /// Returns category-aware memory taxonomy label.
    #[must_use]
    pub const fn category(&self) -> MemoryRecordCategory {
        self.category
    }

    /// Returns memory scope.
    #[must_use]
    pub const fn scope(&self) -> MemoryTrustScope {
        self.scope
    }

    /// Returns safe source label.
    #[must_use]
    pub fn source_label(&self) -> &str {
        self.source_label.as_str()
    }

    /// Returns safe evidence handle.
    #[must_use]
    pub fn evidence_handle(&self) -> &str {
        self.evidence_handle.as_str()
    }

    /// Returns admission status.
    #[must_use]
    pub const fn admission_status(&self) -> MemoryAdmissionStatus {
        self.admission_status
    }

    /// Returns freshness state.
    #[must_use]
    pub const fn freshness(&self) -> MemoryFreshness {
        self.freshness
    }

    /// Returns confidence score.
    #[must_use]
    pub const fn confidence(&self) -> MemoryConfidence {
        self.confidence
    }

    /// Returns redaction class.
    #[must_use]
    pub const fn redaction_class(&self) -> MemoryRedactionClass {
        self.redaction_class
    }

    /// Returns BLAKE3 body hash.
    #[must_use]
    pub const fn body_hash(&self) -> &crate::artifact::Hash {
        &self.body_hash
    }

    /// Returns prompt budget units.
    #[must_use]
    pub const fn budget_units(&self) -> NonZeroU32 {
        self.budget_units
    }

    /// Returns optional safe expiration label.
    #[must_use]
    pub fn expiration_label(&self) -> Option<&str> {
        self.expiration_label.as_deref()
    }

    /// Returns optional superseded record identity.
    #[must_use]
    pub fn supersedes_id(&self) -> Option<&str> {
        self.supersedes_id.as_deref()
    }

    /// Returns optional newer superseding record identity.
    #[must_use]
    pub fn superseded_by_id(&self) -> Option<&str> {
        self.superseded_by_id.as_deref()
    }

    /// Returns optional conflict group.
    #[must_use]
    pub fn conflict_group(&self) -> Option<&str> {
        self.conflict_group.as_deref()
    }
}

/// Adapter-declared safe match reason for memory retrieval.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum MemoryMatchReason {
    /// Exact scope or identity match.
    ExactScope,
    /// Keyword or lexical match supplied by an adapter.
    Keyword,
    /// Semantic match supplied by an embedding or hybrid adapter.
    Semantic,
    /// Recent interaction match.
    Recent,
    /// Host or user pin.
    ManualPin,
}

impl MemoryMatchReason {
    /// Returns the stable match reason label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ExactScope => "exact-scope",
            Self::Keyword => "keyword",
            Self::Semantic => "semantic",
            Self::Recent => "recent",
            Self::ManualPin => "manual-pin",
        }
    }
}

/// Memory routing operation intent.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum MemoryRouteIntent {
    /// Read or search existing records.
    ReadSearch,
    /// Import a source into a normalized record stream.
    SourceImport,
    /// Extract candidate memories from admitted context.
    Extraction,
    /// Persist admitted memory records.
    DurableStore,
    /// Update, forget, or tombstone memory records.
    UpdateDelete,
    /// Prepare admitted memory for prompt influence.
    PromptInfluence,
}

impl MemoryRouteIntent {
    /// Returns the stable route-intent label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ReadSearch => "read-search",
            Self::SourceImport => "source-import",
            Self::Extraction => "extraction",
            Self::DurableStore => "durable-store",
            Self::UpdateDelete => "update-delete",
            Self::PromptInfluence => "prompt-influence",
        }
    }
}

/// Borrowed memory route candidate before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MemoryRouteCandidateDraft<'a> {
    /// Stable route identity.
    pub route_id: &'a str,
    /// Safe adapter label.
    pub adapter_label: &'a str,
    /// Route operation intent.
    pub intent: MemoryRouteIntent,
    /// Category hint for the route.
    pub category: MemoryRecordCategory,
    /// Scope the route can serve.
    pub scope: MemoryTrustScope,
    /// Required adapter role.
    pub required_role: GenericMemoryAdapterRole,
    /// Declared adapter availability.
    pub availability: MemoryAvailabilityStatus,
    /// Whether the adapter declares the required capability.
    pub capability_supported: bool,
    /// Whether active policy admits this route.
    pub policy_admitted: bool,
    /// Whether stale route records are allowed.
    pub freshness: MemoryFreshness,
    /// Bounded classifier or policy confidence.
    pub confidence_percent: u8,
    /// Prompt or operation budget units requested by this route.
    pub budget_units: u32,
    /// Raw classifier output, forbidden at this boundary.
    pub raw_classifier_output: Option<&'a str>,
    /// Raw memory body, forbidden at this boundary.
    pub raw_memory_body: Option<&'a str>,
    /// Credential marker, forbidden at this boundary.
    pub credential: Option<&'a str>,
    /// Provider payload marker, forbidden at this boundary.
    pub provider_payload: Option<&'a str>,
    /// Full host path marker, forbidden at this boundary.
    pub full_path: Option<&'a str>,
}

/// Validated memory route candidate.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemoryRouteCandidate {
    route_id: String,
    adapter_label: String,
    intent: MemoryRouteIntent,
    category: MemoryRecordCategory,
    scope: MemoryTrustScope,
    required_role: GenericMemoryAdapterRole,
    availability: MemoryAvailabilityStatus,
    capability_supported: bool,
    policy_admitted: bool,
    freshness: MemoryFreshness,
    confidence: MemoryConfidence,
    budget_units: NonZeroU32,
}

impl MemoryRouteCandidate {
    /// Validates safe route-candidate metadata.
    ///
    /// # Errors
    ///
    /// Returns [`MemoryError`] when safe labels, confidence, budget, or leak boundaries fail.
    // r[impl agent_harness_runtime.memory_routing.route_plan]
    // r[impl agent_harness_runtime.memory_routing.fail_closed]
    pub fn validate(draft: &MemoryRouteCandidateDraft<'_>) -> Result<Self, MemoryError> {
        reject_route_candidate_leaks(draft)?;
        let route_id = normalize(DESCRIPTOR_ID_FIELD, draft.route_id)?;
        let budget_units =
            NonZeroU32::new(draft.budget_units).ok_or_else(|| MemoryError::ZeroBudgetUnits {
                descriptor_id: route_id.clone(),
            })?;
        Ok(Self {
            route_id,
            adapter_label: normalize(MEMORY_ADAPTER_LABEL_FIELD, draft.adapter_label)?,
            intent: draft.intent,
            category: draft.category,
            scope: draft.scope,
            required_role: draft.required_role,
            availability: draft.availability,
            capability_supported: draft.capability_supported,
            policy_admitted: draft.policy_admitted,
            freshness: draft.freshness,
            confidence: MemoryConfidence::new(draft.confidence_percent)?,
            budget_units,
        })
    }

    /// Returns stable route identity.
    #[must_use]
    pub fn route_id(&self) -> &str {
        self.route_id.as_str()
    }

    /// Returns safe adapter label.
    #[must_use]
    pub fn adapter_label(&self) -> &str {
        self.adapter_label.as_str()
    }

    /// Returns route intent.
    #[must_use]
    pub const fn intent(&self) -> MemoryRouteIntent {
        self.intent
    }

    /// Returns category hint.
    #[must_use]
    pub const fn category(&self) -> MemoryRecordCategory {
        self.category
    }

    /// Returns route scope.
    #[must_use]
    pub const fn scope(&self) -> MemoryTrustScope {
        self.scope
    }

    /// Returns required adapter role.
    #[must_use]
    pub const fn required_role(&self) -> GenericMemoryAdapterRole {
        self.required_role
    }

    /// Returns route confidence.
    #[must_use]
    pub const fn confidence(&self) -> MemoryConfidence {
        self.confidence
    }

    /// Returns requested route budget units.
    #[must_use]
    pub const fn budget_units(&self) -> NonZeroU32 {
        self.budget_units
    }
}

/// Memory route-planning policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MemoryRoutePolicy<'a> {
    /// Safe query or operation label.
    pub route_label: &'a str,
    /// BLAKE3 policy hash.
    pub policy_hash: &'a str,
    /// Scopes admitted by policy.
    pub allowed_scopes: &'a [MemoryTrustScope],
    /// Whether stale route candidates can be selected.
    pub include_stale: bool,
    /// Route budget.
    pub budget: MemoryInjectionBudget,
}

/// Input for pure memory route planning.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MemoryRoutePlanningInput<'a> {
    /// Route policy.
    pub policy: MemoryRoutePolicy<'a>,
    /// Validated route candidates.
    pub candidates: &'a [MemoryRouteCandidate],
}

/// Outcome status for a memory route candidate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MemoryRouteOutcomeStatus {
    /// Route is selected.
    Selected,
    /// Policy denied the route.
    Denied,
    /// Route scope is not admitted.
    ScopeDenied,
    /// Route is stale and stale routes are disabled.
    Stale,
    /// Required adapter is disabled or unavailable.
    Unavailable,
    /// Required adapter role or capability is unsupported.
    Unsupported,
    /// Route exceeds remaining budget.
    OverBudget,
}

/// Safe route outcome.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemoryRouteOutcome {
    route_id: String,
    adapter_label: String,
    intent: MemoryRouteIntent,
    category: MemoryRecordCategory,
    required_role: GenericMemoryAdapterRole,
    status: MemoryRouteOutcomeStatus,
    confidence: MemoryConfidence,
    rank: u32,
    requested_units: NonZeroU32,
    consumed_units: u32,
}

impl MemoryRouteOutcome {
    /// Returns stable route identity.
    #[must_use]
    pub fn route_id(&self) -> &str {
        self.route_id.as_str()
    }

    /// Returns safe adapter label.
    #[must_use]
    pub fn adapter_label(&self) -> &str {
        self.adapter_label.as_str()
    }

    /// Returns route intent.
    #[must_use]
    pub const fn intent(&self) -> MemoryRouteIntent {
        self.intent
    }

    /// Returns category hint.
    #[must_use]
    pub const fn category(&self) -> MemoryRecordCategory {
        self.category
    }

    /// Returns required adapter role.
    #[must_use]
    pub const fn required_role(&self) -> GenericMemoryAdapterRole {
        self.required_role
    }

    /// Returns outcome status.
    #[must_use]
    pub const fn status(&self) -> MemoryRouteOutcomeStatus {
        self.status
    }

    /// Returns confidence.
    #[must_use]
    pub const fn confidence(&self) -> MemoryConfidence {
        self.confidence
    }

    /// Returns deterministic route rank.
    #[must_use]
    pub const fn rank(&self) -> u32 {
        self.rank
    }

    /// Returns requested units.
    #[must_use]
    pub const fn requested_units(&self) -> NonZeroU32 {
        self.requested_units
    }

    /// Returns consumed units.
    #[must_use]
    pub const fn consumed_units(&self) -> u32 {
        self.consumed_units
    }
}

/// Pure memory route plan.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemoryRoutePlan {
    outcomes: Vec<MemoryRouteOutcome>,
    selected_count: u32,
    consumed_units: u32,
    policy_hash: crate::artifact::Hash,
}

impl MemoryRoutePlan {
    /// Returns route outcomes in deterministic order.
    #[must_use]
    pub fn outcomes(&self) -> &[MemoryRouteOutcome] {
        self.outcomes.as_slice()
    }

    /// Returns selected route count.
    #[must_use]
    pub const fn selected_count(&self) -> u32 {
        self.selected_count
    }

    /// Returns consumed route budget units.
    #[must_use]
    pub const fn consumed_units(&self) -> u32 {
        self.consumed_units
    }

    /// Returns route policy hash.
    #[must_use]
    pub const fn policy_hash(&self) -> &crate::artifact::Hash {
        &self.policy_hash
    }
}

/// Borrowed safe route receipt metadata before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MemoryRouteReceiptDraft<'a> {
    /// Stable route plan label.
    pub route_label: &'a str,
    /// BLAKE3 policy hash.
    pub policy_hash: &'a str,
    /// Selected route count.
    pub selected_count: u32,
    /// Skipped route count.
    pub skipped_count: u32,
    /// Raw classifier output, forbidden at this boundary.
    pub raw_classifier_output: Option<&'a str>,
    /// Raw memory body, forbidden at this boundary.
    pub raw_memory_body: Option<&'a str>,
    /// Credential marker, forbidden at this boundary.
    pub credential: Option<&'a str>,
    /// Provider payload marker, forbidden at this boundary.
    pub provider_payload: Option<&'a str>,
    /// Full host path marker, forbidden at this boundary.
    pub full_path: Option<&'a str>,
}

/// Safe memory route receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemoryRouteReceipt {
    route_label: String,
    policy_hash: crate::artifact::Hash,
    selected_count: u32,
    skipped_count: u32,
}

impl MemoryRouteReceipt {
    /// Validates safe route receipt metadata.
    ///
    /// # Errors
    ///
    /// Returns [`MemoryError`] when labels, hashes, or leak boundaries fail.
    // r[impl agent_harness_runtime.memory_routing.route_plan]
    pub fn validate(draft: &MemoryRouteReceiptDraft<'_>) -> Result<Self, MemoryError> {
        reject_route_receipt_leaks(draft)?;
        Ok(Self {
            route_label: normalize(MEMORY_QUERY_LABEL_FIELD, draft.route_label)?,
            policy_hash: crate::artifact::Hash::new(draft.policy_hash)
                .map_err(MemoryError::InvalidHash)?,
            selected_count: draft.selected_count,
            skipped_count: draft.skipped_count,
        })
    }

    /// Returns safe route label.
    #[must_use]
    pub fn route_label(&self) -> &str {
        self.route_label.as_str()
    }

    /// Returns policy hash.
    #[must_use]
    pub const fn policy_hash(&self) -> &crate::artifact::Hash {
        &self.policy_hash
    }

    /// Returns selected route count.
    #[must_use]
    pub const fn selected_count(&self) -> u32 {
        self.selected_count
    }

    /// Returns skipped route count.
    #[must_use]
    pub const fn skipped_count(&self) -> u32 {
        self.skipped_count
    }
}

/// Plans memory routes without dispatching adapters, providers, stores, files, clocks, or networks.
///
/// # Errors
///
/// Returns [`MemoryError`] when labels, policy hashes, candidate counts, or route bounds fail.
///
/// # Panics
///
/// Panics only if internal route bounds or validated budget invariants are violated.
// r[impl agent_harness_runtime.memory_routing.pure_planner]
pub fn plan_memory_routes(
    input: MemoryRoutePlanningInput<'_>,
) -> Result<MemoryRoutePlan, MemoryError> {
    normalize(MEMORY_QUERY_LABEL_FIELD, input.policy.route_label)?;
    let policy_hash =
        crate::artifact::Hash::new(input.policy.policy_hash).map_err(MemoryError::InvalidHash)?;
    assert!(input.policy.budget.units().get() > ZERO_MEMORY_UNITS);
    if input.candidates.len() > MAX_MEMORY_DESCRIPTORS {
        return Err(MemoryError::TooManyItems {
            field: DESCRIPTOR_ID_FIELD.as_str(),
            max_items: MAX_MEMORY_DESCRIPTORS,
            actual_items: input.candidates.len(),
        });
    }
    assert!(input.candidates.len() <= MAX_MEMORY_DESCRIPTORS);
    let mut ordered: Vec<&MemoryRouteCandidate> = Vec::with_capacity(input.candidates.len());
    for candidate in input.candidates {
        ordered.push(candidate);
    }
    ordered.sort_by(|left, right| compare_memory_route_candidates(left, right));
    let mut outcomes = Vec::with_capacity(ordered.len());
    let mut selected_count = u32::default();
    let mut consumed_units = ZERO_MEMORY_UNITS;
    for candidate in ordered {
        let decision = memory_route_decision(candidate, &input.policy, consumed_units);
        if decision.status == MemoryRouteOutcomeStatus::Selected {
            selected_count = selected_count.saturating_add(RANK_OFFSET);
            consumed_units = consumed_units.saturating_add(decision.consumed_units);
        }
        outcomes.push(memory_route_outcome(MemoryRouteOutcomeInput {
            candidate,
            status: decision.status,
            rank: memory_rank_from_len(outcomes.len())?,
            consumed_units: decision.consumed_units,
        }));
    }
    Ok(MemoryRoutePlan {
        outcomes,
        selected_count,
        consumed_units,
        policy_hash,
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct MemoryRouteDecision {
    status: MemoryRouteOutcomeStatus,
    consumed_units: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct MemoryRouteOutcomeInput<'a> {
    candidate: &'a MemoryRouteCandidate,
    status: MemoryRouteOutcomeStatus,
    rank: u32,
    consumed_units: u32,
}

fn memory_route_decision(
    candidate: &MemoryRouteCandidate,
    policy: &MemoryRoutePolicy<'_>,
    consumed_units: u32,
) -> MemoryRouteDecision {
    assert!(candidate.budget_units.get() > ZERO_MEMORY_UNITS);
    assert!(policy.budget.units().get() > ZERO_MEMORY_UNITS);
    if !candidate.policy_admitted {
        return skipped_route(MemoryRouteOutcomeStatus::Denied);
    }
    if !scope_allowed(candidate.scope, policy.allowed_scopes) {
        return skipped_route(MemoryRouteOutcomeStatus::ScopeDenied);
    }
    if candidate.freshness == MemoryFreshness::Stale && !policy.include_stale {
        return skipped_route(MemoryRouteOutcomeStatus::Stale);
    }
    if candidate.availability != MemoryAvailabilityStatus::Available {
        return skipped_route(MemoryRouteOutcomeStatus::Unavailable);
    }
    if !candidate.capability_supported {
        return skipped_route(MemoryRouteOutcomeStatus::Unsupported);
    }
    let remaining_units = policy.budget.units().get().saturating_sub(consumed_units);
    let requested_units = candidate.budget_units.get();
    if requested_units > remaining_units {
        return skipped_route(MemoryRouteOutcomeStatus::OverBudget);
    }
    MemoryRouteDecision {
        status: MemoryRouteOutcomeStatus::Selected,
        consumed_units: requested_units,
    }
}

fn memory_route_outcome(input: MemoryRouteOutcomeInput<'_>) -> MemoryRouteOutcome {
    MemoryRouteOutcome {
        route_id: input.candidate.route_id.clone(),
        adapter_label: input.candidate.adapter_label.clone(),
        intent: input.candidate.intent,
        category: input.candidate.category,
        required_role: input.candidate.required_role,
        status: input.status,
        confidence: input.candidate.confidence,
        rank: input.rank,
        requested_units: input.candidate.budget_units,
        consumed_units: input.consumed_units,
    }
}

fn skipped_route(status: MemoryRouteOutcomeStatus) -> MemoryRouteDecision {
    MemoryRouteDecision {
        status,
        consumed_units: ZERO_MEMORY_UNITS,
    }
}

fn compare_memory_route_candidates(
    left: &MemoryRouteCandidate,
    right: &MemoryRouteCandidate,
) -> core::cmp::Ordering {
    right
        .confidence()
        .cmp(&left.confidence())
        .then(left.category().cmp(&right.category()))
        .then(left.intent().cmp(&right.intent()))
        .then(left.route_id().cmp(right.route_id()))
}

/// Borrowed memory evaluation fixture before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MemoryEvaluationFixtureDraft<'a> {
    /// Stable fixture identity.
    pub fixture_id: &'a str,
    /// Safe query label.
    pub query_label: &'a str,
    /// Expected relevant record identities.
    pub expected_record_ids: &'a [&'a str],
    /// Scope expected by the fixture.
    pub scope: MemoryTrustScope,
    /// Category expected by the fixture.
    pub category: MemoryRecordCategory,
    /// Whether the answer should abstain.
    pub abstention_expected: bool,
    /// Safe answer-use expectation label.
    pub answer_use_label: &'a str,
    /// BLAKE3 evidence hash.
    pub evidence_hash: &'a str,
    /// Raw prompt text, forbidden at this boundary.
    pub raw_prompt_text: Option<&'a str>,
    /// Raw memory body, forbidden at this boundary.
    pub raw_memory_body: Option<&'a str>,
    /// Credential marker, forbidden at this boundary.
    pub credential: Option<&'a str>,
    /// Provider payload marker, forbidden at this boundary.
    pub provider_payload: Option<&'a str>,
    /// Full host path marker, forbidden at this boundary.
    pub full_path: Option<&'a str>,
}

/// Safe memory evaluation fixture.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemoryEvaluationFixture {
    fixture_id: String,
    query_label: String,
    expected_record_ids: Vec<String>,
    scope: MemoryTrustScope,
    category: MemoryRecordCategory,
    abstention_expected: bool,
    answer_use_label: String,
    evidence_hash: crate::artifact::Hash,
}

impl MemoryEvaluationFixture {
    /// Validates a memory evaluation fixture.
    ///
    /// # Errors
    ///
    /// Returns [`MemoryError`] when safe labels, IDs, hashes, bounds, or leak checks fail.
    // r[impl agent_harness_runtime.memory_evaluation.fixtures]
    pub fn validate(draft: &MemoryEvaluationFixtureDraft<'_>) -> Result<Self, MemoryError> {
        reject_evaluation_fixture_leaks(draft)?;
        let fixture_id = normalize(DESCRIPTOR_ID_FIELD, draft.fixture_id)?;
        if draft.expected_record_ids.is_empty() {
            return Err(MemoryError::MissingEvidence {
                record_id: fixture_id,
            });
        }
        if draft.expected_record_ids.len() > MAX_MEMORY_DESCRIPTORS {
            return Err(MemoryError::TooManyItems {
                field: DESCRIPTOR_ID_FIELD.as_str(),
                max_items: MAX_MEMORY_DESCRIPTORS,
                actual_items: draft.expected_record_ids.len(),
            });
        }
        Ok(Self {
            fixture_id,
            query_label: normalize(MEMORY_QUERY_LABEL_FIELD, draft.query_label)?,
            expected_record_ids: normalize_ids(draft.expected_record_ids)?,
            scope: draft.scope,
            category: draft.category,
            abstention_expected: draft.abstention_expected,
            answer_use_label: normalize(MEMORY_EVIDENCE_FIELD, draft.answer_use_label)?,
            evidence_hash: crate::artifact::Hash::new(draft.evidence_hash)
                .map_err(MemoryError::InvalidHash)?,
        })
    }

    /// Returns fixture identity.
    #[must_use]
    pub fn fixture_id(&self) -> &str {
        self.fixture_id.as_str()
    }

    /// Returns query label.
    #[must_use]
    pub fn query_label(&self) -> &str {
        self.query_label.as_str()
    }

    /// Returns expected safe record identities.
    #[must_use]
    pub fn expected_record_ids(&self) -> &[String] {
        self.expected_record_ids.as_slice()
    }

    /// Returns expected scope.
    #[must_use]
    pub const fn scope(&self) -> MemoryTrustScope {
        self.scope
    }

    /// Returns expected category.
    #[must_use]
    pub const fn category(&self) -> MemoryRecordCategory {
        self.category
    }

    /// Returns whether abstention is expected.
    #[must_use]
    pub const fn abstention_expected(&self) -> bool {
        self.abstention_expected
    }

    /// Returns safe answer-use label.
    #[must_use]
    pub fn answer_use_label(&self) -> &str {
        self.answer_use_label.as_str()
    }

    /// Returns evidence hash.
    #[must_use]
    pub const fn evidence_hash(&self) -> &crate::artifact::Hash {
        &self.evidence_hash
    }
}

/// Observed safe record outcome for evaluation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MemoryEvaluationObservedRecord<'a> {
    /// Safe record identity.
    pub record_id: &'a str,
    /// Category observed for the record.
    pub category: MemoryRecordCategory,
    /// Freshness observed for the record.
    pub freshness: MemoryFreshness,
    /// Whether selected answer used this memory.
    pub answer_used: bool,
    /// Whether the record is contradictory for the fixture.
    pub contradictory: bool,
}

/// Input for pure memory metric reduction.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MemoryEvaluationInput<'a> {
    /// Safe fixture.
    pub fixture: &'a MemoryEvaluationFixture,
    /// Observed retrieval or injection ledger records.
    pub observed: &'a [MemoryEvaluationObservedRecord<'a>],
}

/// Pure memory metric report.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemoryEvaluationMetrics {
    observed_count: u32,
    expected_count: u32,
    relevant_hit_count: u32,
    stale_hit_count: u32,
    contradiction_hit_count: u32,
    answer_used_count: u32,
    precision_percent: u32,
    recall_percent: u32,
    abstention_correct: bool,
}

impl MemoryEvaluationMetrics {
    /// Returns observed item count.
    #[must_use]
    pub const fn observed_count(&self) -> u32 {
        self.observed_count
    }

    /// Returns expected item count.
    #[must_use]
    pub const fn expected_count(&self) -> u32 {
        self.expected_count
    }

    /// Returns relevant hit count.
    #[must_use]
    pub const fn relevant_hit_count(&self) -> u32 {
        self.relevant_hit_count
    }

    /// Returns stale hit count.
    #[must_use]
    pub const fn stale_hit_count(&self) -> u32 {
        self.stale_hit_count
    }

    /// Returns contradiction hit count.
    #[must_use]
    pub const fn contradiction_hit_count(&self) -> u32 {
        self.contradiction_hit_count
    }

    /// Returns answer-used count.
    #[must_use]
    pub const fn answer_used_count(&self) -> u32 {
        self.answer_used_count
    }

    /// Returns integer Precision@K percentage.
    #[must_use]
    pub const fn precision_percent(&self) -> u32 {
        self.precision_percent
    }

    /// Returns integer Recall@K percentage.
    #[must_use]
    pub const fn recall_percent(&self) -> u32 {
        self.recall_percent
    }

    /// Returns whether abstention expectation matched observed answer use.
    #[must_use]
    pub const fn abstention_correct(&self) -> bool {
        self.abstention_correct
    }
}

/// Baseline descriptor for memory-quality claims.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemoryEvaluationBaseline {
    baseline_label: String,
    policy_hash: crate::artifact::Hash,
    evidence_hash: crate::artifact::Hash,
    no_memory: bool,
}

/// Borrowed baseline descriptor before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MemoryEvaluationBaselineDraft<'a> {
    /// Safe baseline label.
    pub baseline_label: &'a str,
    /// BLAKE3 policy hash.
    pub policy_hash: &'a str,
    /// BLAKE3 evidence hash.
    pub evidence_hash: &'a str,
    /// Whether this is a no-memory baseline.
    pub no_memory: bool,
}

impl MemoryEvaluationBaseline {
    /// Validates baseline evidence for memory-quality claims.
    ///
    /// # Errors
    ///
    /// Returns [`MemoryError`] when labels or hashes fail validation.
    // r[impl agent_harness_runtime.memory_evaluation.baselines]
    pub fn validate(draft: &MemoryEvaluationBaselineDraft<'_>) -> Result<Self, MemoryError> {
        Ok(Self {
            baseline_label: normalize(MEMORY_QUERY_LABEL_FIELD, draft.baseline_label)?,
            policy_hash: crate::artifact::Hash::new(draft.policy_hash)
                .map_err(MemoryError::InvalidHash)?,
            evidence_hash: crate::artifact::Hash::new(draft.evidence_hash)
                .map_err(MemoryError::InvalidHash)?,
            no_memory: draft.no_memory,
        })
    }

    /// Returns baseline label.
    #[must_use]
    pub fn baseline_label(&self) -> &str {
        self.baseline_label.as_str()
    }

    /// Returns policy hash.
    #[must_use]
    pub const fn policy_hash(&self) -> &crate::artifact::Hash {
        &self.policy_hash
    }

    /// Returns evidence hash.
    #[must_use]
    pub const fn evidence_hash(&self) -> &crate::artifact::Hash {
        &self.evidence_hash
    }

    /// Returns whether this baseline includes memory.
    #[must_use]
    pub const fn has_memory(&self) -> bool {
        !self.no_memory
    }
}

/// Borrowed memory-quality claim before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MemoryQualityClaimDraft<'a> {
    /// Safe claim label.
    pub claim_label: &'a str,
    /// Required baseline.
    pub baseline: Option<&'a MemoryEvaluationBaseline>,
    /// Current metrics.
    pub current_metrics: &'a MemoryEvaluationMetrics,
}

/// Validated memory-quality claim.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemoryQualityClaim {
    claim_label: String,
    baseline_label: String,
    precision_delta_percent: i32,
    recall_delta_percent: i32,
}

impl MemoryQualityClaim {
    /// Validates a baseline-backed quality claim.
    ///
    /// # Errors
    ///
    /// Returns [`MemoryError`] when the claim has no baseline or an unsafe label.
    pub fn validate(draft: &MemoryQualityClaimDraft<'_>) -> Result<Self, MemoryError> {
        let claim_label = normalize(MEMORY_QUERY_LABEL_FIELD, draft.claim_label)?;
        let baseline = draft.baseline.ok_or_else(|| MemoryError::MissingBaseline {
            claim_id: claim_label.clone(),
        })?;
        Ok(Self {
            claim_label,
            baseline_label: String::from(baseline.baseline_label()),
            precision_delta_percent: i32::default(),
            recall_delta_percent: i32::default(),
        })
    }

    /// Returns claim label.
    #[must_use]
    pub fn claim_label(&self) -> &str {
        self.claim_label.as_str()
    }

    /// Returns baseline label.
    #[must_use]
    pub fn baseline_label(&self) -> &str {
        self.baseline_label.as_str()
    }

    /// Returns precision delta percentage.
    #[must_use]
    pub const fn precision_delta_percent(&self) -> i32 {
        self.precision_delta_percent
    }

    /// Returns recall delta percentage.
    #[must_use]
    pub const fn recall_delta_percent(&self) -> i32 {
        self.recall_delta_percent
    }
}

/// Borrowed external benchmark import descriptor before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MemoryExternalBenchmarkImportDraft<'a> {
    /// Safe benchmark family label.
    pub benchmark_family: &'a str,
    /// Safe run label.
    pub run_label: &'a str,
    /// Safe metric label.
    pub metric_label: &'a str,
    /// Count of imported cases.
    pub case_count: u32,
    /// BLAKE3 policy hash.
    pub policy_hash: &'a str,
    /// BLAKE3 evidence hash.
    pub evidence_hash: &'a str,
    /// Raw conversation marker, forbidden at this boundary.
    pub raw_conversation: Option<&'a str>,
    /// Raw judge prompt marker, forbidden at this boundary.
    pub raw_judge_prompt: Option<&'a str>,
    /// Provider payload marker, forbidden at this boundary.
    pub provider_payload: Option<&'a str>,
    /// Credential marker, forbidden at this boundary.
    pub credential: Option<&'a str>,
}

/// Safe external benchmark import descriptor.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemoryExternalBenchmarkImport {
    benchmark_family: String,
    run_label: String,
    metric_label: String,
    case_count: u32,
    policy_hash: crate::artifact::Hash,
    evidence_hash: crate::artifact::Hash,
}

impl MemoryExternalBenchmarkImport {
    /// Validates a host-owned benchmark import descriptor.
    ///
    /// # Errors
    ///
    /// Returns [`MemoryError`] when safe labels, hashes, counts, or leak checks fail.
    // r[impl agent_harness_runtime.memory_evaluation.external_import]
    pub fn validate(draft: &MemoryExternalBenchmarkImportDraft<'_>) -> Result<Self, MemoryError> {
        reject_external_import_leaks(draft)?;
        let run_label = normalize(MEMORY_QUERY_LABEL_FIELD, draft.run_label)?;
        let case_count =
            NonZeroU32::new(draft.case_count).ok_or_else(|| MemoryError::ZeroBudgetUnits {
                descriptor_id: run_label.clone(),
            })?;
        Ok(Self {
            benchmark_family: normalize(MEMORY_ADAPTER_LABEL_FIELD, draft.benchmark_family)?,
            run_label,
            metric_label: normalize(MEMORY_EVIDENCE_FIELD, draft.metric_label)?,
            case_count: case_count.get(),
            policy_hash: crate::artifact::Hash::new(draft.policy_hash)
                .map_err(MemoryError::InvalidHash)?,
            evidence_hash: crate::artifact::Hash::new(draft.evidence_hash)
                .map_err(MemoryError::InvalidHash)?,
        })
    }

    /// Returns benchmark family.
    #[must_use]
    pub fn benchmark_family(&self) -> &str {
        self.benchmark_family.as_str()
    }

    /// Returns run label.
    #[must_use]
    pub fn run_label(&self) -> &str {
        self.run_label.as_str()
    }

    /// Returns metric label.
    #[must_use]
    pub fn metric_label(&self) -> &str {
        self.metric_label.as_str()
    }

    /// Returns imported case count.
    #[must_use]
    pub const fn case_count(&self) -> u32 {
        self.case_count
    }

    /// Returns policy hash.
    #[must_use]
    pub const fn policy_hash(&self) -> &crate::artifact::Hash {
        &self.policy_hash
    }

    /// Returns evidence hash.
    #[must_use]
    pub const fn evidence_hash(&self) -> &crate::artifact::Hash {
        &self.evidence_hash
    }
}

/// Reduces safe memory evaluation fixtures and observed ledgers into deterministic metrics.
///
/// # Errors
///
/// Returns [`MemoryError`] when observed IDs are unsafe or counts exceed supported bounds.
///
/// # Panics
///
/// Panics if observed or fixture descriptor counts drift above the supported bound after
/// validation. This indicates an internal evaluation-fixture contract violation.
// r[impl agent_harness_runtime.memory_evaluation.metrics]
pub fn reduce_memory_evaluation_metrics(
    input: MemoryEvaluationInput<'_>,
) -> Result<MemoryEvaluationMetrics, MemoryError> {
    if input.observed.len() > MAX_MEMORY_DESCRIPTORS {
        return Err(MemoryError::TooManyItems {
            field: DESCRIPTOR_ID_FIELD.as_str(),
            max_items: MAX_MEMORY_DESCRIPTORS,
            actual_items: input.observed.len(),
        });
    }
    assert!(input.observed.len() <= MAX_MEMORY_DESCRIPTORS);
    assert!(input.fixture.expected_record_ids().len() <= MAX_MEMORY_DESCRIPTORS);
    let mut relevant_hit_count = u32::default();
    let mut stale_hit_count = u32::default();
    let mut contradiction_hit_count = u32::default();
    let mut answer_used_count = u32::default();
    for observed in input.observed {
        normalize(DESCRIPTOR_ID_FIELD, observed.record_id)?;
        let is_expected =
            contains_record_id(input.fixture.expected_record_ids(), observed.record_id);
        if is_expected {
            relevant_hit_count = relevant_hit_count.saturating_add(RANK_OFFSET);
        }
        if observed.freshness == MemoryFreshness::Stale {
            stale_hit_count = stale_hit_count.saturating_add(RANK_OFFSET);
        }
        if observed.contradictory {
            contradiction_hit_count = contradiction_hit_count.saturating_add(RANK_OFFSET);
        }
        if observed.answer_used {
            answer_used_count = answer_used_count.saturating_add(RANK_OFFSET);
        }
    }
    let observed_count = memory_count_from_len(input.observed.len())?;
    let expected_count = memory_count_from_len(input.fixture.expected_record_ids().len())?;
    Ok(MemoryEvaluationMetrics {
        observed_count,
        expected_count,
        relevant_hit_count,
        stale_hit_count,
        contradiction_hit_count,
        answer_used_count,
        precision_percent: percent(PercentInput {
            numerator_count: relevant_hit_count,
            denominator_count: observed_count,
        }),
        recall_percent: percent(PercentInput {
            numerator_count: relevant_hit_count,
            denominator_count: expected_count,
        }),
        abstention_correct: input.fixture.abstention_expected()
            == (answer_used_count == u32::default()),
    })
}

/// Candidate memory returned by a retrieval adapter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MemoryRetrievalCandidate<'a> {
    /// Normalized record envelope.
    pub record: &'a MemoryRecordEnvelope,
    /// Safe match reason.
    pub match_reason: MemoryMatchReason,
    /// Adapter-declared relevance percentage.
    pub relevance_percent: u8,
    /// Safe backend label; never used for model-visible ordering.
    pub backend_label: &'a str,
}

/// Store-agnostic retrieval policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MemoryRetrievalPolicy<'a> {
    /// Safe query label.
    pub query_label: &'a str,
    /// Allowed memory scopes.
    pub allowed_scopes: &'a [MemoryTrustScope],
    /// Whether stale records may be selected.
    pub include_stale: bool,
    /// Prompt budget for memory injection.
    pub budget: MemoryInjectionBudget,
}

/// Borrowed retrieval-quality policy before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MemoryRetrievalQualityPolicyDraft<'a> {
    /// Safe quality-policy label.
    pub policy_label: &'a str,
    /// BLAKE3 policy hash.
    pub policy_hash: &'a str,
    /// Whether manual pins sort before other match reasons.
    pub prioritize_manual_pins: bool,
    /// Category preference order.
    pub category_priority: &'a [MemoryRecordCategory],
    /// Number of records per diversity group allowed to consume budget.
    pub diversity_group_limit: u32,
    /// Raw policy text, forbidden at this boundary.
    pub raw_policy_text: Option<&'a str>,
}

/// Validated retrieval-quality policy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemoryRetrievalQualityPolicy {
    policy_label: String,
    policy_hash: crate::artifact::Hash,
    prioritize_manual_pins: bool,
    category_priority: Vec<MemoryRecordCategory>,
    diversity_group_limit: NonZeroU32,
}

impl MemoryRetrievalQualityPolicy {
    /// Validates declarative retrieval-quality policy metadata.
    ///
    /// # Errors
    ///
    /// Returns [`MemoryError`] when labels, hashes, bounds, or leak checks fail.
    // r[impl agent_harness_runtime.memory_retrieval_quality.policy]
    pub fn validate(draft: &MemoryRetrievalQualityPolicyDraft<'_>) -> Result<Self, MemoryError> {
        if draft.raw_policy_text.is_some() {
            return Err(memory_leak(MemoryReceiptLeak::RawMemoryBody));
        }
        let policy_label = normalize(MEMORY_QUERY_LABEL_FIELD, draft.policy_label)?;
        if draft.category_priority.len() > MAX_MEMORY_DESCRIPTORS {
            return Err(MemoryError::TooManyItems {
                field: DESCRIPTOR_ID_FIELD.as_str(),
                max_items: MAX_MEMORY_DESCRIPTORS,
                actual_items: draft.category_priority.len(),
            });
        }
        let error_label = policy_label.clone();
        Ok(Self {
            policy_label,
            policy_hash: crate::artifact::Hash::new(draft.policy_hash)
                .map_err(MemoryError::InvalidHash)?,
            prioritize_manual_pins: draft.prioritize_manual_pins,
            category_priority: draft.category_priority.to_vec(),
            diversity_group_limit: NonZeroU32::new(draft.diversity_group_limit).ok_or(
                MemoryError::ZeroBudgetUnits {
                    descriptor_id: error_label,
                },
            )?,
        })
    }

    /// Returns safe policy label.
    #[must_use]
    pub fn policy_label(&self) -> &str {
        self.policy_label.as_str()
    }

    /// Returns policy hash.
    #[must_use]
    pub const fn policy_hash(&self) -> &crate::artifact::Hash {
        &self.policy_hash
    }

    /// Returns whether manual pins sort before other match reasons.
    #[must_use]
    pub const fn prioritize_manual_pins(&self) -> bool {
        self.prioritize_manual_pins
    }

    /// Returns category priority order.
    #[must_use]
    pub fn category_priority(&self) -> &[MemoryRecordCategory] {
        self.category_priority.as_slice()
    }

    /// Returns diversity group limit.
    #[must_use]
    pub const fn diversity_group_limit(&self) -> NonZeroU32 {
        self.diversity_group_limit
    }
}

/// Borrowed rerank evidence before validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MemoryRerankEvidenceDraft<'a> {
    /// Safe record identity.
    pub record_id: &'a str,
    /// BLAKE3 policy hash this evidence was produced under.
    pub policy_hash: &'a str,
    /// Safe reason label.
    pub reason_label: &'a str,
    /// Bounded rerank score percentage.
    pub score_percent: u8,
    /// Raw memory body, forbidden at this boundary.
    pub raw_memory_body: Option<&'a str>,
    /// Provider payload marker, forbidden at this boundary.
    pub provider_payload: Option<&'a str>,
    /// Credential marker, forbidden at this boundary.
    pub credential: Option<&'a str>,
}

/// Safe rerank evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemoryRerankEvidence {
    record_id: String,
    policy_hash: crate::artifact::Hash,
    reason_label: String,
    score: MemoryConfidence,
}

impl MemoryRerankEvidence {
    /// Validates safe rerank evidence.
    ///
    /// # Errors
    ///
    /// Returns [`MemoryError`] when labels, hashes, scores, or leak checks fail.
    // r[impl agent_harness_runtime.memory_retrieval_quality.rerank_evidence]
    pub fn validate(draft: &MemoryRerankEvidenceDraft<'_>) -> Result<Self, MemoryError> {
        if draft.raw_memory_body.is_some() {
            return Err(memory_leak(MemoryReceiptLeak::RawMemoryBody));
        }
        if draft.provider_payload.is_some() {
            return Err(memory_leak(MemoryReceiptLeak::ProviderPayload));
        }
        if draft.credential.is_some() {
            return Err(memory_leak(MemoryReceiptLeak::Credential));
        }
        Ok(Self {
            record_id: normalize(DESCRIPTOR_ID_FIELD, draft.record_id)?,
            policy_hash: crate::artifact::Hash::new(draft.policy_hash)
                .map_err(MemoryError::InvalidHash)?,
            reason_label: normalize(MEMORY_EVIDENCE_FIELD, draft.reason_label)?,
            score: MemoryConfidence::new(draft.score_percent)?,
        })
    }

    /// Returns record identity.
    #[must_use]
    pub fn record_id(&self) -> &str {
        self.record_id.as_str()
    }

    /// Returns policy hash.
    #[must_use]
    pub const fn policy_hash(&self) -> &crate::artifact::Hash {
        &self.policy_hash
    }

    /// Returns reason label.
    #[must_use]
    pub fn reason_label(&self) -> &str {
        self.reason_label.as_str()
    }

    /// Returns bounded rerank score.
    #[must_use]
    pub const fn score(&self) -> MemoryConfidence {
        self.score
    }
}

/// Input for quality-aware pure retrieval planning.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MemoryQualityRetrievalInput<'a> {
    /// Retrieval policy.
    pub retrieval_policy: MemoryRetrievalPolicy<'a>,
    /// Quality policy.
    pub quality_policy: &'a MemoryRetrievalQualityPolicy,
    /// Candidate records from one or more adapters.
    pub candidates: &'a [MemoryRetrievalCandidate<'a>],
    /// Optional rerank evidence.
    pub rerank_evidence: &'a [MemoryRerankEvidence],
}

/// Input for pure store-agnostic retrieval planning.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MemoryRetrievalInput<'a> {
    /// Retrieval policy.
    pub policy: MemoryRetrievalPolicy<'a>,
    /// Candidate records from one or more adapters.
    pub candidates: &'a [MemoryRetrievalCandidate<'a>],
}

/// Outcome status for one memory retrieval candidate.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MemoryRetrievalOutcomeStatus {
    /// Record is selected for prompt influence.
    Selected,
    /// Record scope is not allowed by policy.
    SkippedScope,
    /// Record is stale and stale inclusion is disabled.
    Stale,
    /// Record is redacted before model-visible influence.
    Redacted,
    /// Record conflicts with an already selected record.
    Conflicting,
    /// Record exceeds policy diversity group limit.
    DiversityDuplicate,
    /// Record exceeds the remaining prompt budget.
    OverBudget,
    /// Record has been superseded by another record.
    Superseded,
    /// Record was denied or unavailable by admission.
    Denied,
}

/// Safe store-agnostic retrieval outcome.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemoryRetrievalOutcome {
    record_id: String,
    source_label: String,
    evidence_handle: String,
    scope: MemoryTrustScope,
    confidence: MemoryConfidence,
    body_hash: crate::artifact::Hash,
    category: MemoryRecordCategory,
    status: MemoryRetrievalOutcomeStatus,
    match_reason: MemoryMatchReason,
    relevance: MemoryConfidence,
    rank: u32,
    requested_units: NonZeroU32,
    consumed_units: u32,
    conflict_group: Option<String>,
}

impl MemoryRetrievalOutcome {
    /// Returns record identity.
    #[must_use]
    pub fn record_id(&self) -> &str {
        self.record_id.as_str()
    }

    /// Returns safe source label.
    #[must_use]
    pub fn source_label(&self) -> &str {
        self.source_label.as_str()
    }

    /// Returns safe evidence handle.
    #[must_use]
    pub fn evidence_handle(&self) -> &str {
        self.evidence_handle.as_str()
    }

    /// Returns memory scope.
    #[must_use]
    pub const fn scope(&self) -> MemoryTrustScope {
        self.scope
    }

    /// Returns record confidence.
    #[must_use]
    pub const fn confidence(&self) -> MemoryConfidence {
        self.confidence
    }

    /// Returns BLAKE3 body hash.
    #[must_use]
    pub const fn body_hash(&self) -> &crate::artifact::Hash {
        &self.body_hash
    }

    /// Returns category-aware memory taxonomy label.
    #[must_use]
    pub const fn category(&self) -> MemoryRecordCategory {
        self.category
    }

    /// Returns outcome status.
    #[must_use]
    pub const fn status(&self) -> MemoryRetrievalOutcomeStatus {
        self.status
    }

    /// Returns match reason.
    #[must_use]
    pub const fn match_reason(&self) -> MemoryMatchReason {
        self.match_reason
    }

    /// Returns relevance score.
    #[must_use]
    pub const fn relevance(&self) -> MemoryConfidence {
        self.relevance
    }

    /// Returns model-visible rank.
    #[must_use]
    pub const fn rank(&self) -> u32 {
        self.rank
    }

    /// Returns requested units.
    #[must_use]
    pub const fn requested_units(&self) -> NonZeroU32 {
        self.requested_units
    }

    /// Returns consumed units.
    #[must_use]
    pub const fn consumed_units(&self) -> u32 {
        self.consumed_units
    }

    /// Returns optional conflict group.
    #[must_use]
    pub fn conflict_group(&self) -> Option<&str> {
        self.conflict_group.as_deref()
    }
}

/// Pure retrieval plan with selected and skipped memory evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemoryRetrievalPlan {
    outcomes: Vec<MemoryRetrievalOutcome>,
    selected_count: u32,
    consumed_units: u32,
}

impl MemoryRetrievalPlan {
    /// Returns safe outcomes in deterministic model-visible order.
    #[must_use]
    pub fn outcomes(&self) -> &[MemoryRetrievalOutcome] {
        self.outcomes.as_slice()
    }

    /// Returns selected outcome count.
    #[must_use]
    pub const fn selected_count(&self) -> u32 {
        self.selected_count
    }

    /// Returns total consumed budget units.
    #[must_use]
    pub const fn consumed_units(&self) -> u32 {
        self.consumed_units
    }
}

/// Plans memory retrieval without contacting stores, embedding services, providers, files, clocks, or networks.
///
/// # Errors
///
/// Returns [`MemoryError`] when candidate metadata, scores, labels, or bounds are invalid.
///
/// # Panics
///
/// Panics only if internal memory bounds or validated budget invariants are violated.
// r[impl agent_harness_runtime.memory_adapters.retrieval_plan]
pub fn plan_memory_retrieval(
    input: MemoryRetrievalInput<'_>,
) -> Result<MemoryRetrievalPlan, MemoryError> {
    normalize(MEMORY_QUERY_LABEL_FIELD, input.policy.query_label)?;
    assert!(MAX_MEMORY_DESCRIPTORS > usize::default());
    assert!(input.policy.budget.units().get() > ZERO_MEMORY_UNITS);
    if input.candidates.len() > MAX_MEMORY_DESCRIPTORS {
        return Err(MemoryError::TooManyItems {
            field: DESCRIPTOR_ID_FIELD.as_str(),
            max_items: MAX_MEMORY_DESCRIPTORS,
            actual_items: input.candidates.len(),
        });
    }
    assert!(input.candidates.len() <= MAX_MEMORY_DESCRIPTORS);
    let mut ordered: Vec<&MemoryRetrievalCandidate<'_>> =
        Vec::with_capacity(input.candidates.len());
    for candidate in input.candidates {
        normalize(MEMORY_ADAPTER_LABEL_FIELD, candidate.backend_label)?;
        MemoryConfidence::new(candidate.relevance_percent)?;
        ordered.push(candidate);
    }
    assert_eq!(ordered.len(), input.candidates.len());
    assert!(ordered.len() <= MAX_MEMORY_DESCRIPTORS);
    ordered.sort_by(|left, right| compare_memory_retrieval_candidates(left, right));
    build_memory_retrieval_plan(ordered.as_slice(), &input.policy, None, false)
}

/// Plans retrieval with declarative quality policy and safe optional rerank evidence.
///
/// # Errors
///
/// Returns [`MemoryError`] when candidates, rerank evidence, policy, or bounds fail validation.
///
/// # Panics
///
/// Panics only if internal memory bounds or validated budget invariants are violated.
// r[impl agent_harness_runtime.memory_retrieval_quality.planning]
// r[impl agent_harness_runtime.memory_retrieval_quality.diversity]
pub fn plan_memory_retrieval_with_quality(
    input: MemoryQualityRetrievalInput<'_>,
) -> Result<MemoryRetrievalPlan, MemoryError> {
    normalize(MEMORY_QUERY_LABEL_FIELD, input.retrieval_policy.query_label)?;
    assert!(input.retrieval_policy.budget.units().get() > ZERO_MEMORY_UNITS);
    if input.candidates.len() > MAX_MEMORY_DESCRIPTORS {
        return Err(MemoryError::TooManyItems {
            field: DESCRIPTOR_ID_FIELD.as_str(),
            max_items: MAX_MEMORY_DESCRIPTORS,
            actual_items: input.candidates.len(),
        });
    }
    if input.rerank_evidence.len() > MAX_MEMORY_DESCRIPTORS {
        return Err(MemoryError::TooManyItems {
            field: DESCRIPTOR_ID_FIELD.as_str(),
            max_items: MAX_MEMORY_DESCRIPTORS,
            actual_items: input.rerank_evidence.len(),
        });
    }
    assert!(input.candidates.len() <= MAX_MEMORY_DESCRIPTORS);
    assert!(input.rerank_evidence.len() <= MAX_MEMORY_DESCRIPTORS);
    let mut ordered: Vec<&MemoryRetrievalCandidate<'_>> =
        Vec::with_capacity(input.candidates.len());
    for candidate in input.candidates {
        normalize(MEMORY_ADAPTER_LABEL_FIELD, candidate.backend_label)?;
        MemoryConfidence::new(candidate.relevance_percent)?;
        ordered.push(candidate);
    }
    ordered.sort_by(|left, right| {
        compare_quality_retrieval_candidates(
            left,
            right,
            input.quality_policy,
            input.rerank_evidence,
        )
    });
    build_memory_retrieval_plan(
        ordered.as_slice(),
        &input.retrieval_policy,
        Some(input.quality_policy),
        true,
    )
}

fn build_memory_retrieval_plan(
    ordered: &[&MemoryRetrievalCandidate<'_>],
    policy: &MemoryRetrievalPolicy<'_>,
    quality_policy: Option<&MemoryRetrievalQualityPolicy>,
    use_quality: bool,
) -> Result<MemoryRetrievalPlan, MemoryError> {
    assert!(ordered.len() <= MAX_MEMORY_DESCRIPTORS);
    assert!(policy.budget.units().get() > ZERO_MEMORY_UNITS);
    let mut group_counts = MemoryDiversityGroupCounts::with_capacity(MAX_MEMORY_DESCRIPTORS);
    let mut conflict_groups: Vec<String> = Vec::with_capacity(MAX_MEMORY_DESCRIPTORS);
    let mut outcomes = Vec::with_capacity(ordered.len());
    let mut consumed_units = ZERO_MEMORY_UNITS;
    let mut selected_count = u32::default();
    for candidate in ordered {
        let decision = if let Some(quality) = quality_policy {
            memory_quality_retrieval_decision(MemoryQualityRetrievalDecisionInput {
                candidate,
                policy,
                quality,
                consumed_units,
                group_counts: group_counts.as_slice(),
            })
        } else {
            memory_retrieval_decision(
                candidate,
                policy,
                consumed_units,
                conflict_groups.as_slice(),
            )
        };
        if decision.status == MemoryRetrievalOutcomeStatus::Selected {
            selected_count = selected_count.saturating_add(RANK_OFFSET);
            consumed_units = consumed_units.saturating_add(decision.consumed_units);
            if let Some(group) = candidate.record.conflict_group() {
                if use_quality {
                    group_counts.increment(group);
                } else {
                    conflict_groups.push(String::from(group));
                }
            }
        }
        outcomes.push(memory_retrieval_outcome(MemoryRetrievalOutcomeInput {
            candidate,
            status: decision.status,
            rank: memory_rank_from_len(outcomes.len())?,
            consumed_units: decision.consumed_units,
        })?);
    }
    Ok(MemoryRetrievalPlan {
        outcomes,
        selected_count,
        consumed_units,
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct MemoryRetrievalDecision {
    status: MemoryRetrievalOutcomeStatus,
    consumed_units: u32,
}

fn memory_retrieval_decision(
    candidate: &MemoryRetrievalCandidate<'_>,
    policy: &MemoryRetrievalPolicy<'_>,
    consumed_units: u32,
    conflict_groups: &[String],
) -> MemoryRetrievalDecision {
    let record = candidate.record;
    assert!(!record.id().is_empty());
    assert!(!record.source_label().is_empty());
    assert!(!record.evidence_handle().is_empty());
    assert!(record.budget_units().get() > ZERO_MEMORY_UNITS);
    assert!(policy.budget.units().get() > ZERO_MEMORY_UNITS);
    if let Some(status) = common_retrieval_skip(record, policy) {
        return skipped_retrieval(status);
    }
    if let Some(group) = record.conflict_group() {
        if conflict_groups.iter().any(|selected| selected == group) {
            return skipped_retrieval(MemoryRetrievalOutcomeStatus::Conflicting);
        }
    }
    budget_retrieval_decision(record, policy, consumed_units)
}

#[derive(Clone, Copy)]
struct MemoryQualityRetrievalDecisionInput<'a> {
    candidate: &'a MemoryRetrievalCandidate<'a>,
    policy: &'a MemoryRetrievalPolicy<'a>,
    quality: &'a MemoryRetrievalQualityPolicy,
    consumed_units: u32,
    group_counts: &'a [(String, u32)],
}

fn memory_quality_retrieval_decision(
    input: MemoryQualityRetrievalDecisionInput<'_>,
) -> MemoryRetrievalDecision {
    let record = input.candidate.record;
    if let Some(status) = common_retrieval_skip(record, input.policy) {
        return skipped_retrieval(status);
    }
    if let Some(group) = record.conflict_group() {
        if group_count(input.group_counts, group) >= input.quality.diversity_group_limit().get() {
            return skipped_retrieval(MemoryRetrievalOutcomeStatus::DiversityDuplicate);
        }
    }
    budget_retrieval_decision(record, input.policy, input.consumed_units)
}

fn common_retrieval_skip(
    record: &MemoryRecordEnvelope,
    policy: &MemoryRetrievalPolicy<'_>,
) -> Option<MemoryRetrievalOutcomeStatus> {
    if record.admission_status() != MemoryAdmissionStatus::Admitted {
        return Some(MemoryRetrievalOutcomeStatus::Denied);
    }
    if !scope_allowed(record.scope(), policy.allowed_scopes) {
        return Some(MemoryRetrievalOutcomeStatus::SkippedScope);
    }
    if record.freshness() == MemoryFreshness::Stale && !policy.include_stale {
        return Some(MemoryRetrievalOutcomeStatus::Stale);
    }
    if record.redaction_class() == MemoryRedactionClass::Secret {
        return Some(MemoryRetrievalOutcomeStatus::Redacted);
    }
    if record.superseded_by_id().is_some() {
        return Some(MemoryRetrievalOutcomeStatus::Superseded);
    }
    None
}

fn budget_retrieval_decision(
    record: &MemoryRecordEnvelope,
    policy: &MemoryRetrievalPolicy<'_>,
    consumed_units: u32,
) -> MemoryRetrievalDecision {
    let remaining_units = policy.budget.units().get().saturating_sub(consumed_units);
    let requested_units = record.budget_units().get();
    if requested_units > remaining_units {
        return skipped_retrieval(MemoryRetrievalOutcomeStatus::OverBudget);
    }
    MemoryRetrievalDecision {
        status: MemoryRetrievalOutcomeStatus::Selected,
        consumed_units: requested_units,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct MemoryRetrievalOutcomeInput<'a> {
    candidate: &'a MemoryRetrievalCandidate<'a>,
    status: MemoryRetrievalOutcomeStatus,
    rank: u32,
    consumed_units: u32,
}

fn memory_retrieval_outcome(
    input: MemoryRetrievalOutcomeInput<'_>,
) -> Result<MemoryRetrievalOutcome, MemoryError> {
    let relevance = MemoryConfidence::new(input.candidate.relevance_percent)?;
    Ok(MemoryRetrievalOutcome {
        record_id: String::from(input.candidate.record.id()),
        source_label: String::from(input.candidate.record.source_label()),
        evidence_handle: String::from(input.candidate.record.evidence_handle()),
        scope: input.candidate.record.scope(),
        confidence: input.candidate.record.confidence(),
        body_hash: input.candidate.record.body_hash().clone(),
        category: input.candidate.record.category(),
        status: input.status,
        match_reason: input.candidate.match_reason,
        relevance,
        rank: input.rank,
        requested_units: input.candidate.record.budget_units(),
        consumed_units: input.consumed_units,
        conflict_group: input.candidate.record.conflict_group().map(String::from),
    })
}

fn skipped_retrieval(status: MemoryRetrievalOutcomeStatus) -> MemoryRetrievalDecision {
    MemoryRetrievalDecision {
        status,
        consumed_units: ZERO_MEMORY_UNITS,
    }
}

fn memory_rank_from_len(len: usize) -> Result<u32, MemoryError> {
    assert!(len < MAX_MEMORY_DESCRIPTORS);
    let rank = len.saturating_add(RANK_OFFSET_INDEX);
    assert!(rank <= MAX_MEMORY_DESCRIPTORS);
    u32::try_from(rank).map_err(|_| MemoryError::TooManyItems {
        field: DESCRIPTOR_ID_FIELD.as_str(),
        max_items: MAX_MEMORY_DESCRIPTORS,
        actual_items: rank,
    })
}

fn compare_memory_retrieval_candidates(
    left: &MemoryRetrievalCandidate<'_>,
    right: &MemoryRetrievalCandidate<'_>,
) -> core::cmp::Ordering {
    right
        .relevance_percent
        .cmp(&left.relevance_percent)
        .then(right.record.confidence().cmp(&left.record.confidence()))
        .then(left.record.scope().cmp(&right.record.scope()))
        .then(left.record.category().cmp(&right.record.category()))
        .then(left.record.kind().cmp(&right.record.kind()))
        .then(left.record.id().cmp(right.record.id()))
}

fn compare_quality_retrieval_candidates(
    left: &MemoryRetrievalCandidate<'_>,
    right: &MemoryRetrievalCandidate<'_>,
    quality: &MemoryRetrievalQualityPolicy,
    rerank_evidence: &[MemoryRerankEvidence],
) -> core::cmp::Ordering {
    rerank_score(right, quality, rerank_evidence)
        .cmp(&rerank_score(left, quality, rerank_evidence))
        .then(manual_pin_score(right, quality).cmp(&manual_pin_score(left, quality)))
        .then(
            category_priority(left.record.category(), quality)
                .cmp(&category_priority(right.record.category(), quality)),
        )
        .then(right.relevance_percent.cmp(&left.relevance_percent))
        .then(right.record.confidence().cmp(&left.record.confidence()))
        .then(left.record.id().cmp(right.record.id()))
}

fn rerank_score(
    candidate: &MemoryRetrievalCandidate<'_>,
    quality: &MemoryRetrievalQualityPolicy,
    rerank_evidence: &[MemoryRerankEvidence],
) -> u8 {
    rerank_evidence
        .iter()
        .find(|evidence| {
            evidence.record_id() == candidate.record.id()
                && evidence.policy_hash() == quality.policy_hash()
        })
        .map_or(u8::default(), |evidence| evidence.score().percent())
}

fn manual_pin_score(
    candidate: &MemoryRetrievalCandidate<'_>,
    quality: &MemoryRetrievalQualityPolicy,
) -> u8 {
    if quality.prioritize_manual_pins() && candidate.match_reason == MemoryMatchReason::ManualPin {
        return MAX_MEMORY_CONFIDENCE_PERCENT;
    }
    u8::default()
}

fn category_priority(
    category: MemoryRecordCategory,
    quality: &MemoryRetrievalQualityPolicy,
) -> usize {
    quality
        .category_priority()
        .iter()
        .position(|candidate| *candidate == category)
        .unwrap_or(MAX_MEMORY_DESCRIPTORS)
}

fn group_count(group_counts: &[(String, u32)], group: &str) -> u32 {
    group_counts
        .iter()
        .find(|(candidate, _)| candidate == group)
        .map_or(u32::default(), |(_, count)| *count)
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct MemoryDiversityGroupCounts {
    entries: Vec<(String, u32)>,
}

impl MemoryDiversityGroupCounts {
    fn with_capacity(capacity_items: usize) -> Self {
        Self {
            entries: Vec::with_capacity(capacity_items),
        }
    }

    fn as_slice(&self) -> &[(String, u32)] {
        self.entries.as_slice()
    }

    fn increment(&mut self, group: &str) {
        if let Some((_, count)) = self
            .entries
            .iter_mut()
            .find(|(candidate, _)| candidate == group)
        {
            *count = count.saturating_add(RANK_OFFSET);
            return;
        }
        self.entries.push((String::from(group), RANK_OFFSET));
    }
}

fn memory_record_category(
    record_id: &str,
    kind: GenericMemoryRecordKind,
    category_label: Option<&str>,
    require_explicit_category: bool,
) -> Result<MemoryRecordCategory, MemoryError> {
    if let Some(label) = category_label {
        let normalized = normalize(MEMORY_CATEGORY_FIELD, label)?;
        return MemoryRecordCategory::from_label(normalized.as_str()).ok_or_else(|| {
            MemoryError::UnsupportedCategory {
                record_id: String::from(record_id),
                category_label: normalized,
            }
        });
    }
    if require_explicit_category {
        return Err(MemoryError::MissingCategory {
            record_id: String::from(record_id),
        });
    }
    Ok(memory_category_for_generic_kind(kind))
}

fn scope_allowed(scope: MemoryTrustScope, allowed_scopes: &[MemoryTrustScope]) -> bool {
    allowed_scopes.contains(&scope)
}

fn role_capability(role: GenericMemoryAdapterRole) -> GenericMemoryAdapterCapability {
    match role {
        GenericMemoryAdapterRole::SourceImport => GenericMemoryAdapterCapability::SourceImport,
        GenericMemoryAdapterRole::DurableStore => GenericMemoryAdapterCapability::DurableStore,
        GenericMemoryAdapterRole::RetrievalSearch => {
            GenericMemoryAdapterCapability::RetrievalSearch
        }
        GenericMemoryAdapterRole::Extraction => GenericMemoryAdapterCapability::Extraction,
        GenericMemoryAdapterRole::EmbeddingSearchSupport => {
            GenericMemoryAdapterCapability::EmbeddingSearch
        }
        GenericMemoryAdapterRole::Admission => GenericMemoryAdapterCapability::AdmissionReview,
        GenericMemoryAdapterRole::UpdateDelete => GenericMemoryAdapterCapability::UpdateDelete,
        GenericMemoryAdapterRole::InstructionSource => {
            GenericMemoryAdapterCapability::InstructionSource
        }
    }
}

fn reject_record_leaks(draft: &MemoryRecordEnvelopeDraft<'_>) -> Result<(), MemoryError> {
    if draft.raw_body_marker.is_some() || draft.hidden_body_marker.is_some() {
        return Err(memory_leak(MemoryReceiptLeak::RawMemoryBody));
    }
    assert!(draft.raw_body_marker.is_none());
    assert!(draft.hidden_body_marker.is_none());
    Ok(())
}

fn reject_route_candidate_leaks(draft: &MemoryRouteCandidateDraft<'_>) -> Result<(), MemoryError> {
    if draft.raw_classifier_output.is_some() || draft.raw_memory_body.is_some() {
        return Err(memory_leak(MemoryReceiptLeak::RawMemoryBody));
    }
    if draft.credential.is_some() {
        return Err(memory_leak(MemoryReceiptLeak::Credential));
    }
    if draft.provider_payload.is_some() {
        return Err(memory_leak(MemoryReceiptLeak::ProviderPayload));
    }
    if draft.full_path.is_some() {
        return Err(memory_leak(MemoryReceiptLeak::FullPath));
    }
    Ok(())
}

fn reject_route_receipt_leaks(draft: &MemoryRouteReceiptDraft<'_>) -> Result<(), MemoryError> {
    if draft.raw_classifier_output.is_some() || draft.raw_memory_body.is_some() {
        return Err(memory_leak(MemoryReceiptLeak::RawMemoryBody));
    }
    if draft.credential.is_some() {
        return Err(memory_leak(MemoryReceiptLeak::Credential));
    }
    if draft.provider_payload.is_some() {
        return Err(memory_leak(MemoryReceiptLeak::ProviderPayload));
    }
    if draft.full_path.is_some() {
        return Err(memory_leak(MemoryReceiptLeak::FullPath));
    }
    Ok(())
}

fn reject_evaluation_fixture_leaks(
    draft: &MemoryEvaluationFixtureDraft<'_>,
) -> Result<(), MemoryError> {
    if draft.raw_prompt_text.is_some() || draft.raw_memory_body.is_some() {
        return Err(memory_leak(MemoryReceiptLeak::RawMemoryBody));
    }
    if draft.credential.is_some() {
        return Err(memory_leak(MemoryReceiptLeak::Credential));
    }
    if draft.provider_payload.is_some() {
        return Err(memory_leak(MemoryReceiptLeak::ProviderPayload));
    }
    if draft.full_path.is_some() {
        return Err(memory_leak(MemoryReceiptLeak::FullPath));
    }
    Ok(())
}

fn reject_external_import_leaks(
    draft: &MemoryExternalBenchmarkImportDraft<'_>,
) -> Result<(), MemoryError> {
    if draft.raw_conversation.is_some() || draft.raw_judge_prompt.is_some() {
        return Err(memory_leak(MemoryReceiptLeak::RawMemoryBody));
    }
    if draft.provider_payload.is_some() {
        return Err(memory_leak(MemoryReceiptLeak::ProviderPayload));
    }
    if draft.credential.is_some() {
        return Err(memory_leak(MemoryReceiptLeak::Credential));
    }
    Ok(())
}

fn normalize_ids(ids: &[&str]) -> Result<Vec<String>, MemoryError> {
    let mut normalized = Vec::with_capacity(ids.len());
    for id in ids {
        normalized.push(normalize(DESCRIPTOR_ID_FIELD, id)?);
    }
    Ok(normalized)
}

fn contains_record_id(records: &[String], needle: &str) -> bool {
    records.iter().any(|record| record == needle)
}

fn memory_count_from_len(len: usize) -> Result<u32, MemoryError> {
    u32::try_from(len).map_err(|_| MemoryError::TooManyItems {
        field: DESCRIPTOR_ID_FIELD.as_str(),
        max_items: MAX_MEMORY_DESCRIPTORS,
        actual_items: len,
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct PercentInput {
    numerator_count: u32,
    denominator_count: u32,
}

fn percent(input: PercentInput) -> u32 {
    let Some(denominator_count) = NonZeroU32::new(input.denominator_count) else {
        return u32::default();
    };
    let denominator_units = denominator_count.get();
    assert!(denominator_units != 0);
    input.numerator_count.saturating_mul(PERCENT_SCALE) / denominator_units
}

fn normalize_optional(
    field: crate::validation::FieldName,
    value: Option<&str>,
) -> Result<Option<String>, MemoryError> {
    value.map(|inner| normalize(field, inner)).transpose()
}
