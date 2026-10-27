use serde::{Deserialize, Serialize};

use super::{SteelRlmError, SteelRlmEvaluation, SteelRlmKernel};

/// Versioned private Steel kernel journal schema.
pub const STEEL_KERNEL_JOURNAL_SCHEMA: &str = "animus.steel-kernel-journal.v1";
/// Exact Steel engine and Animus replay cohort.
pub const STEEL_KERNEL_COHORT: &str = "steel-core-0.8.2.animus-rlm-v1";
/// Exact resumable bridge replay schema.
pub const STEEL_KERNEL_BRIDGE_SCHEMA: &str = "animus.rlm-bridge-replay.v1";
/// Empty BLAKE3 chain head before the first private record.
pub const STEEL_KERNEL_EMPTY_HEAD: &str =
    "0000000000000000000000000000000000000000000000000000000000000000";

const JOURNAL_RECORD_HASH_DOMAIN: &str = "animus.steel-kernel-record.v1";
const JOURNAL_BODY_HASH_DOMAIN: &str = "animus.steel-kernel-record-body.v1";
const JOURNAL_RECEIPT_HASH_DOMAIN: &str = "animus.steel-kernel-restore-receipt.v1";
const SOURCE_HASH_DOMAIN: &str = "animus.steel-kernel-source.v1";
const MAX_PRIVATE_JOURNAL_BYTES: usize = 1_048_576;
const MAX_CONTEXT_RECORDS: usize = 256;
const MAX_CLEAR_RECORDS: usize = 1;
const MAX_MODULE_RECORDS: usize = 1;
const MAX_PRIVATE_RECORDS: usize = MAX_CONTEXT_RECORDS + MAX_CLEAR_RECORDS + MAX_MODULE_RECORDS;
const BLAKE3_HEX_CHARS: usize = 64;
const FIRST_RECORD_SEQUENCE: u32 = 0;
const NEXT_RECORD_INCREMENT: u32 = 1;
const VALUE_FINAL_CLASS: &str = "value";
const HOST_REQUEST_FINAL_CLASS: &str = "host_request";
const REDACTION_MARKER: &str = "private-kernel-content-omitted";
const RESTORE_NON_CLAIM: &str = "no-mid-cell-restore,no-effect-rollback,no-vm-image,no-encryption-claim,no-production-durability";

/// Durable recovery posture observed outside the committed kernel chain.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SteelKernelRecoveryPosture {
    /// The private journal ended at a clean committed boundary.
    Clean,
    /// An interrupted cell was observed after the committed boundary.
    InterruptedCell,
}

/// Safe restore result class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SteelKernelRestoreStatus {
    /// All private records restored from a clean journal.
    Restored,
    /// Only the prior committed head restored after an interrupted cell.
    PriorHeadRestored,
}

impl SteelKernelRestoreStatus {
    /// Returns the stable status label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Restored => "restored",
            Self::PriorHeadRestored => "prior_head_restored",
        }
    }
}

/// Exact current facts required before private replay starts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SteelKernelRestoreAdmission<'a> {
    /// BLAKE3 identity of the owning session.
    pub session_identity: &'a str,
    /// Exact host-owned kernel generation.
    pub kernel_generation: u64,
    /// Exact committed private kernel chain head.
    pub committed_head_identity: &'a str,
    /// Recovery posture from the protected durable channel.
    pub recovery_posture: SteelKernelRecoveryPosture,
}

/// Private values that must remain absent from safe restore evidence.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SteelKernelRestoreSensitiveDraft<'a> {
    /// Model-written Steel source.
    pub source: Option<&'a str>,
    /// Captured host result body.
    pub result: Option<&'a str>,
    /// Prompt or context body.
    pub prompt: Option<&'a str>,
    /// Credential material.
    pub credential: Option<&'a str>,
    /// Private physical path.
    pub private_path: Option<&'a str>,
    /// Raw evaluator diagnostic.
    pub raw_diagnostic: Option<&'a str>,
}

/// Metadata-only durable Steel kernel restore receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SteelKernelRestoreReceipt {
    schema: &'static str,
    status: SteelKernelRestoreStatus,
    session_identity: String,
    kernel_generation: u64,
    policy_identity: String,
    steel_cohort: &'static str,
    bridge_schema: &'static str,
    committed_head_identity: String,
    record_count: u32,
    cell_count: u32,
    binding_count: u32,
    captured_result_count: u32,
    module_count: u32,
    module_set_identity: Option<String>,
    redaction_marker: &'static str,
    non_claim: &'static str,
    receipt_identity: String,
}

impl SteelKernelRestoreReceipt {
    /// Returns the safe restore status.
    #[must_use]
    pub const fn status(&self) -> SteelKernelRestoreStatus {
        self.status
    }

    /// Returns the exact committed private chain head.
    #[must_use]
    pub fn committed_head_identity(&self) -> &str {
        self.committed_head_identity.as_str()
    }

    /// Returns the safe record count.
    #[must_use]
    pub const fn record_count(&self) -> u32 {
        self.record_count
    }

