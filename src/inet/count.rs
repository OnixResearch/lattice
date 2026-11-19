//! Optional semantic interaction count, recomputation integrity and the exact
//! semantic-limit consequence (Lattice #22).
//!
//! One receipt can at most bind its counter to the producer's named model
//! theorems and proof-source identities; it stays `Observed`. The count
//! becomes `Checked` only when two fresh, admitted, complete evaluations of one
//! immutable computation agree on output and count under that same binding.
//! `Checked` is census-checked recomputation consistency under model-only
//! theorems; it is not a refinement proof of the producer's Rust reducer.

/// The semantic unit the qualified theorems count.
pub const UNIT: &str = "inets-principal-pair-interaction";
/// The producer counter that reports it.
pub const COUNTER: &str = "noble-net-reducer-interactions";
/// The observation boundary of the count.
pub const BOUNDARY: &str = "complete-normal-form-and-ordered-read-back";
/// The producer's census formula, checked per run against the counter.
pub const CENSUS: &str = "2*ops+dups+eras";
/// The only per-row basis a bridge-bound count accepts.
pub const COUNT_BASIS: &str = "census-checked";
/// The producer's declared assurance class for the theorems.
pub const MODEL_ASSURANCE: &str = "model-only";
/// The instantiated Lean interaction system.
pub const LEAN_SYSTEM: &str = "NobleFoundation.NetI64.system";
/// The exact theorem declarations the bridge names.
pub const THEOREMS: [&str; 3] = [
    "NobleFoundation.NetI64.i64_normal_form_unique",
    "NobleFoundation.NetI64.i64_equal_interaction_count",
    "NobleFoundation.NetI64.Certificate.accepted_exact_count",
];
/// The exact proof-source identities the qualified producer reports.
pub const PROOF_SOURCE: [(&str, &str); 11] = [
    ("module", "proofs/foundation/NobleFoundation/NetI64.lean"),
    ("module_blake3", "a5d9ade10d41b2c0a7e2d209ce2f162918c7250805a13187c3428be357c5dc74"),
    ("module_sha256", "c7706a132361014ee1b5eec5c724fe95d4a81b1a7024bc6d16e519f8c00b86cb"),
    ("lean_toolchain", "leanprover/lean4:v4.35.0"),
    ("inets_module", "proofs/inets/InteractionNets.lean"),
    ("inets_commit", "98bbb01070664e4462a35efa5d62de6f27143572"),
    ("inets_blake3", "26f32cef1dae84c10443b7f42efe3ad78bb2fca2e4cfdb9c456fe7c49ec58d22"),
    ("inets_sha256", "c7a35dad49822583a9ee209b47d8ba5617d5abcb5a7f9f96df05e8502f362f54"),
    ("receipt", "verification/foundation/receipts/518be7fd5/foundation-check.json"),
    ("receipt_blake3", "7f76a66321c597c8ac7a9aebf83a525a1d73bda73ca85f5298a0c29e843f4b1a"),
    ("receipt_source_commit", "518be7fd5691ac6925c99f2cae3b592ab8243ad1"),
];
const MODEL_BOUNDARY_PREFIX: &str = "model-only:";
const UNQUALIFIED_UNIT: &str = "unqualified-producer-counter";

/// Assurance of a semantic count.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Assurance {
    Observed,
    Checked,
}

/// Version 1 of the optional `semantic_interactions` observation.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Interactions {
    pub version: String,
    pub value: String,
    pub unit: String,
    pub boundary: String,
    pub counter: String,
    pub assurance: Assurance,
    /// The named theorems and proof sources; absent when the receipt names none.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theorems: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proof_source: Option<std::collections::BTreeMap<String, String>>,
}

impl Interactions {
    /// True when the receipt bound this counter to the qualified theorems.
    pub fn is_bridge_bound(&self) -> bool {
        self.theorems.is_some() && self.proof_source.is_some()
    }
}

