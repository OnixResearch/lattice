use alloc::string::String;
use alloc::vec::Vec;

const MAX_BINDING_NAME_CHARS: u32 = 128;
const EVALUATION_IDENTITY_DOMAIN: &str = "animus.rlm-kernel-evaluation.v1";

/// One named prompt-context value retained across kernel turns.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContextBinding {
    name: String,
    value: String,
    value_hash: String,
}

impl ContextBinding {
    /// Builds a validated Steel binding.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error`] when the name is unsafe or the value violates
    /// the supplied character bound.
    pub fn new(name: &str, value: &str, max_value_chars: u32) -> Result<Self, crate::Error> {
        let name = crate::required_text("binding_name", name, MAX_BINDING_NAME_CHARS)?;
        if !is_safe_identifier(name.as_str()) {
            return Err(crate::Error::InvalidBindingName);
        }
        let value = crate::required_text("binding_value", value, max_value_chars)?;
        let value_hash = crate::hash_parts("animus.rlm-context-value.v1", &[value.as_str()]);
        Ok(Self {
            name,
            value,
            value_hash,
        })
    }

    /// Returns the safe Steel identifier.
    #[must_use]
    pub fn name(&self) -> &str {
        self.name.as_str()
    }

    /// Returns the in-memory binding value for the evaluator.
    #[must_use]
    pub fn value(&self) -> &str {
        self.value.as_str()
    }

    /// Returns the domain-separated BLAKE3 value identity.
    #[must_use]
    pub fn value_hash(&self) -> &str {
        self.value_hash.as_str()
    }

    fn character_count(&self) -> usize {
        self.name.chars().count().saturating_add(self.value.chars().count())
    }
}

/// Metadata for one committed helper-definition cell.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DefinitionCommit {
    source_hash: String,
    source_chars: u32,
}

impl DefinitionCommit {
    /// Builds a definition commit from a source identity and character count.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error`] when the hash is malformed or the count is zero.
    pub fn new(source_hash: &str, source_chars: u32) -> Result<Self, crate::Error> {
        if !crate::is_lower_hex(source_hash, 64) {
            return Err(crate::Error::InvalidBlake3Identity {
                field: "definition_source_hash",
            });
        }
        crate::ensure_positive("definition_source_chars", u64::from(source_chars))?;
        Ok(Self {
            source_hash: String::from(source_hash),
            source_chars,
        })
    }

    /// Returns the definition source hash.
    #[must_use]
    pub fn source_hash(&self) -> &str {
        self.source_hash.as_str()
    }

    /// Returns the definition source character count.
    #[must_use]
    pub const fn source_chars(&self) -> u32 {
        self.source_chars
    }
}

/// Pure persistent kernel environment state.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct EnvironmentState {
    generation: u64,
    bindings: Vec<ContextBinding>,
    definitions: Vec<DefinitionCommit>,
    retained_chars: u32,
}

impl EnvironmentState {
    /// Returns the monotonic committed environment generation.
    #[must_use]
    pub const fn generation(&self) -> u64 {
        self.generation
    }

    /// Returns canonical prompt-context bindings.
    #[must_use]
    pub fn bindings(&self) -> &[ContextBinding] {
        self.bindings.as_slice()
    }

    /// Returns committed helper-definition metadata.
    #[must_use]
    pub fn definitions(&self) -> &[DefinitionCommit] {
        self.definitions.as_slice()
    }

    /// Returns the retained environment character count.
    #[must_use]
    pub const fn retained_chars(&self) -> u32 {
        self.retained_chars
    }
}

/// Deterministic evaluator usage facts checked before Steel evaluation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EvaluationMetrics {
    /// Planned evaluator fuel units.
    pub fuel: u64,
    /// Planned structural recursion depth.
    pub recursion_depth: u32,
    /// Planned evaluator allocation bytes.
    pub allocation_bytes: u64,
}

/// Borrowed model-written Steel evaluation request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EvaluationRequest<'a> {
    /// Model-written Steel source.
    pub source: &'a str,
    /// Deterministic usage facts supplied by the evaluator planner.
    pub metrics: EvaluationMetrics,
}

/// Pure admitted evaluation plan before Steel runs.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvaluationPlan {
    source_hash: String,
    source_chars: u32,
    prior_generation: u64,
    projected_environment_chars: u32,
    metrics: EvaluationMetrics,
    policy_identity: String,
}

