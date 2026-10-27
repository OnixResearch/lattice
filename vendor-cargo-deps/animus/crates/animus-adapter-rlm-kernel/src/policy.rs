use alloc::{string::String, vec::Vec};

const GIT_REVISION_HEX_CHARS: usize = 40;
const POLICY_IDENTITY_DOMAIN: &str = "animus.rlm-kernel-policy.v1";
const MAX_COHORT_LABEL_CHARS: u32 = 128;
const MAX_PROFILE_LABEL_CHARS: u32 = 128;
const MAX_OPERATION_CLASSES: usize = 8;
const MAX_CHILD_PROFILES: usize = 32;
const MAX_NON_CLAIMS: usize = 16;

/// Stable RLM kernel policy schema.
pub const RLM_KERNEL_POLICY_SCHEMA: &str = "animus.rlm-kernel-policy.v1";

/// Required claim boundaries for every RLM kernel policy and receipt.
pub const REQUIRED_NON_CLAIMS: &[&str] = &[
    "security_sandboxing",
    "model_correctness",
    "task_completion",
    "production_readiness",
    "release_eligibility",
];

/// Named finite resource bounds for one model-facing kernel.
#[allow(clippy::struct_field_names)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResourceBounds {
    /// Maximum Steel source characters in one evaluation.
    pub max_source_chars: u32,
    /// Maximum retained environment characters.
    pub max_environment_chars: u32,
    /// Maximum prompt bindings retained by the kernel.
    pub max_bindings: u32,
    /// Maximum committed helper-definition cells.
    pub max_definitions: u32,
    /// Maximum deterministic evaluation fuel units reported by the evaluator.
    pub max_evaluation_fuel: u64,
    /// Maximum structural recursion depth reported before evaluation.
    pub max_recursion_depth: u32,
    /// Maximum evaluator allocation budget in bytes.
    pub max_allocation_bytes: u64,
    /// Maximum characters in one typed host request.
    pub max_host_request_chars: u32,
    /// Maximum recursive child depth.
    pub max_child_depth: u32,
    /// Maximum child fan-out from one parent.
    pub max_child_fan_out: u32,
    /// Maximum child token budget.
    pub max_child_tokens: u64,
    /// Maximum child wall-clock budget in milliseconds.
    pub max_child_wall_time_ms: u64,
    /// Maximum model-visible child result characters.
    pub max_child_result_chars: u32,
    /// Maximum admitted host calls in one bridged cell.
    pub max_host_calls_per_cell: u32,
    /// Maximum characters in one bridged host result.
    pub max_host_result_chars: u32,
    /// Maximum cumulative host-result characters in one bridged cell.
    pub max_cumulative_host_result_chars: u32,
    /// Maximum nested bridge depth.
    pub max_bridge_nesting: u32,
    /// Maximum cumulative deterministic evaluator fuel for one bridged cell.
    pub max_bridge_evaluation_fuel: u64,
    /// Maximum elapsed wall time for one bridged cell.
    pub max_bridge_wall_time_ms: u64,
}

impl ResourceBounds {
    fn validate(self) -> Result<(), crate::Error> {
        let bounds = [
            ("max_source_chars", u64::from(self.max_source_chars)),
            (
                "max_environment_chars",
                u64::from(self.max_environment_chars),
            ),
            ("max_bindings", u64::from(self.max_bindings)),
            ("max_definitions", u64::from(self.max_definitions)),
            ("max_evaluation_fuel", self.max_evaluation_fuel),
            ("max_recursion_depth", u64::from(self.max_recursion_depth)),
            ("max_allocation_bytes", self.max_allocation_bytes),
            (
                "max_host_request_chars",
                u64::from(self.max_host_request_chars),
            ),
            ("max_child_depth", u64::from(self.max_child_depth)),
            ("max_child_fan_out", u64::from(self.max_child_fan_out)),
            ("max_child_tokens", self.max_child_tokens),
            ("max_child_wall_time_ms", self.max_child_wall_time_ms),
            (
                "max_child_result_chars",
                u64::from(self.max_child_result_chars),
            ),
            (
                "max_host_calls_per_cell",
                u64::from(self.max_host_calls_per_cell),
            ),
            (
                "max_host_result_chars",
                u64::from(self.max_host_result_chars),
            ),
            (
                "max_cumulative_host_result_chars",
                u64::from(self.max_cumulative_host_result_chars),
            ),
            ("max_bridge_nesting", u64::from(self.max_bridge_nesting)),
            (
                "max_bridge_evaluation_fuel",
                self.max_bridge_evaluation_fuel,
            ),
            ("max_bridge_wall_time_ms", self.max_bridge_wall_time_ms),
        ];
        for (field, value) in bounds {
            crate::ensure_positive(field, value)?;
        }
        Ok(())
    }
}

