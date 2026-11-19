//! Optional version-bound rollout facts and requirements.
//!
//! Ordinary completion candidates never acquire these requirements. A caller
//! that requires a rollout profile names the exact base model, tokenizer,
//! adapter, policy generation, probability semantics, and operations.

/// Probability semantics a rollout consumer depends on.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProbabilitySemantics {
    /// No probabilities are returned.
    Absent,
    /// Log-probabilities of the sampled tokens.
    SampledTokenLogprobs,
    /// Full next-token distributions.
    FullDistribution,
}

impl ProbabilitySemantics {
    /// Stable label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Absent => "absent",
            Self::SampledTokenLogprobs => "sampled-token-logprobs",
            Self::FullDistribution => "full-distribution",
        }
    }
}

/// A provider operation a rollout consumer requires.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RolloutOperation {
    /// Return log-probabilities for generated tokens.
    TokenLogprobs,
    /// Return log-probabilities for prompt tokens.
    PromptLogprobs,
    /// Honour a caller-supplied sampling seed.
    SeededSampling,
    /// Route the request to a named adapter.
    AdapterRouting,
}

impl RolloutOperation {
    /// Stable label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::TokenLogprobs => "token-logprobs",
            Self::PromptLogprobs => "prompt-logprobs",
            Self::SeededSampling => "seeded-sampling",
            Self::AdapterRouting => "adapter-routing",
        }
    }
}

/// The rollout fact that failed to match a requirement.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RolloutField {
    /// Base model identity.
    BaseModel,
    /// Tokenizer identity.
    Tokenizer,
    /// Adapter artifact identity.
    Adapter,
    /// Loaded policy generation.
    PolicyGeneration,
    /// Probability semantics.
    ProbabilitySemantics,
    /// Required operations.
    Operations,
    /// Qualification no longer matches the serving configuration.
    QualificationConfiguration,
    /// Policy activation was only claimed, not observed.
    Activation,
    /// Request exceeds the qualified envelope.
    QualifiedEnvelope,
}

impl RolloutField {
    /// Stable label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::BaseModel => "base-model",
            Self::Tokenizer => "tokenizer",
            Self::Adapter => "adapter",
            Self::PolicyGeneration => "policy-generation",
            Self::ProbabilitySemantics => "probability-semantics",
            Self::Operations => "operations",
            Self::QualificationConfiguration => "qualification-configuration",
            Self::Activation => "activation",
            Self::QualifiedEnvelope => "qualified-envelope",
        }
    }
}

/// Borrowed rollout facts supplied by the snapshot source.
#[derive(Clone, Copy, Debug)]
pub struct RolloutDraft<'a> {
    /// Qualification evidence reference.
    pub qualification_ref: &'a str,
    /// Backend the qualification tested.
    pub qualified_backend_ref: &'a str,
    /// Serving configuration the qualification tested.
    pub qualified_configuration_ref: &'a str,
    /// Base-model artifact identity.
    pub base_model_ref: &'a str,
    /// Tokenizer identity.
    pub tokenizer_ref: &'a str,
    /// Loaded adapter artifact identity, if any.
    pub adapter_ref: Option<&'a str>,
    /// Loaded policy generation (> 0).
    pub loaded_policy_generation: u64,
    /// Whether activation of that generation was observed or only claimed.
    pub activation: super::Attestation,
    /// Probability semantics returned.
    pub probability_semantics: ProbabilitySemantics,
    /// Supported operations.
    pub operations: &'a [RolloutOperation],
    /// Request envelope the qualification covers.
    pub qualified_envelope: super::Envelope,
}

/// Validated rollout facts.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RolloutFacts {
    qualification_ref: super::ContentRef,
    qualified_backend_ref: super::ContentRef,
    qualified_configuration_ref: super::ContentRef,
    artifacts: Artifacts,
    activation: super::Attestation,
    operations: alloc::vec::Vec<RolloutOperation>,
    qualified_envelope: super::Envelope,
    identity: super::ContentRef,
}