impl EvaluationPlan {
    /// Returns the model-written source identity without source content.
    #[must_use]
    pub fn source_hash(&self) -> &str {
        self.source_hash.as_str()
    }

    /// Returns the model-written source character count.
    #[must_use]
    pub const fn source_chars(&self) -> u32 {
        self.source_chars
    }

    /// Returns the environment generation used for admission.
    #[must_use]
    pub const fn prior_generation(&self) -> u64 {
        self.prior_generation
    }

    /// Returns the projected retained environment character count.
    #[must_use]
    pub const fn projected_environment_chars(&self) -> u32 {
        self.projected_environment_chars
    }

    /// Returns the admitted deterministic usage facts.
    #[must_use]
    pub const fn metrics(&self) -> EvaluationMetrics {
        self.metrics
    }

    /// Returns the policy identity used for admission.
    #[must_use]
    pub fn policy_identity(&self) -> &str {
        self.policy_identity.as_str()
    }
}

/// Atomically binds prompt context into a cloned environment.
///
/// # Errors
///
/// Returns [`crate::Error`] without changing `current` when names duplicate or
/// the binding count or environment size exceeds policy.
pub fn bind_prompt_context(
    current: &EnvironmentState,
    bindings: &[ContextBinding],
    policy: &crate::RlmKernelPolicy,
) -> Result<EnvironmentState, crate::Error> {
    let mut next = current.clone();
    let mut incoming = bindings.to_vec();
    incoming.sort_by(|left, right| left.name.cmp(&right.name));
    if incoming.windows(2).any(|pair| pair[0].name == pair[1].name) {
        return Err(crate::Error::DuplicateIdentity {
            field: "context_bindings",
        });
    }
    for binding in incoming {
        match next.bindings.binary_search_by(|candidate| candidate.name.cmp(&binding.name)) {
            Ok(index) => next.bindings[index] = binding,
            Err(index) => next.bindings.insert(index, binding),
        }
    }
    crate::ensure_at_most(
        "environment_bindings",
        crate::usize_to_u64(next.bindings.len()),
        u64::from(policy.bounds().max_bindings),
    )?;
    next.retained_chars = retained_char_count(&next);
    crate::ensure_at_most(
        "environment_chars",
        u64::from(next.retained_chars),
        u64::from(policy.bounds().max_environment_chars),
    )?;
    next.generation = next.generation.saturating_add(1);
    Ok(next)
}

/// Plans one bounded model-facing Steel evaluation.
///
/// # Errors
///
/// Returns [`crate::Error`] before evaluation when source, fuel, depth,
/// allocation, definition count, or projected environment size exceeds policy.
pub fn plan_evaluation(
    current: &EnvironmentState,
    request: &EvaluationRequest<'_>,
    policy: &crate::RlmKernelPolicy,
) -> Result<EvaluationPlan, crate::Error> {
    let source = crate::required_text("evaluation_source", request.source, policy.bounds().max_source_chars)?;
    let bounds = policy.bounds();
    crate::ensure_at_most("evaluation_fuel", request.metrics.fuel, bounds.max_evaluation_fuel)?;
    crate::ensure_at_most(
        "recursion_depth",
        u64::from(request.metrics.recursion_depth),
        u64::from(bounds.max_recursion_depth),
    )?;
    crate::ensure_at_most("allocation_bytes", request.metrics.allocation_bytes, bounds.max_allocation_bytes)?;
    let next_definition_count = current.definitions.len().saturating_add(1);
    crate::ensure_at_most(
        "environment_definitions",
        crate::usize_to_u64(next_definition_count),
        u64::from(bounds.max_definitions),
    )?;
    let source_chars = crate::usize_to_u32(source.chars().count());
    let projected_environment_chars = current.retained_chars.saturating_add(source_chars);
    crate::ensure_at_most(
        "environment_chars",
        u64::from(projected_environment_chars),
        u64::from(bounds.max_environment_chars),
    )?;
    let source_hash = crate::hash_parts(EVALUATION_IDENTITY_DOMAIN, &[source.as_str()]);
    Ok(EvaluationPlan {
        source_hash,
        source_chars,
        prior_generation: current.generation,
        projected_environment_chars,
        metrics: request.metrics,
        policy_identity: String::from(policy.identity()),
    })
}