/// Admit one receipt's semantic count. A receipt without theorems stays
/// observed only; a receipt that names theorems must match every pinned
/// unit, census, proof and per-run census binding or fail admission.
pub fn admit(
    semantic_count: Option<&serde_json::Value>,
    counters: &super::report::Counters,
) -> Result<Interactions, super::Error> {
    let declared = semantic_count.unwrap_or(&serde_json::Value::Null);
    let has_theorems = declared["theorems"].as_array().is_some_and(|theorems| !theorems.is_empty());
    let text = |key: &str, fallback: &str| declared[key].as_str().unwrap_or(fallback).to_string();
    let mut observed = Interactions {
        version: "1".to_string(),
        value: counters.interactions.to_string(),
        unit: text("unit", UNQUALIFIED_UNIT),
        boundary: text("boundary", UNQUALIFIED_UNIT),
        counter: text("counter", UNQUALIFIED_UNIT),
        assurance: Assurance::Observed,
        theorems: None,
        proof_source: None,
    };
    if !has_theorems {
        return Ok(observed);
    }
    let theorems = THEOREMS.iter().map(|theorem| serde_json::json!(theorem)).collect::<Vec<_>>();
    let proof = PROOF_SOURCE.iter().map(|(key, value)| ((*key).to_string(), serde_json::json!(value))).collect();
    let is_unit_exact = declared["unit"] == UNIT && declared["counter"] == COUNTER && declared["boundary"] == BOUNDARY;
    let is_basis_exact = declared["census"] == CENSUS
        && declared["count_basis"] == COUNT_BASIS
        && declared["assurance"] == MODEL_ASSURANCE
        && declared["model_boundary"]
            .as_str()
            .is_some_and(|boundary| boundary.starts_with(MODEL_BOUNDARY_PREFIX));
    let is_proof_exact = declared["theorems"] == serde_json::Value::Array(theorems)
        && declared["lean_system"] == LEAN_SYSTEM
        && declared["proof_source"] == serde_json::Value::Object(proof);
    let is_census_exact = counters.interactions == counters.expected_interactions;
    if !(is_unit_exact && is_basis_exact && is_proof_exact && is_census_exact) {
        return Err(super::Error::CrossedCountBinding);
    }
    observed.theorems = Some(THEOREMS.iter().map(|theorem| (*theorem).to_string()).collect());
    observed.proof_source =
        Some(PROOF_SOURCE.iter().map(|(key, value)| ((*key).to_string(), (*value).to_string())).collect());
    debug_assert!(observed.is_bridge_bound());
    Ok(observed)
}

/// One admitted complete evaluation, as published by the worker route.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Evaluation {
    /// Identity of the immutable computation, excluding attempt and limits.
    pub computation: String,
    pub invocation: String,
    pub attempt: String,
    pub result: i64,
    pub interactions: Interactions,
}

impl Evaluation {
    /// Read a published complete result document; non-authoritative,
    /// incomplete or malformed documents never supply a comparison baseline.
    pub fn from_document(document: &serde_json::Value) -> Result<Self, super::Error> {
        let is_complete = document["evaluation"] == "complete" && document["authoritative"] == true;
        let binding = &document["binding"];
        let text = |value: &serde_json::Value| value.as_str().map(str::to_string);
        let fields = (
            text(&binding["computation"]),
            text(&binding["invocation"]),
            text(&binding["attempt"]),
            serde_json::from_value::<Interactions>(document["semantic_interactions"].clone()).ok(),
        );
        let (true, (Some(computation), Some(invocation), Some(attempt), Some(interactions))) = (is_complete, fields)
        else {
            return Err(super::Error::RecomputationRefused);
        };
        let result = crate::pure_inet_profile::canonical_i64_from_json(&document["result"])?;
        Ok(Self {
            computation,
            invocation,
            attempt,
            result,
            interactions,
        })
    }
}