    /// Returns the safe receipt identity.
    #[must_use]
    pub fn receipt_identity(&self) -> &str {
        self.receipt_identity.as_str()
    }

    /// Returns the explicit bounded non-claim.
    #[must_use]
    pub const fn non_claim(&self) -> &'static str {
        self.non_claim
    }
}

/// Typed private journal or restore failure.
#[derive(Debug)]
pub enum SteelKernelJournalError {
    /// Private bytes were malformed or had unknown fields.
    InvalidEncoding,
    /// The private journal exceeded a finite record or byte bound.
    BoundExceeded,
    /// The journal schema was not the current exact schema.
    UnsupportedSchema,
    /// Session identity differed.
    SessionMismatch,
    /// Host-owned kernel generation differed.
    GenerationMismatch,
    /// Steel engine or RLM cohort differed.
    CohortMismatch,
    /// Current RLM policy identity differed.
    PolicyMismatch,
    /// Bridge replay schema differed.
    BridgeSchemaMismatch,
    /// The admitted committed head differed.
    HeadMismatch,
    /// A record body, identity, sequence, or prior-head link was corrupt.
    ChainCorrupt,
    /// A private context binding was malformed or drifted.
    ContextMismatch,
    /// Captured call order, identity, operation, or result differed.
    CapturedResultMismatch,
    /// Effect-disabled Steel replay failed or produced another final class.
    ReplayMismatch,
    /// Safe receipt input contained private material.
    ReceiptLeak,
    /// Steel rejected current policy, bounds, or replay source.
    Steel(SteelRlmError),
}

impl core::fmt::Display for SteelKernelJournalError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let label = match self {
            Self::InvalidEncoding => "invalid_encoding",
            Self::BoundExceeded => "bound_exceeded",
            Self::UnsupportedSchema => "unsupported_schema",
            Self::SessionMismatch => "session_mismatch",
            Self::GenerationMismatch => "generation_mismatch",
            Self::CohortMismatch => "cohort_mismatch",
            Self::PolicyMismatch => "policy_mismatch",
            Self::BridgeSchemaMismatch => "bridge_schema_mismatch",
            Self::HeadMismatch => "head_mismatch",
            Self::ChainCorrupt => "chain_corrupt",
            Self::ContextMismatch => "context_mismatch",
            Self::CapturedResultMismatch => "captured_result_mismatch",
            Self::ReplayMismatch => "replay_mismatch",
            Self::ReceiptLeak => "receipt_leak",
            Self::Steel(_error) => "steel_replay_rejected",
        };
        write!(formatter, "Steel kernel journal rejected: {label}")
    }
}

impl std::error::Error for SteelKernelJournalError {}

impl From<SteelRlmError> for SteelKernelJournalError {
    fn from(error: SteelRlmError) -> Self {
        Self::Steel(error)
    }
}

/// Validated private snapshot for one committed kernel safe point.
#[derive(Clone)]
pub struct SteelKernelPrivateJournal {
    wire: JournalWire,
    private_bytes: Vec<u8>,
}

impl core::fmt::Debug for SteelKernelPrivateJournal {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("SteelKernelPrivateJournal")
            .field("schema", &self.wire.schema)
            .field("session_identity", &self.wire.session_identity)
            .field("kernel_generation", &self.wire.kernel_generation)
            .field("policy_identity", &self.wire.policy_identity)
            .field("steel_cohort", &self.wire.steel_cohort)
            .field("bridge_schema", &self.wire.bridge_schema)
            .field("record_count", &self.wire.records.len())
            .field(
                "committed_head_identity",
                &self.wire.committed_head_identity,
            )
            .field("private_bytes", &"<private>")
            .finish()
    }
}

impl SteelKernelPrivateJournal {
    /// Returns the exact committed kernel chain head.
    #[must_use]
    pub fn committed_head_identity(&self) -> &str {
        self.wire.committed_head_identity.as_str()
    }

    /// Returns the exact session identity.
    #[must_use]
    pub fn session_identity(&self) -> &str {
        self.wire.session_identity.as_str()
    }

    /// Returns the host-owned kernel generation.
    #[must_use]
    pub const fn kernel_generation(&self) -> u64 {
        self.wire.kernel_generation
    }

    /// Returns the bounded private encoding for the protected journal channel.
    #[must_use]
    pub fn private_bytes(&self) -> &[u8] {
        self.private_bytes.as_slice()
    }