/// Commits successful evaluation metadata to a cloned environment.
///
/// # Errors
///
/// Returns [`crate::Error`] without changing `current` when the plan is stale,
/// malformed, or exceeds current policy.
pub fn commit_evaluation(
    current: &EnvironmentState,
    plan: &EvaluationPlan,
    policy: &crate::RlmKernelPolicy,
) -> Result<EnvironmentState, crate::Error> {
    if plan.prior_generation != current.generation
        || plan.policy_identity != policy.identity()
        || plan.projected_environment_chars != current.retained_chars.saturating_add(plan.source_chars)
    {
        return Err(crate::Error::InconsistentReceipt);
    }
    let definition = DefinitionCommit::new(plan.source_hash(), plan.source_chars)?;
    let mut next = current.clone();
    next.definitions.push(definition);
    crate::ensure_at_most(
        "environment_definitions",
        crate::usize_to_u64(next.definitions.len()),
        u64::from(policy.bounds().max_definitions),
    )?;
    next.retained_chars = retained_char_count(&next);
    crate::ensure_at_most(
        "environment_chars",
        u64::from(next.retained_chars),
        u64::from(policy.bounds().max_environment_chars),
    )?;
    next.generation = next.generation.saturating_add(1);
    Ok(next)
}

/// Cause of a context transition that can interact with kernel state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompactionCause {
    /// Prompt context compaction.
    Compaction,
    /// Persistent-goal state changed.
    GoalChange,
    /// A child completed.
    ChildCompletion,
    /// An explicit host action requested a kernel clear.
    ExplicitHostClear,
}

/// Requested kernel behavior during a context transition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CompactionRequest {
    /// Transition cause.
    pub cause: CompactionCause,
    /// Whether the caller requests a kernel clear.
    pub clear_kernel: bool,
}

/// Pure kernel-aware compaction plan.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct KernelCompactionPlan {
    prior_generation: u64,
    binding_count: u32,
    definition_count: u32,
    kernel_state_preserved: bool,
    explicit_clear: bool,
}

impl KernelCompactionPlan {
    /// Returns the environment generation covered by the plan.
    #[must_use]
    pub const fn prior_generation(&self) -> u64 {
        self.prior_generation
    }

    /// Returns the kernel-held prompt binding count.
    #[must_use]
    pub const fn binding_count(&self) -> u32 {
        self.binding_count
    }

    /// Returns the kernel-held helper-definition count.
    #[must_use]
    pub const fn definition_count(&self) -> u32 {
        self.definition_count
    }

    /// Returns whether compaction preserves kernel state.
    #[must_use]
    pub const fn kernel_state_preserved(&self) -> bool {
        self.kernel_state_preserved
    }

    /// Returns whether an explicit host clear was admitted.
    #[must_use]
    pub const fn explicit_clear(&self) -> bool {
        self.explicit_clear
    }
}

/// Plans compaction or an explicit host clear without mutating the environment.
///
/// # Errors
///
/// Returns [`crate::Error::ImplicitKernelClearDenied`] when compaction, a goal
/// change, or child completion attempts to clear the kernel.
pub fn plan_compaction(
    current: &EnvironmentState,
    request: CompactionRequest,
) -> Result<KernelCompactionPlan, crate::Error> {
    if request.clear_kernel && request.cause != CompactionCause::ExplicitHostClear {
        return Err(crate::Error::ImplicitKernelClearDenied);
    }
    let explicit_clear = request.clear_kernel && request.cause == CompactionCause::ExplicitHostClear;
    Ok(KernelCompactionPlan {
        prior_generation: current.generation,
        binding_count: crate::usize_to_u32(current.bindings.len()),
        definition_count: crate::usize_to_u32(current.definitions.len()),
        kernel_state_preserved: !explicit_clear,
        explicit_clear,
    })
}

fn retained_char_count(state: &EnvironmentState) -> u32 {
    let binding_chars =
        state.bindings.iter().fold(0usize, |total, binding| total.saturating_add(binding.character_count()));
    let definition_chars = state
        .definitions
        .iter()
        .fold(0u32, |total, definition| total.saturating_add(definition.source_chars));
    crate::usize_to_u32(binding_chars).saturating_add(definition_chars)
}

fn is_safe_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first.is_ascii_alphabetic() || matches!(first, '_' | '-' | '?' | '!')) {
        return false;
    }
    chars.all(|character| character.is_ascii_alphanumeric() || matches!(character, '_' | '-' | '?' | '!' | '/'))
}