/// Base model, tokenizer, adapter, generation, and semantics shared by facts and requirements.
#[derive(Clone, Debug, Eq, PartialEq)]
struct Artifacts {
    base_model_ref: super::ContentRef,
    tokenizer_ref: super::ContentRef,
    adapter_ref: Option<super::ContentRef>,
    policy_generation: u64,
    probability_semantics: ProbabilitySemantics,
}

impl Artifacts {
    fn new(
        refs: (&str, &str, Option<&str>),
        policy_generation: u64,
        probability_semantics: ProbabilitySemantics,
    ) -> Result<Self, super::ContractIssue> {
        Ok(Self {
            base_model_ref: super::ContentRef::new("rollout.base_model_ref", refs.0)?,
            tokenizer_ref: super::ContentRef::new("rollout.tokenizer_ref", refs.1)?,
            adapter_ref: optional_ref("rollout.adapter_ref", refs.2)?,
            policy_generation: super::basis::positive(
                "rollout.policy_generation",
                policy_generation,
            )?,
            probability_semantics,
        })
    }

    fn mismatch(&self, facts: &Self) -> Option<RolloutField> {
        if facts.base_model_ref != self.base_model_ref {
            return Some(RolloutField::BaseModel);
        }
        if facts.tokenizer_ref != self.tokenizer_ref {
            return Some(RolloutField::Tokenizer);
        }
        if facts.adapter_ref != self.adapter_ref {
            return Some(RolloutField::Adapter);
        }
        if facts.policy_generation != self.policy_generation {
            return Some(RolloutField::PolicyGeneration);
        }
        if facts.probability_semantics != self.probability_semantics {
            return Some(RolloutField::ProbabilitySemantics);
        }
        None
    }

    fn frame(&self, framer: &mut super::basis::Framer) {
        framer.text(self.base_model_ref.as_str());
        framer.text(self.tokenizer_ref.as_str());
        framer.optional(self.adapter_ref.as_ref().map(super::ContentRef::as_str));
        framer.number(self.policy_generation);
        framer.text(self.probability_semantics.as_str());
    }
}

impl RolloutFacts {
    pub(super) fn new(draft: &RolloutDraft<'_>) -> Result<Self, super::ContractIssue> {
        let artifacts = Artifacts::new(
            (draft.base_model_ref, draft.tokenizer_ref, draft.adapter_ref),
            draft.loaded_policy_generation,
            draft.probability_semantics,
        )?;
        let operations = collect_operations(draft.operations)?;
        debug_assert!(artifacts.policy_generation > 0);
        debug_assert!(operations.len() <= draft.operations.len());
        let qualification_ref =
            super::ContentRef::new("rollout.qualification_ref", draft.qualification_ref)?;
        let qualified_backend_ref =
            super::ContentRef::new("rollout.qualified_backend_ref", draft.qualified_backend_ref)?;
        let qualified_configuration_ref = super::ContentRef::new(
            "rollout.qualified_configuration_ref",
            draft.qualified_configuration_ref,
        )?;
        let mut framer = super::basis::Framer::new(ROLLOUT_FACTS_DOMAIN);
        framer.text(qualification_ref.as_str());
        framer.text(qualified_backend_ref.as_str());
        framer.text(qualified_configuration_ref.as_str());
        artifacts.frame(&mut framer);
        framer.text(draft.activation.as_str());
        frame_tail(&mut framer, &operations, draft.qualified_envelope);
        Ok(Self {
            qualification_ref,
            qualified_backend_ref,
            qualified_configuration_ref,
            artifacts,
            activation: draft.activation,
            operations,
            qualified_envelope: draft.qualified_envelope,
            identity: framer.finish(),
        })
    }

    pub(super) fn identity(&self) -> &str {
        self.identity.as_str()
    }

    /// Loaded policy generation.
    #[must_use]
    pub const fn loaded_policy_generation(&self) -> u64 {
        self.artifacts.policy_generation
    }

    /// Qualification evidence reference.
    #[must_use]
    pub fn qualification_ref(&self) -> &str {
        self.qualification_ref.as_str()
    }