    /// Returns the safe private record count.
    #[must_use]
    pub fn record_count(&self) -> u32 {
        u32::try_from(self.wire.records.len()).unwrap_or(u32::MAX)
    }
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct JournalWire {
    schema: String,
    session_identity: String,
    kernel_generation: u64,
    environment_generation: u64,
    steel_cohort: String,
    rlm_cohort: String,
    rlm_cohort_revision: String,
    policy_identity: String,
    bridge_schema: String,
    records: Vec<RecordWire>,
    committed_head_identity: String,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct RecordWire {
    sequence: u32,
    prior_record_identity: String,
    body_identity: String,
    record_identity: String,
    body: RecordBodyWire,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum RecordBodyWire {
    Clear {
        clear_epoch: u64,
    },
    Context {
        bindings: Vec<BindingWire>,
    },
    Cell {
        source: String,
        source_identity: String,
        final_value_class: String,
        captured_calls: Vec<CallWire>,
    },
    Modules {
        module_set_identity: String,
        modules: Vec<SkillModuleWire>,
    },
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct SkillModuleWire {
    name: String,
    version: String,
    manifest_identity: String,
    frozen_source_identity: String,
    source_identity: String,
    source: String,
    exports: Vec<String>,
    dependencies: Vec<SkillDependencyWire>,
    capability_requests: Vec<String>,
    source_chars: u32,
    evaluation_fuel: u64,
    module_identity: String,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct SkillDependencyWire {
    name: String,
    version: String,
    module_identity: String,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct BindingWire {
    name: String,
    value: String,
    value_identity: String,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct CallWire {
    cell_identity: String,
    sequence: u32,
    call_identity: String,
    operation: String,
    request_identity: String,
    result_value: String,
    result_identity: String,
}

/// Projects metadata-only restore evidence from a validated private journal.
///
/// # Errors
///
/// Returns [`SteelKernelJournalError::ReceiptLeak`] without echoing private
/// content when any sensitive draft field is present.
// r[impl agent_harness_runtime.steel_kernel_restore.evidence]
pub fn project_steel_kernel_restore_receipt(
    journal: &SteelKernelPrivateJournal,
    status: SteelKernelRestoreStatus,
    sensitive: SteelKernelRestoreSensitiveDraft<'_>,
) -> Result<SteelKernelRestoreReceipt, SteelKernelJournalError> {
    project_restore_receipt(&journal.wire, status, sensitive)
}

impl SteelRlmKernel {
    /// Exports the current committed safe point as bounded private journal bytes.
    ///
    /// Source, context, and captured result bodies remain only in the returned
    /// private value. Its `Debug` output is redacted.
    ///
    /// # Errors
    ///
    /// Returns [`SteelKernelJournalError`] for malformed identities, bounds,
    /// serialization failure, or internal chain inconsistency.
    // r[impl agent_harness_runtime.steel_kernel_restore.journal]
    // r[impl agent_harness_runtime.steel_kernel_restore.safe_points]
    pub fn private_journal_snapshot(
        &self,
        session_identity: &str,
        kernel_generation: u64,
    ) -> Result<SteelKernelPrivateJournal, SteelKernelJournalError> {
        validate_blake3(session_identity).map_err(|()| SteelKernelJournalError::SessionMismatch)?;
        let mut bodies = Vec::new();
        if self.clear_epoch > 0 {
            bodies.push(RecordBodyWire::Clear {
                clear_epoch: self.clear_epoch,
            });
        }
        for context_commit in &self.context_commits {
            let bindings = context_commit
                .iter()
                .map(|binding| BindingWire {
                    name: String::from(binding.name()),
                    value: String::from(binding.value()),
                    value_identity: String::from(binding.value_hash()),
                })
                .collect();
            bodies.push(RecordBodyWire::Context { bindings });
        }
        if !self.skill_modules.is_empty() {
            let module_set_identity = self
                .skill_module_set_identity
                .clone()
                .ok_or(SteelKernelJournalError::ChainCorrupt)?;
            let modules = self
                .skill_modules
                .iter()
                .map(module_to_wire)
                .collect::<Vec<_>>();
            bodies.push(RecordBodyWire::Modules {
                module_set_identity,
                modules,
            });
        }
        for cell in &self.committed_cells {
            let captured_calls = cell
                .bridge_calls
                .iter()
                .map(|call| CallWire {
                    cell_identity: String::from(call.correlation().cell().cell_identity()),
                    sequence: call.correlation().sequence(),
                    call_identity: String::from(call.correlation().call_identity()),
                    operation: String::from(call.operation().as_str()),
                    request_identity: String::from(call.request_identity()),
                    result_value: String::from(call.result_value()),
                    result_identity: String::from(call.result_identity()),
                })
                .collect();
            bodies.push(RecordBodyWire::Cell {
                source: cell.source.clone(),
                source_identity: source_identity(cell.source.as_str()),
                final_value_class: cell.final_value_class.clone(),
                captured_calls,
            });
        }
        if bodies.len() > MAX_PRIVATE_RECORDS {
            return Err(SteelKernelJournalError::BoundExceeded);
        }
        let records = chain_records(bodies.as_slice())?;
        let committed_head_identity = records.last().map_or_else(
            || String::from(STEEL_KERNEL_EMPTY_HEAD),
            |record| record.record_identity.clone(),
        );
        let wire = JournalWire {
            schema: String::from(STEEL_KERNEL_JOURNAL_SCHEMA),
            session_identity: String::from(session_identity),
            kernel_generation,
            environment_generation: self.environment.generation(),
            steel_cohort: String::from(STEEL_KERNEL_COHORT),
            rlm_cohort: String::from(self.policy.cohort()),
            rlm_cohort_revision: String::from(self.policy.cohort_revision()),
            policy_identity: String::from(self.policy.identity()),
            bridge_schema: String::from(STEEL_KERNEL_BRIDGE_SCHEMA),
            records,
            committed_head_identity,
        };
        let private_bytes =
            serde_json::to_vec(&wire).map_err(|_error| SteelKernelJournalError::InvalidEncoding)?;
        if private_bytes.len() > MAX_PRIVATE_JOURNAL_BYTES {
            return Err(SteelKernelJournalError::BoundExceeded);
        }
        validate_wire(&wire, &self.policy)?;
        Ok(SteelKernelPrivateJournal {
            wire,
            private_bytes,
        })
    }

    /// Restores a fresh kernel from one exact private committed chain.
    ///
    /// Replay has no executor. Captured results are supplied directly to the
    /// sandboxed Steel replay path and every request identity is rechecked.
    /// The returned kernel becomes visible only after all records pass.
    ///
    /// # Errors
    ///
    /// Returns [`SteelKernelJournalError`] for admission drift, corruption,
    /// bounds, missing or extra results, request-order drift, or Steel failure.
    // r[impl agent_harness_runtime.steel_kernel_restore.replay]
    // r[impl agent_harness_runtime.steel_kernel_restore.admission]
    pub fn restore_private_journal(
        policy: &animus_adapter_rlm_kernel::RlmKernelPolicy,
        admission: SteelKernelRestoreAdmission<'_>,
        private_bytes: &[u8],
    ) -> Result<(Self, SteelKernelRestoreReceipt), SteelKernelJournalError> {
        if private_bytes.len() > MAX_PRIVATE_JOURNAL_BYTES {
            return Err(SteelKernelJournalError::BoundExceeded);
        }
        let wire: JournalWire = serde_json::from_slice(private_bytes)
            .map_err(|_error| SteelKernelJournalError::InvalidEncoding)?;
        validate_admission(&wire, policy, admission)?;
        validate_wire(&wire, policy)?;
        let mut kernel = Self::new(policy.clone())?;
        replay_records(&mut kernel, &wire)?;
        if kernel.environment.generation() != wire.environment_generation {
            return Err(SteelKernelJournalError::ReplayMismatch);
        }
        kernel.clear_epoch = clear_epoch(wire.records.as_slice());
        let status = match admission.recovery_posture {
            SteelKernelRecoveryPosture::Clean => SteelKernelRestoreStatus::Restored,
            SteelKernelRecoveryPosture::InterruptedCell => {
                SteelKernelRestoreStatus::PriorHeadRestored
            }
        };
        let receipt =
            project_restore_receipt(&wire, status, SteelKernelRestoreSensitiveDraft::default())?;
        Ok((kernel, receipt))
    }
}

fn replay_records(
    kernel: &mut SteelRlmKernel,
    wire: &JournalWire,
) -> Result<(), SteelKernelJournalError> {
    for record in &wire.records {
        match &record.body {
            RecordBodyWire::Clear { clear_epoch } => {
                if *clear_epoch == 0 {
                    return Err(SteelKernelJournalError::ChainCorrupt);
                }
                kernel.environment = animus_adapter_rlm_kernel::EnvironmentState::default();
                kernel.context_commits.clear();
                kernel.committed_cells.clear();
                kernel.skill_modules.clear();
                kernel.skill_module_set_identity = None;
                kernel.engine = super::build_engine(&[], &[], &[], &kernel.policy)?;
                kernel.clear_epoch = *clear_epoch;
            }
            RecordBodyWire::Context { bindings } => {
                let parsed = bindings
                    .iter()
                    .map(|binding| {
                        let parsed = animus_adapter_rlm_kernel::ContextBinding::new(
                            binding.name.as_str(),
                            binding.value.as_str(),
                            kernel.policy.bounds().max_environment_chars,
                        )
                        .map_err(|_error| SteelKernelJournalError::ContextMismatch)?;
                        if parsed.value_hash() != binding.value_identity {
                            return Err(SteelKernelJournalError::ContextMismatch);
                        }
                        Ok(parsed)
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                kernel.bind_prompt_context(parsed.as_slice())?;
            }
            RecordBodyWire::Modules {
                module_set_identity,
                modules,
            } => restore_modules(kernel, module_set_identity.as_str(), modules.as_slice())?,
            RecordBodyWire::Cell {
                source,
                source_identity: expected_source,
                final_value_class,
                captured_calls,
            } => {
                if source_identity(source.as_str()) != *expected_source {
                    return Err(SteelKernelJournalError::ChainCorrupt);
                }
                let calls = restore_calls(captured_calls, wire, &kernel.policy)?;
                let evaluation = if calls.is_empty() {
                    kernel.evaluate(source.as_str())?
                } else {
                    kernel.commit_bridged(source.as_str(), calls.as_slice())?
                };
                if evaluation_final_class(&evaluation) != final_value_class {
                    return Err(SteelKernelJournalError::ReplayMismatch);
                }
            }
        }
    }
    Ok(())
}

fn module_to_wire(
    module: &animus_adapter_rlm_kernel::SteelSkillModuleDescriptor,
) -> SkillModuleWire {
    SkillModuleWire {
        name: String::from(module.name()),
        version: String::from(module.version()),
        manifest_identity: String::from(module.manifest_identity()),
        frozen_source_identity: String::from(module.frozen_source_identity()),
        source_identity: String::from(module.source_identity()),
        source: String::from(module.source()),
        exports: module.exports().to_vec(),
        dependencies: module
            .dependencies()
            .iter()
            .map(|dependency| SkillDependencyWire {
                name: String::from(dependency.name()),
                version: String::from(dependency.version()),
                module_identity: String::from(dependency.module_identity()),
            })
            .collect(),
        capability_requests: module.capability_requests().to_vec(),
        source_chars: module.bounds().source_chars,
        evaluation_fuel: module.bounds().evaluation_fuel,
        module_identity: String::from(module.module_identity()),
    }
}

fn restore_modules(
    kernel: &mut SteelRlmKernel,
    expected_module_set_identity: &str,
    modules: &[SkillModuleWire],
) -> Result<(), SteelKernelJournalError> {
    let plan = parse_module_plan(expected_module_set_identity, modules)?;
    kernel.load_skill_modules(&plan)?;
    Ok(())
}

fn validate_modules(
    expected_module_set_identity: &str,
    modules: &[SkillModuleWire],
) -> Result<(), SteelKernelJournalError> {
    let _plan = parse_module_plan(expected_module_set_identity, modules)?;
    Ok(())
}

fn parse_module_plan(
    expected_module_set_identity: &str,
    modules: &[SkillModuleWire],
) -> Result<animus_adapter_rlm_kernel::SteelSkillModulePlan, SteelKernelJournalError> {
    let mut descriptors = Vec::with_capacity(modules.len());
    for module in modules {
        let dependency_drafts = module
            .dependencies
            .iter()
            .map(
                |dependency| animus_adapter_rlm_kernel::SteelSkillModuleDependencyDraft {
                    name: dependency.name.as_str(),
                    version: dependency.version.as_str(),
                    module_identity: dependency.module_identity.as_str(),
                },
            )
            .collect::<Vec<_>>();
        let exports = module
            .exports
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>();
        let capabilities = module
            .capability_requests
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>();
        let descriptor = animus_adapter_rlm_kernel::SteelSkillModuleDescriptor::validate(
            &animus_adapter_rlm_kernel::SteelSkillModuleDescriptorDraft {
                schema: animus_adapter_rlm_kernel::STEEL_SKILL_MODULE_SCHEMA,
                name: module.name.as_str(),
                version: module.version.as_str(),
                manifest_identity: module.manifest_identity.as_str(),
                frozen_source_identity: module.frozen_source_identity.as_str(),
                source_identity: module.source_identity.as_str(),
                source: module.source.as_str(),
                exports: exports.as_slice(),
                dependencies: dependency_drafts.as_slice(),
                capability_requests: capabilities.as_slice(),
                trust: animus_adapter_rlm_kernel::SteelSkillModuleTrust::Trusted,
                bounds: animus_adapter_rlm_kernel::SteelSkillModuleBounds {
                    source_chars: module.source_chars,
                    evaluation_fuel: module.evaluation_fuel,
                },
            },
        )
        .map_err(|_error| SteelKernelJournalError::ChainCorrupt)?;
        if descriptor.module_identity() != module.module_identity {
            return Err(SteelKernelJournalError::ChainCorrupt);
        }
        descriptors.push(descriptor);
    }
    let capabilities = descriptors
        .iter()
        .flat_map(animus_adapter_rlm_kernel::SteelSkillModuleDescriptor::capability_requests)
        .map(String::as_str)
        .collect::<Vec<_>>();
    let authority = animus_adapter_rlm_kernel::SteelSkillModuleAuthority {
        session: capabilities.as_slice(),
        profile: capabilities.as_slice(),
        skill: capabilities.as_slice(),
        host: capabilities.as_slice(),
    };
    let plan =
        animus_adapter_rlm_kernel::resolve_steel_skill_modules(descriptors.as_slice(), authority)
            .map_err(|_error| SteelKernelJournalError::ChainCorrupt)?;
    if plan.module_set_identity() != expected_module_set_identity {
        return Err(SteelKernelJournalError::ChainCorrupt);
    }
    Ok(plan)
}

fn restore_calls(
    calls: &[CallWire],
    wire: &JournalWire,
    policy: &animus_adapter_rlm_kernel::RlmKernelPolicy,
) -> Result<Vec<animus_adapter_rlm_kernel::BridgeCompletedCall>, SteelKernelJournalError> {
    let mut restored = Vec::with_capacity(calls.len());
    for (index, call) in calls.iter().enumerate() {
        let expected_sequence = u32::try_from(index)
            .map_err(|_error| SteelKernelJournalError::CapturedResultMismatch)?;
        if call.sequence != expected_sequence {
            return Err(SteelKernelJournalError::CapturedResultMismatch);
        }
        let operation = parse_operation(call.operation.as_str())
            .ok_or(SteelKernelJournalError::CapturedResultMismatch)?;
        let cell = animus_adapter_rlm_kernel::BridgeCellIdentity::new(
            wire.session_identity.as_str(),
            call.cell_identity.as_str(),
        )
        .map_err(|_error| SteelKernelJournalError::CapturedResultMismatch)?;
        let completed = animus_adapter_rlm_kernel::BridgeCompletedCall::from_replay_parts(
            animus_adapter_rlm_kernel::BridgeReplayCallDraft {
                cell: &cell,
                sequence: call.sequence,
                call_identity: call.call_identity.as_str(),
                operation,
                request_identity: call.request_identity.as_str(),
                result_value: call.result_value.as_str(),
                result_identity: call.result_identity.as_str(),
            },
            policy,
        )
        .map_err(|_error| SteelKernelJournalError::CapturedResultMismatch)?;
        restored.push(completed);
    }
    Ok(restored)
}

fn validate_admission(
    wire: &JournalWire,
    policy: &animus_adapter_rlm_kernel::RlmKernelPolicy,
    admission: SteelKernelRestoreAdmission<'_>,
) -> Result<(), SteelKernelJournalError> {
    validate_blake3(admission.session_identity)
        .map_err(|()| SteelKernelJournalError::SessionMismatch)?;
    validate_blake3(admission.committed_head_identity)
        .map_err(|()| SteelKernelJournalError::HeadMismatch)?;
    if wire.session_identity != admission.session_identity {
        return Err(SteelKernelJournalError::SessionMismatch);
    }
    if wire.kernel_generation != admission.kernel_generation {
        return Err(SteelKernelJournalError::GenerationMismatch);
    }
    if wire.steel_cohort != STEEL_KERNEL_COHORT
        || wire.rlm_cohort != policy.cohort()
        || wire.rlm_cohort_revision != policy.cohort_revision()
    {
        return Err(SteelKernelJournalError::CohortMismatch);
    }
    if wire.policy_identity != policy.identity() {
        return Err(SteelKernelJournalError::PolicyMismatch);
    }
    if wire.bridge_schema != STEEL_KERNEL_BRIDGE_SCHEMA {
        return Err(SteelKernelJournalError::BridgeSchemaMismatch);
    }
    if wire.committed_head_identity != admission.committed_head_identity {
        return Err(SteelKernelJournalError::HeadMismatch);
    }
    Ok(())
}

fn validate_wire(
    wire: &JournalWire,
    policy: &animus_adapter_rlm_kernel::RlmKernelPolicy,
) -> Result<(), SteelKernelJournalError> {
    if wire.schema != STEEL_KERNEL_JOURNAL_SCHEMA {
        return Err(SteelKernelJournalError::UnsupportedSchema);
    }
    if wire.records.len() > MAX_PRIVATE_RECORDS {
        return Err(SteelKernelJournalError::BoundExceeded);
    }
    validate_blake3(wire.session_identity.as_str())
        .map_err(|()| SteelKernelJournalError::SessionMismatch)?;
    validate_blake3(wire.policy_identity.as_str())
        .map_err(|()| SteelKernelJournalError::PolicyMismatch)?;
    validate_blake3(wire.committed_head_identity.as_str())
        .map_err(|()| SteelKernelJournalError::HeadMismatch)?;
    let mut prior = STEEL_KERNEL_EMPTY_HEAD;
    for (index, record) in wire.records.iter().enumerate() {
        let expected_sequence =
            u32::try_from(index).map_err(|_error| SteelKernelJournalError::BoundExceeded)?;
        if record.sequence != expected_sequence || record.prior_record_identity != prior {
            return Err(SteelKernelJournalError::ChainCorrupt);
        }
        let body_identity = record_body_identity(&record.body)?;
        let record_identity = record_identity(record.sequence, prior, body_identity.as_str());
        if record.body_identity != body_identity || record.record_identity != record_identity {
            return Err(SteelKernelJournalError::ChainCorrupt);
        }
        validate_body(&record.body, wire, policy)?;
        prior = record.record_identity.as_str();
    }
    let expected_head = wire
        .records
        .last()
        .map_or(STEEL_KERNEL_EMPTY_HEAD, |record| {
            record.record_identity.as_str()
        });
    if wire.committed_head_identity != expected_head {
        return Err(SteelKernelJournalError::HeadMismatch);
    }
    Ok(())
}

fn validate_body(
    body: &RecordBodyWire,
    wire: &JournalWire,
    policy: &animus_adapter_rlm_kernel::RlmKernelPolicy,
) -> Result<(), SteelKernelJournalError> {
    match body {
        RecordBodyWire::Clear { clear_epoch } if *clear_epoch == 0 => {
            Err(SteelKernelJournalError::ChainCorrupt)
        }
        RecordBodyWire::Clear { .. } => Ok(()),
        RecordBodyWire::Context { bindings } => {
            if bindings.is_empty()
                || bindings.len()
                    > usize::try_from(policy.bounds().max_bindings).unwrap_or(usize::MAX)
            {
                return Err(SteelKernelJournalError::BoundExceeded);
            }
            for binding in bindings {
                let parsed = animus_adapter_rlm_kernel::ContextBinding::new(
                    binding.name.as_str(),
                    binding.value.as_str(),
                    policy.bounds().max_environment_chars,
                )
                .map_err(|_error| SteelKernelJournalError::ContextMismatch)?;
                if parsed.value_hash() != binding.value_identity {
                    return Err(SteelKernelJournalError::ContextMismatch);
                }
            }
            Ok(())
        }
        RecordBodyWire::Modules {
            module_set_identity,
            modules,
        } => validate_modules(module_set_identity.as_str(), modules.as_slice()),
        RecordBodyWire::Cell {
            source,
            source_identity: expected_source,
            final_value_class,
            captured_calls,
        } => {
            let source_chars = u32::try_from(source.chars().count()).unwrap_or(u32::MAX);
            if source.is_empty()
                || source_chars > policy.bounds().max_source_chars
                || source_identity(source.as_str()) != *expected_source
                || !matches!(
                    final_value_class.as_str(),
                    VALUE_FINAL_CLASS | HOST_REQUEST_FINAL_CLASS
                )
                || captured_calls.len()
                    > usize::try_from(policy.bounds().max_host_calls_per_cell).unwrap_or(usize::MAX)
            {
                return Err(SteelKernelJournalError::BoundExceeded);
            }
            let _calls = restore_calls(captured_calls, wire, policy)?;
            Ok(())
        }
    }
}

fn chain_records(bodies: &[RecordBodyWire]) -> Result<Vec<RecordWire>, SteelKernelJournalError> {
    let mut records = Vec::with_capacity(bodies.len());
    let mut prior = String::from(STEEL_KERNEL_EMPTY_HEAD);
    let mut sequence = FIRST_RECORD_SEQUENCE;
    for body in bodies {
        let body_identity = record_body_identity(body)?;
        let identity = record_identity(sequence, prior.as_str(), body_identity.as_str());
        records.push(RecordWire {
            sequence,
            prior_record_identity: prior,
            body_identity,
            record_identity: identity.clone(),
            body: body.clone(),
        });
        prior = identity;
        sequence = sequence
            .checked_add(NEXT_RECORD_INCREMENT)
            .ok_or(SteelKernelJournalError::BoundExceeded)?;
    }
    Ok(records)
}

fn record_body_identity(body: &RecordBodyWire) -> Result<String, SteelKernelJournalError> {
    let bytes =
        serde_json::to_vec(body).map_err(|_error| SteelKernelJournalError::InvalidEncoding)?;
    Ok(hash_bytes(JOURNAL_BODY_HASH_DOMAIN, bytes.as_slice()))
}

fn record_identity(sequence: u32, prior: &str, body_identity: &str) -> String {
    hash_parts(
        JOURNAL_RECORD_HASH_DOMAIN,
        &[sequence.to_string().as_str(), prior, body_identity],
    )
}

fn source_identity(source: &str) -> String {
    hash_parts(SOURCE_HASH_DOMAIN, &[source])
}

fn evaluation_final_class(evaluation: &SteelRlmEvaluation) -> &'static str {
    match evaluation {
        SteelRlmEvaluation::Value { .. } => VALUE_FINAL_CLASS,
        SteelRlmEvaluation::HostRequest { .. } => HOST_REQUEST_FINAL_CLASS,
    }
}

fn clear_epoch(records: &[RecordWire]) -> u64 {
    records.iter().fold(0, |epoch, record| match &record.body {
        RecordBodyWire::Clear { clear_epoch } => *clear_epoch,
        _other => epoch,
    })
}

fn parse_operation(value: &str) -> Option<animus_adapter_rlm_kernel::HostOperationClass> {
    match value {
        "file_read" => Some(animus_adapter_rlm_kernel::HostOperationClass::FileRead),
        "file_edit" => Some(animus_adapter_rlm_kernel::HostOperationClass::FileEdit),
        "shell" => Some(animus_adapter_rlm_kernel::HostOperationClass::Shell),
        "tool" => Some(animus_adapter_rlm_kernel::HostOperationClass::Tool),
        "subagent" => Some(animus_adapter_rlm_kernel::HostOperationClass::Subagent),
        "context_read" => Some(animus_adapter_rlm_kernel::HostOperationClass::ContextRead),
        "kernel_clear" => Some(animus_adapter_rlm_kernel::HostOperationClass::KernelClear),
        _other => None,
    }
}

fn project_restore_receipt(
    wire: &JournalWire,
    status: SteelKernelRestoreStatus,
    sensitive: SteelKernelRestoreSensitiveDraft<'_>,
) -> Result<SteelKernelRestoreReceipt, SteelKernelJournalError> {
    if sensitive.source.is_some()
        || sensitive.result.is_some()
        || sensitive.prompt.is_some()
        || sensitive.credential.is_some()
        || sensitive.private_path.is_some()
        || sensitive.raw_diagnostic.is_some()
    {
        return Err(SteelKernelJournalError::ReceiptLeak);
    }
    let record_count = u32::try_from(wire.records.len())
        .map_err(|_error| SteelKernelJournalError::BoundExceeded)?;
    let cell_count = count_records(wire, |body| matches!(body, RecordBodyWire::Cell { .. }))?;
    let binding_count = wire.records.iter().try_fold(0_u32, |total, record| {
        let count = match &record.body {
            RecordBodyWire::Context { bindings } => u32::try_from(bindings.len())
                .map_err(|_error| SteelKernelJournalError::BoundExceeded)?,
            _other => 0,
        };
        total
            .checked_add(count)
            .ok_or(SteelKernelJournalError::BoundExceeded)
    })?;
    let captured_result_count = wire.records.iter().try_fold(0_u32, |total, record| {
        let count = match &record.body {
            RecordBodyWire::Cell { captured_calls, .. } => u32::try_from(captured_calls.len())
                .map_err(|_error| SteelKernelJournalError::BoundExceeded)?,
            _other => 0,
        };
        total
            .checked_add(count)
            .ok_or(SteelKernelJournalError::BoundExceeded)
    })?;
    let (module_count, module_set_identity) = wire.records.iter().try_fold(
        (0_u32, None),
        |state, record| -> Result<(u32, Option<String>), SteelKernelJournalError> {
            match &record.body {
                RecordBodyWire::Modules {
                    module_set_identity,
                    modules,
                } => {
                    let count = u32::try_from(modules.len())
                        .map_err(|_error| SteelKernelJournalError::BoundExceeded)?;
                    Ok((
                        state.0.saturating_add(count),
                        Some(module_set_identity.clone()),
                    ))
                }
                _other => Ok(state),
            }
        },
    )?;
    let generation = wire.kernel_generation.to_string();
    let records = record_count.to_string();
    let cells = cell_count.to_string();
    let bindings = binding_count.to_string();
    let results = captured_result_count.to_string();
    let modules = module_count.to_string();
    let module_set = module_set_identity.as_deref().unwrap_or("none");
    let receipt_identity = hash_parts(
        JOURNAL_RECEIPT_HASH_DOMAIN,
        &[
            status.as_str(),
            wire.session_identity.as_str(),
            generation.as_str(),
            wire.policy_identity.as_str(),
            wire.steel_cohort.as_str(),
            wire.bridge_schema.as_str(),
            wire.committed_head_identity.as_str(),
            records.as_str(),
            cells.as_str(),
            bindings.as_str(),
            results.as_str(),
            modules.as_str(),
            module_set,
            RESTORE_NON_CLAIM,
        ],
    );
    Ok(SteelKernelRestoreReceipt {
        schema: STEEL_KERNEL_JOURNAL_SCHEMA,
        status,
        session_identity: wire.session_identity.clone(),
        kernel_generation: wire.kernel_generation,
        policy_identity: wire.policy_identity.clone(),
        steel_cohort: STEEL_KERNEL_COHORT,
        bridge_schema: STEEL_KERNEL_BRIDGE_SCHEMA,
        committed_head_identity: wire.committed_head_identity.clone(),
        record_count,
        cell_count,
        binding_count,
        captured_result_count,
        module_count,
        module_set_identity,
        redaction_marker: REDACTION_MARKER,
        non_claim: RESTORE_NON_CLAIM,
        receipt_identity,
    })
}

fn count_records(
    wire: &JournalWire,
    predicate: impl Fn(&RecordBodyWire) -> bool,
) -> Result<u32, SteelKernelJournalError> {
    let count = wire
        .records
        .iter()
        .filter(|record| predicate(&record.body))
        .count();
    u32::try_from(count).map_err(|_error| SteelKernelJournalError::BoundExceeded)
}

fn validate_blake3(value: &str) -> Result<(), ()> {
    if value.len() == BLAKE3_HEX_CHARS
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || matches!(byte, b'a'..=b'f'))
    {
        Ok(())
    } else {
        Err(())
    }
}

fn hash_parts(domain: &str, parts: &[&str]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain.as_bytes());
    for part in parts {
        hasher.update(&u64::try_from(part.len()).unwrap_or(u64::MAX).to_le_bytes());
        hasher.update(part.as_bytes());
    }
    hasher.finalize().to_hex().to_string()
}

fn hash_bytes(domain: &str, bytes: &[u8]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain.as_bytes());
    hasher.update(&u64::try_from(bytes.len()).unwrap_or(u64::MAX).to_le_bytes());
    hasher.update(bytes);
    hasher.finalize().to_hex().to_string()
}