/// Outcome of comparing two fresh complete evaluations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    /// Output and bridge-bound count agree; the count is now checked.
    Consistent(Interactions),
    /// Output agrees; at least one count is observed only and stays so.
    ConsistentObservedCount,
    OutputIntegrityFailure,
    InteractionCountIntegrityFailure,
}

/// Compare two fresh admitted evaluations of one immutable computation.
pub fn compare(
    first: &Evaluation,
    second: &Evaluation,
    is_checked_count_required: bool,
) -> Result<Verdict, super::Error> {
    let is_fresh = first.attempt != second.attempt && first.invocation != second.invocation;
    if first.computation != second.computation || !is_fresh {
        return Err(super::Error::RecomputationRefused);
    }
    debug_assert!(is_fresh && first.computation == second.computation);
    if first.result != second.result {
        return Ok(Verdict::OutputIntegrityFailure);
    }
    let is_bridge_bound = first.interactions.is_bridge_bound() && second.interactions.is_bridge_bound();
    if !is_bridge_bound {
        return if is_checked_count_required {
            Err(super::Error::CrossedCountBinding)
        } else {
            Ok(Verdict::ConsistentObservedCount)
        };
    }
    let (left, right) = (&first.interactions, &second.interactions);
    let is_binding_equal = left.unit == right.unit
        && left.boundary == right.boundary
        && left.theorems == right.theorems
        && left.proof_source == right.proof_source;
    if !is_binding_equal {
        return Err(super::Error::CrossedCountBinding);
    }
    debug_assert!(left.is_bridge_bound() && right.is_bridge_bound());
    if left.value != right.value {
        return Ok(Verdict::InteractionCountIntegrityFailure);
    }
    Ok(Verdict::Consistent(Interactions {
        assurance: Assurance::Checked,
        ..left.clone()
    }))
}

/// The producer outcome of a later invocation of the same computation.
#[derive(Debug, Clone, Copy)]
pub struct Rerun<'a> {
    pub computation: &'a str,
    pub fuel_interactions: u64,
    pub outcome: &'a super::report::Normalized,
}

/// What an exact semantic limit means for a rerun.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LimitVerdict {
    /// The rerun completed with exactly the checked count.
    CompletedAtCount,
    /// Fuel below the checked count stopped the rerun, as it must.
    ExpectedExhaustion,
    /// Another bound stopped the rerun; no count disagreement is inferred.
    IndependentBound,
    /// The rerun contradicts the checked count.
    InteractionCountIntegrityFailure,
    /// The rerun did not reach completion or exhaustion; its outcome stands.
    Retained,
}

/// Apply a checked count `K` to a rerun charged in the same unit: fuel at
/// or above `K` must complete with `K` interactions.
pub fn limit_consequence(baseline: &Evaluation, rerun: &Rerun<'_>) -> Result<LimitVerdict, super::Error> {
    let checked = baseline.interactions.value.parse::<u64>().ok();
    let is_checked = baseline.interactions.assurance == Assurance::Checked && baseline.interactions.unit == UNIT;
    let (true, Some(checked)) = (is_checked && baseline.computation == rerun.computation, checked) else {
        return Err(super::Error::CrossedCountBinding);
    };
    let verdict = match (&rerun.outcome.completion, &rerun.outcome.exhaustion) {
        (Some(completion), _) if completion.counters.interactions == checked => LimitVerdict::CompletedAtCount,
        (Some(_), _) => LimitVerdict::InteractionCountIntegrityFailure,
        (None, Some(exhaustion)) if exhaustion.limit == "fuel" && rerun.fuel_interactions >= checked => {
            LimitVerdict::InteractionCountIntegrityFailure
        }
        (None, Some(exhaustion)) if exhaustion.limit == "fuel" => LimitVerdict::ExpectedExhaustion,
        (None, Some(_)) => LimitVerdict::IndependentBound,
        (None, None) => LimitVerdict::Retained,
    };
    Ok(verdict)
}