/// Borrowed human-authored RLM kernel policy before Rust validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RlmKernelPolicyDraft<'a> {
    /// Exact policy schema.
    pub schema: &'a str,
    /// Safe reviewed cohort label.
    pub cohort: &'a str,
    /// Exact reviewed Prime Agent Git revision.
    pub cohort_revision: &'a str,
    /// Safe profile label.
    pub profile: &'a str,
    /// Finite resource bounds.
    pub bounds: ResourceBounds,
    /// Explicit host operation classes admitted by the profile.
    pub admitted_operations: &'a [crate::HostOperationClass],
    /// Explicit child profiles admitted for `rlm(...)`.
    pub admitted_child_profiles: &'a [&'a str],
    /// Required claim boundaries.
    pub non_claims: &'a [&'a str],
}

/// Validated deterministic RLM kernel policy.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RlmKernelPolicy {
    cohort: String,
    cohort_revision: String,
    profile: String,
    bounds: ResourceBounds,
    admitted_operations: Vec<crate::HostOperationClass>,
    admitted_child_profiles: Vec<String>,
    non_claims: Vec<String>,
    identity: String,
}

impl RlmKernelPolicy {
    /// Validates and canonicalizes a human-authored policy.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error`] for schema drift, malformed identities, missing
    /// bounds, duplicates, omitted non-claims, or unsafe operation defaults.
    pub fn from_draft(draft: &RlmKernelPolicyDraft<'_>) -> Result<Self, crate::Error> {
        if draft.schema != RLM_KERNEL_POLICY_SCHEMA {
            return Err(crate::Error::EmptyField { field: "schema" });
        }
        draft.bounds.validate()?;
        let cohort = crate::required_text("cohort", draft.cohort, MAX_COHORT_LABEL_CHARS)?;
        if !crate::is_lower_hex(draft.cohort_revision, GIT_REVISION_HEX_CHARS) {
            return Err(crate::Error::InvalidCohortRevision);
        }
        let profile = crate::required_text("profile", draft.profile, MAX_PROFILE_LABEL_CHARS)?;
        let admitted_operations = canonical_operations(draft.admitted_operations)?;
        let admitted_child_profiles = canonical_labels(
            "admitted_child_profiles",
            draft.admitted_child_profiles,
            MAX_CHILD_PROFILES,
        )?;
        let non_claims = canonical_labels("non_claims", draft.non_claims, MAX_NON_CLAIMS)?;
        for required in REQUIRED_NON_CLAIMS {
            if !non_claims.iter().any(|value| value == required) {
                return Err(crate::Error::MissingNonClaim {
                    non_claim: required,
                });
            }
        }
        let identity = policy_identity(&IdentityInputs {
            cohort: &cohort,
            revision: draft.cohort_revision,
            profile: &profile,
            bounds: draft.bounds,
            operations: &admitted_operations,
            child_profiles: &admitted_child_profiles,
            non_claims: &non_claims,
        });
        Ok(Self {
            cohort,
            cohort_revision: String::from(draft.cohort_revision),
            profile,
            bounds: draft.bounds,
            admitted_operations,
            admitted_child_profiles,
            non_claims,
            identity,
        })
    }

    /// Returns the safe cohort label.
    #[must_use]
    pub fn cohort(&self) -> &str {
        self.cohort.as_str()
    }

    /// Returns the exact reviewed cohort revision.
    #[must_use]
    pub fn cohort_revision(&self) -> &str {
        self.cohort_revision.as_str()
    }

    /// Returns the safe profile label.
    #[must_use]
    pub fn profile(&self) -> &str {
        self.profile.as_str()
    }

    /// Returns the admitted resource bounds.
    #[must_use]
    pub const fn bounds(&self) -> ResourceBounds {
        self.bounds
    }

    /// Returns canonical admitted operation classes.
    #[must_use]
    pub fn admitted_operations(&self) -> &[crate::HostOperationClass] {
        self.admitted_operations.as_slice()
    }

