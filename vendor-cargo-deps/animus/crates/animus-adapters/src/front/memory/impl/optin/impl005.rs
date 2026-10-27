#![allow(unknown_lints)]
#![cfg_attr(animus_octet, allow(tigerstyle::excessive_file_length))]

use super::{
    MemoryAdmissionDecision, MemoryAvailabilityStatus, MemoryCandidate, MemoryDescriptor,
    MemoryPersistenceIntent, OptInMemoryRecord, PromptAssemblyError, String, Vec,
};

/// Swappable memory adapter boundary kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MemoryAdapterBoundaryKind {
    /// Descriptor source discovery boundary.
    Source,
    /// Descriptor persistence boundary.
    Store,
    /// Compatibility import boundary.
    Importer,
    /// Provider-assisted or heuristic extraction boundary.
    Extraction,
    /// Host admission review boundary.
    Admission,
    /// Retrieval or search boundary for memory records.
    Retrieval,
    /// Update or delete boundary for stored records.
    Mutation,
    /// Availability-status boundary for memory services.
    Availability,
}

/// Error returned by host-provided memory adapter boundaries.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MemoryAdapterError {
    /// Adapter boundary was unavailable.
    Unavailable {
        /// Unavailable boundary kind.
        boundary: MemoryAdapterBoundaryKind,
    },
    /// Adapter returned invalid DTO metadata.
    Contract(MemoryError),
}

impl core::fmt::Display for MemoryAdapterError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Unavailable { boundary } => {
                write!(formatter, "memory adapter unavailable: {boundary:?}")
            }
            Self::Contract(error) => write!(formatter, "{error}"),
        }
    }
}

/// Result type used by memory adapter boundaries.
pub type MemoryAdapterResult<T> = Result<T, MemoryAdapterError>;

/// Swappable source adapter for already validated memory descriptors.
pub trait MemorySourceAdapter {
    /// Loads memory descriptors without allowing reusable logic to discover storage.
    ///
    /// # Errors
    ///
    /// Returns [`MemoryAdapterError`] when the host source is unavailable or returns invalid DTOs.
    fn load_memory_descriptors(&self) -> MemoryAdapterResult<Vec<MemoryDescriptor>>;
}

/// Swappable store adapter for admitted descriptors.
pub trait MemoryStoreAdapter {
    /// Persists an admitted descriptor when host policy allows it.
    ///
    /// # Errors
    ///
    /// Returns [`MemoryAdapterError`] when persistence is unavailable or rejected by the host.
    fn persist_memory_descriptor(
        &self,
        descriptor: &MemoryDescriptor,
    ) -> MemoryAdapterResult<MemoryPersistenceIntent>;
}

/// Swappable compatibility importer for external memory formats.
pub trait MemoryImporterAdapter {
    /// Converts a host-owned compatibility source into validated descriptors.
    ///
    /// # Errors
    ///
    /// Returns [`MemoryAdapterError`] when import is unavailable or compatibility input is invalid.
    fn import_memory_descriptors(
        &self,
        source_label: &str,
        source_hash: &crate::artifact::Hash,
    ) -> MemoryAdapterResult<Vec<MemoryDescriptor>>;
}

/// Swappable extraction adapter for learned-memory candidates.
pub trait MemoryExtractionAdapter {
    /// Proposes a learned-memory candidate for host admission.
    ///
    /// # Errors
    ///
    /// Returns [`MemoryAdapterError`] when extraction is unavailable or candidate metadata is invalid.
    fn propose_memory_candidate(&self) -> MemoryAdapterResult<MemoryCandidate>;
}

/// Swappable host admission adapter for learned-memory candidates.
pub trait MemoryAdmissionAdapter {
    /// Decides whether a candidate may persist or influence future prompts.
    ///
    /// # Errors
    ///
    /// Returns [`MemoryAdapterError`] when admission review is unavailable or decision metadata is invalid.
    fn decide_memory_admission(
        &self,
        candidate: &MemoryCandidate,
    ) -> MemoryAdapterResult<MemoryAdmissionDecision>;
}

/// Swappable retrieval adapter for opt-in memory records.
pub trait OptInMemoryRetrievalAdapter {
    /// Retrieves candidate memory records through a host-owned backend.
    ///
    /// # Errors
    ///
    /// Returns [`MemoryAdapterError`] when retrieval is unavailable or record DTOs are invalid.
    fn retrieve_memory_records(&self) -> MemoryAdapterResult<Vec<OptInMemoryRecord>>;
}

/// Swappable mutation adapter for opt-in memory records.
pub trait OptInMemoryMutationAdapter {
    /// Applies an update or delete for a host-owned memory record.
    ///
    /// # Errors
    ///
    /// Returns [`MemoryAdapterError`] when mutation is unavailable or rejected by the host.
    fn apply_memory_mutation(
        &self,
        record: &OptInMemoryRecord,
    ) -> MemoryAdapterResult<MemoryPersistenceIntent>;
}

/// Swappable availability adapter for memory services.
pub trait OptInMemoryAvailabilityAdapter {
    /// Reports whether memory storage, retrieval, and extraction are available.
    ///
    /// # Errors
    ///
    /// Returns [`MemoryAdapterError`] when service availability cannot be determined safely.
    fn memory_availability(&self) -> MemoryAdapterResult<MemoryAvailabilityStatus>;
}

/// Sensitive material class rejected from safe memory metadata.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MemoryReceiptLeak {
    /// Raw memory body was present.
    RawMemoryBody,
    /// Raw candidate body was present.
    RawCandidateBody,
    /// Credential-looking value was present.
    Credential,
    /// Provider payload was present.
    ProviderPayload,
    /// Full host path was present.
    FullPath,
    /// Unbounded prompt text was present.
    UnboundedPromptText,
}