    /// Backend the qualification tested.
    #[must_use]
    pub fn qualified_backend_ref(&self) -> &str {
        self.qualified_backend_ref.as_str()
    }
}

const ROLLOUT_FACTS_DOMAIN: &str = "animus-serving-rollout-facts-v1";

fn frame_tail(
    framer: &mut super::basis::Framer,
    operations: &[RolloutOperation],
    envelope: super::Envelope,
) {
    for operation in operations {
        framer.text(operation.as_str());
    }
    framer.number(u64::from(envelope.max_context_tokens()));
    framer.number(u64::from(envelope.max_output_tokens()));
}

/// Borrowed rollout requirement named by a caller.
#[derive(Clone, Copy, Debug)]
pub struct RolloutRequirementDraft<'a> {
    /// Required base-model artifact identity.
    pub base_model_ref: &'a str,
    /// Required tokenizer identity.
    pub tokenizer_ref: &'a str,
    /// Required adapter artifact identity, if any.
    pub adapter_ref: Option<&'a str>,
    /// Required policy generation (> 0).
    pub policy_generation: u64,
    /// Required probability semantics.
    pub probability_semantics: ProbabilitySemantics,
    /// Required operations.
    pub operations: &'a [RolloutOperation],
}

/// Validated rollout requirement.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RolloutRequirement {
    artifacts: Artifacts,
    operations: alloc::vec::Vec<RolloutOperation>,
}

impl RolloutRequirement {
    /// Validates a rollout requirement.
    ///
    /// # Errors
    ///
    /// Returns [`super::ContractIssue`] for noncanonical references, a zero
    /// generation, or duplicate or excessive operations.
    pub fn new(draft: &RolloutRequirementDraft<'_>) -> Result<Self, super::ContractIssue> {
        Ok(Self {
            artifacts: Artifacts::new(
                (draft.base_model_ref, draft.tokenizer_ref, draft.adapter_ref),
                draft.policy_generation,
                draft.probability_semantics,
            )?,
            operations: collect_operations(draft.operations)?,
        })
    }

    /// Required policy generation.
    #[must_use]
    pub const fn policy_generation(&self) -> u64 {
        self.artifacts.policy_generation
    }

    /// Checks facts for `(configuration_ref, (input, output) tokens)`.
    /// `Err(None)` means no rollout facts exist.
    pub(super) fn check(
        &self,
        facts: Option<&RolloutFacts>,
        request: (&str, (u32, u32)),
    ) -> Result<(), Option<RolloutField>> {
        let Some(facts) = facts else {
            return Err(None);
        };
        if let Some(field) = self.artifacts.mismatch(&facts.artifacts) {
            return Err(Some(field));
        }
        if facts.qualified_configuration_ref.as_str() != request.0 {
            return Err(Some(RolloutField::QualificationConfiguration));
        }
        if facts.activation != super::Attestation::Observed {
            return Err(Some(RolloutField::Activation));
        }
        let has_every_operation = self
            .operations
            .iter()
            .all(|operation| facts.operations.contains(operation));
        if !has_every_operation {
            return Err(Some(RolloutField::Operations));
        }
        if facts.qualified_envelope.admits(request.1) {
            return Ok(());
        }
        Err(Some(RolloutField::QualifiedEnvelope))
    }
}

fn optional_ref(
    field: &'static str,
    value: Option<&str>,
) -> Result<Option<super::ContentRef>, super::ContractIssue> {
    match value {
        None => Ok(None),
        Some(text) => Ok(Some(super::ContentRef::new(field, text)?)),
    }
}

fn collect_operations(
    operations: &[RolloutOperation],
) -> Result<alloc::vec::Vec<RolloutOperation>, super::ContractIssue> {
    super::basis::bounded_len(
        "rollout.operations",
        operations.len(),
        super::MAX_ROLLOUT_OPERATIONS,
    )?;
    let mut collected = alloc::vec::Vec::with_capacity(operations.len());
    for operation in operations {
        if collected.contains(operation) {
            return Err(super::ContractIssue::Duplicate {
                field: "rollout.operations",
            });
        }
        collected.push(*operation);
    }
    Ok(collected)
}