    /// Returns canonical admitted child profiles.
    #[must_use]
    pub fn admitted_child_profiles(&self) -> &[String] {
        self.admitted_child_profiles.as_slice()
    }

    /// Returns canonical required non-claims.
    #[must_use]
    pub fn non_claims(&self) -> &[String] {
        self.non_claims.as_slice()
    }

    /// Returns the domain-separated BLAKE3 policy identity.
    #[must_use]
    pub fn identity(&self) -> &str {
        self.identity.as_str()
    }

    /// Returns true when the policy admits an operation class.
    #[must_use]
    pub fn admits_operation(&self, operation: crate::HostOperationClass) -> bool {
        self.admitted_operations.binary_search(&operation).is_ok()
    }

    /// Returns true when the policy admits a child profile.
    #[must_use]
    pub fn admits_child_profile(&self, profile: &str) -> bool {
        self.admitted_child_profiles
            .binary_search_by(|candidate| candidate.as_str().cmp(profile))
            .is_ok()
    }
}

fn canonical_operations(
    values: &[crate::HostOperationClass],
) -> Result<Vec<crate::HostOperationClass>, crate::Error> {
    if values.is_empty() {
        return Err(crate::Error::EmptyField {
            field: "admitted_operations",
        });
    }
    crate::ensure_at_most(
        "admitted_operations",
        crate::usize_to_u64(values.len()),
        crate::usize_to_u64(MAX_OPERATION_CLASSES),
    )?;
    let mut canonical = values.to_vec();
    canonical.sort();
    let original_len = canonical.len();
    canonical.dedup();
    if canonical.len() != original_len {
        return Err(crate::Error::DuplicateIdentity {
            field: "admitted_operations",
        });
    }
    Ok(canonical)
}

fn canonical_labels(
    field: &'static str,
    values: &[&str],
    max_items: usize,
) -> Result<Vec<String>, crate::Error> {
    if values.is_empty() {
        return Err(crate::Error::EmptyField { field });
    }
    crate::ensure_at_most(
        field,
        crate::usize_to_u64(values.len()),
        crate::usize_to_u64(max_items),
    )?;
    let mut canonical = values
        .iter()
        .map(|value| crate::required_text(field, value, MAX_PROFILE_LABEL_CHARS))
        .collect::<Result<Vec<_>, _>>()?;
    canonical.sort();
    let original_len = canonical.len();
    canonical.dedup();
    if canonical.len() != original_len {
        return Err(crate::Error::DuplicateIdentity { field });
    }
    Ok(canonical)
}

struct IdentityInputs<'a> {
    cohort: &'a str,
    revision: &'a str,
    profile: &'a str,
    bounds: ResourceBounds,
    operations: &'a [crate::HostOperationClass],
    child_profiles: &'a [String],
    non_claims: &'a [String],
}

fn policy_identity(inputs: &IdentityInputs<'_>) -> String {
    let bounds = inputs.bounds;
    let bounds_text = alloc::format!(
        "{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}:{}",
        bounds.max_source_chars,
        bounds.max_environment_chars,
        bounds.max_bindings,
        bounds.max_definitions,
        bounds.max_evaluation_fuel,
        bounds.max_recursion_depth,
        bounds.max_allocation_bytes,
        bounds.max_host_request_chars,
        bounds.max_child_depth,
        bounds.max_child_fan_out,
        bounds.max_child_tokens,
        bounds.max_child_wall_time_ms,
        bounds.max_child_result_chars,
        bounds.max_host_calls_per_cell,
        bounds.max_host_result_chars,
        bounds.max_cumulative_host_result_chars,
        bounds.max_bridge_nesting,
        bounds.max_bridge_evaluation_fuel,
        bounds.max_bridge_wall_time_ms,
    );
    let operation_text = inputs
        .operations
        .iter()
        .map(|operation| operation.as_str())
        .collect::<Vec<_>>()
        .join(",");
    let child_text = inputs.child_profiles.join(",");
    let non_claim_text = inputs.non_claims.join(",");
    crate::hash_parts(
        POLICY_IDENTITY_DOMAIN,
        &[
            inputs.cohort,
            inputs.revision,
            inputs.profile,
            bounds_text.as_str(),
            operation_text.as_str(),
            child_text.as_str(),
            non_claim_text.as_str(),
        ],
    )
}