/// Memory contract validation error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MemoryError {
    /// A text field failed adapter-boundary validation.
    Contract(crate::AdapterContractError),
    /// A BLAKE3 content hash was invalid.
    InvalidHash(crate::artifact::Error),
    /// Underlying prompt assembly failed.
    PromptAssembly(PromptAssemblyError),
    /// A bounded item list exceeded the allowed item count.
    TooManyItems {
        /// Field containing too many items.
        field: &'static str,
        /// Maximum supported items.
        max_items: usize,
        /// Actual item count.
        actual_items: usize,
    },
    /// Descriptor did not include a trust scope.
    MissingTrustScope {
        /// Descriptor identity.
        descriptor_id: String,
    },
    /// Descriptor did not include an inclusion policy.
    MissingInclusionPolicy {
        /// Descriptor identity.
        descriptor_id: String,
    },
    /// Descriptor freshness has expired.
    ExpiredDescriptor {
        /// Descriptor identity.
        descriptor_id: String,
    },
    /// Descriptor requested zero prompt budget units.
    ZeroBudgetUnits {
        /// Descriptor identity.
        descriptor_id: String,
    },
    /// Opt-in memory record did not include a scope.
    MissingRecordScope {
        /// Record identity.
        record_id: String,
    },
    /// Memory adapter descriptor did not declare capabilities.
    MissingAdapterCapabilities {
        /// Adapter identity.
        adapter_label: String,
    },
    /// Memory record or command did not include evidence metadata.
    MissingEvidence {
        /// Record, candidate, or command identity.
        record_id: String,
    },
    /// Explicit memory category metadata was required but absent.
    MissingCategory {
        /// Record identity.
        record_id: String,
    },
    /// Explicit memory category metadata was not supported.
    UnsupportedCategory {
        /// Record identity.
        record_id: String,
        /// Unsupported category label.
        category_label: String,
    },
    /// A memory-quality claim omitted a required baseline.
    MissingBaseline {
        /// Claim identity.
        claim_id: String,
    },
    /// Memory confidence was outside the supported range.
    InvalidConfidence {
        /// Minimum supported confidence percentage.
        min_percent: u8,
        /// Maximum supported confidence percentage.
        max_percent: u8,
        /// Actual confidence percentage.
        actual_percent: u8,
    },
    /// Memory injection budget was zero.
    ZeroInjectionBudget,
    /// Admission decision referenced a different candidate.
    AdmissionCandidateMismatch {
        /// Candidate identity under review.
        candidate_id: String,
        /// Candidate identity referenced by the decision.
        decision_candidate_id: String,
    },
    /// Safe metadata contained forbidden sensitive material.
    ReceiptLeak {
        /// Leak class.
        leak: MemoryReceiptLeak,
    },
}

impl core::fmt::Display for MemoryError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Contract(error) => write!(formatter, "{error}"),
            Self::InvalidHash(error) => write!(formatter, "{error}"),
            Self::PromptAssembly(error) => write!(formatter, "{error}"),
            Self::TooManyItems {
                field,
                max_items,
                actual_items,
            } => write!(
                formatter,
                "{field} supports at most {max_items} items; got {actual_items}"
            ),
            Self::MissingTrustScope { descriptor_id } => {
                write!(formatter, "{descriptor_id} is missing memory trust scope")
            }
            Self::MissingInclusionPolicy { descriptor_id } => {
                write!(
                    formatter,
                    "{descriptor_id} is missing memory inclusion policy"
                )
            }
            Self::ExpiredDescriptor { descriptor_id } => {
                write!(formatter, "{descriptor_id} has expired memory freshness")
            }
            Self::ZeroBudgetUnits { descriptor_id } => {
                write!(formatter, "{descriptor_id} must use non-zero budget units")
            }
            Self::MissingRecordScope { record_id } => {
                write!(formatter, "{record_id} is missing memory record scope")
            }
            Self::MissingAdapterCapabilities { adapter_label } => {
                write!(
                    formatter,
                    "{adapter_label} is missing memory adapter capabilities"
                )
            }
            Self::MissingEvidence { record_id } => {
                write!(formatter, "{record_id} is missing memory evidence metadata")
            }
            Self::MissingCategory { record_id } => {
                write!(formatter, "{record_id} is missing explicit memory category")
            }
            Self::UnsupportedCategory {
                record_id,
                category_label,
            } => write!(
                formatter,
                "{record_id} uses unsupported memory category {category_label}"
            ),
            Self::MissingBaseline { claim_id } => {
                write!(
                    formatter,
                    "{claim_id} is missing memory evaluation baseline"
                )
            }
            Self::InvalidConfidence {
                min_percent,
                max_percent,
                actual_percent,
            } => write!(
                formatter,
                "memory confidence must be between {min_percent} and {max_percent}; got {actual_percent}"
            ),
            Self::ZeroInjectionBudget => {
                formatter.write_str("memory injection budget must be non-zero")
            }
            Self::AdmissionCandidateMismatch {
                candidate_id,
                decision_candidate_id,
            } => write!(
                formatter,
                "memory admission decision for {decision_candidate_id} does not match candidate {candidate_id}"
            ),
            Self::ReceiptLeak { leak } => write!(formatter, "memory receipt leak: {leak:?}"),
        }
    }
}

impl From<crate::AdapterContractError> for MemoryError {
    fn from(error: crate::AdapterContractError) -> Self {
        Self::Contract(error)
    }
}

impl From<PromptAssemblyError> for MemoryError {
    fn from(error: PromptAssemblyError) -> Self {
        Self::PromptAssembly(error)
    }
}
