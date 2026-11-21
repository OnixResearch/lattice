//! Consumer of Kiln #61's admitted recursive budget lineage.
//!
//! Kiln owns the lineage: its root allocation, subtractive transfers to child
//! accounts, revocation, deadlines and serving backpressure. Lattice receives
//! one frozen snapshot of the attempt's account in the request and only ever
//! draws from it. Nothing here mints, refunds or resets quota: every worker
//! step, check and child of one attempt draws from the one account, and a
//! child is admitted only as a subtractive part of its parent's remainder.
//!
//! The Kiln types are mapped, not vendored. The field mapping against Kiln
//! [`KILN_SOURCE_REVISION`] is recorded in `docs/candidate-worker.md`.

// r[impl lattice.implementation_attempt.lineage]

/// Wire schema of the Lattice consumer type.
pub const LINEAGE_SCHEMA: &str = "lattice.kiln-budget-lineage.v1";
/// The Kiln revision whose `fair_admission::lineage` and `serving` types this
/// consumer maps.
pub const KILN_SOURCE_REVISION: &str = "339116f68e762e4caf0d05a9ac73a6e97a74b67d";
/// Kiln's `MAX_LINEAGE_DEPTH`.
pub const MAX_LINEAGE_DEPTH: u32 = 8;
/// Kiln's `MAX_DIMENSIONS` for one `Amounts` map.
pub const MAX_DIMENSIONS: usize = 16;
const MAX_DIMENSION_NAME_BYTES: usize = 64;
/// The dimension Lattice charges for model requests of a worker step.
pub const MODEL_REQUESTS: &str = "model_requests";
/// The dimension Lattice charges for worker tool calls and check executions.
pub const TOOL_INVOCATIONS: &str = "tool_invocations";
const IDENTITY_DOMAIN: &str = "lattice.kiln-budget-lineage.v1 binding identity";

/// Kiln's lineage blocker (`LineageExplanation.blocker`, an `AdmissionReason`).
/// Only the reasons Kiln's `lineage_blocker` returns are representable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Blocker {
    /// A revoked account or ancestor.
    Denied,
    /// An ancestor deadline is reached.
    Expired,
    /// A parent attempt was cancelled or had cancellation requested.
    Cancelled,
    /// The lineage refused the last allowance.
    BudgetExhausted,
}

/// Kiln's serving gate (`ServingExplanation.gate`), or `none` when the work
/// item has no serving owner.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Backpressure {
    None,
    Eligible,
    Waiting,
    Overloaded,
    Stale,
    Incomplete,
    Incompatible,
}

/// The frozen Kiln lineage snapshot of one attempt's account.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Binding {
    pub schema: String,
    /// `LineageExplanation.root`: the root account identity.
    pub root: String,
    /// `LineageExplanation.account`: this attempt's account identity.
    pub account: String,
    /// The account's parent; `None` exactly for the root account.
    pub parent: Option<String>,
    /// `LineageExplanation.depth`.
    pub depth: u32,
    /// The account's available `Amounts` at dispatch, by dimension.
    pub remaining: std::collections::BTreeMap<String, u64>,
    /// `LineageExplanation.blocker`.
    pub blocker: Option<Blocker>,
    /// `ServingExplanation.gate`.
    pub backpressure: Backpressure,
}

/// Conservative amounts already charged against the account, by dimension.
#[derive(Debug, Clone, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Draws {
    pub model_requests: u64,
    pub tool_invocations: u64,
}

impl Draws {
    /// The amount charged in `dimension`; `None` for a dimension Lattice
    /// cannot account, whose usage stays unknown.
    #[must_use]
    pub fn amount(&self, dimension: &str) -> Option<u64> {
        match dimension {
            MODEL_REQUESTS => Some(self.model_requests),
            TOOL_INVOCATIONS => Some(self.tool_invocations),
            _ => None,
        }
    }

    /// Add `other`. An overflowing sum saturates, which only exhausts
    /// every bound sooner.
    #[must_use]
    pub const fn saturating_add(&self, other: &Self) -> Self {
        Self {
            model_requests: self.model_requests.saturating_add(other.model_requests),
            tool_invocations: self.tool_invocations.saturating_add(other.tool_invocations),
        }
    }
}

/// Why the lineage refuses another spawn or a child.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case", tag = "code")]
pub enum Refusal {
    /// Kiln reports a lineage blocker.
    Blocked { blocker: Blocker },
    /// Kiln's serving gate is not eligible.
    Backpressure { gate: Backpressure },
    /// The dimension's remainder cannot cover the next reservation.
    Exhausted { dimension: String },
    /// A child names another root, parent or depth than this account's child.
    CrossedChild,
    /// A child's ceiling exceeds this account's remainder or omits one of its
    /// dimensions: it would be a fresh budget.
    FreshBudget { dimension: String },
}

/// Why a binding is malformed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum BindingError {
    #[error("budget lineage schema is not supported")]
    Schema,
    #[error("budget lineage account identity is not 64 lowercase hexadecimal digits")]
    Identity,
    #[error("budget lineage root, parent and depth disagree")]
    Shape,
    #[error("budget lineage dimensions are empty, unbounded or malformed")]
    Dimensions,
}

impl Binding {
    /// Check the snapshot's structure; it grants nothing.
    pub fn validate(&self) -> Result<(), BindingError> {
        if self.schema != LINEAGE_SCHEMA {
            return Err(BindingError::Schema);
        }
        let parent = self.parent.as_deref();
        let is_identity_valid = [Some(self.root.as_str()), Some(self.account.as_str()), parent]
            .into_iter()
            .flatten()
            .all(super::is_blake3);
        if !is_identity_valid {
            return Err(BindingError::Identity);
        }
        if !self.is_shape_consistent() {
            return Err(BindingError::Shape);
        }
        let is_dimension_valid = |name: &String| {
            !name.is_empty()
                && name.len() <= MAX_DIMENSION_NAME_BYTES
                && name.bytes().all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
        };
        if self.remaining.is_empty()
            || self.remaining.len() > MAX_DIMENSIONS
            || !self.remaining.keys().all(is_dimension_valid)
        {
            return Err(BindingError::Dimensions);
        }
        Ok(())
    }

    fn is_shape_consistent(&self) -> bool {
        let is_root = self.depth == 0;
        match &self.parent {
            None => is_root && self.account == self.root,
            Some(parent) => {
                let is_nested = !is_root && self.depth <= MAX_LINEAGE_DEPTH;
                is_nested && *parent != self.account && self.account != self.root
            }
        }
    }

    /// Domain-separated BLAKE3 identity of the complete snapshot.
    #[must_use]
    pub fn identity(&self) -> String {
        let mut hasher = blake3::Hasher::new_derive_key(IDENTITY_DOMAIN);
        for value in [&self.schema, &self.root, &self.account] {
            frame(&mut hasher, value.as_bytes());
        }
        frame(&mut hasher, self.parent.as_deref().unwrap_or_default().as_bytes());
        hasher.update(&[u8::from(self.parent.is_some())]);
        hasher.update(&self.depth.to_be_bytes());
        frame_count(&mut hasher, self.remaining.len());
        for (dimension, amount) in &self.remaining {
            frame(&mut hasher, dimension.as_bytes());
            hasher.update(&amount.to_be_bytes());
        }
        let blocker = self.blocker.map_or("none", Blocker::wire);
        frame(&mut hasher, blocker.as_bytes());
        frame(&mut hasher, self.backpressure.wire().as_bytes());
        hasher.finalize().to_hex().to_string()
    }

    /// Whether one more step reserving `next` may spawn after `drawn`.
    ///
    /// A blocker or ineligible serving gate refuses first. Every dimension
    /// must keep a positive remainder; a dimension Lattice charges must also
    /// cover `next`. A dimension Lattice cannot charge is never drawn, so its
    /// usage stays unknown, but a zero remainder still refuses.
    pub fn gate(&self, drawn: &Draws, next: &Draws) -> Result<(), Refusal> {
        if let Some(blocker) = self.blocker {
            return Err(Refusal::Blocked { blocker });
        }
        if !matches!(self.backpressure, Backpressure::None | Backpressure::Eligible) {
            return Err(Refusal::Backpressure {
                gate: self.backpressure,
            });
        }
        for (dimension, remaining) in &self.remaining {
            let left = remaining.checked_sub(drawn.amount(dimension).unwrap_or(0));
            let needed = next.amount(dimension).unwrap_or(0).max(1);
            if left.is_none_or(|left| left < needed) {
                return Err(Refusal::Exhausted {
                    dimension: dimension.clone(),
                });
            }
        }
        Ok(())
    }

    /// Admit `child` as part of this account after `drawn`.
    ///
    /// The child must be this account's direct child in the same root, and
    /// its ceiling in every one of this account's dimensions must fit what
    /// is left here. Omitting a dimension or exceeding the remainder would
    /// be a fresh budget and is refused.
    pub fn admit_child(&self, drawn: &Draws, child: &Self) -> Result<(), Refusal> {
        child.validate().map_err(|_| Refusal::CrossedChild)?;
        let is_direct_child = child.root == self.root
            && child.parent.as_deref() == Some(self.account.as_str())
            && self.depth.checked_add(1) == Some(child.depth);
        if !is_direct_child {
            return Err(Refusal::CrossedChild);
        }
        for (dimension, remaining) in &self.remaining {
            let left = remaining.checked_sub(drawn.amount(dimension).unwrap_or(0));
            let ceiling = child.remaining.get(dimension);
            if ceiling.is_none() || left.is_none_or(|left| ceiling.is_some_and(|ceiling| *ceiling > left)) {
                return Err(Refusal::FreshBudget {
                    dimension: dimension.clone(),
                });
            }
        }
        if let Some(extra) = child.remaining.keys().find(|dimension| !self.remaining.contains_key(*dimension)) {
            return Err(Refusal::FreshBudget {
                dimension: extra.clone(),
            });
        }
        Ok(())
    }

    /// Dimensions Lattice cannot charge; their usage stays unknown to Kiln.
    #[must_use]
    pub fn unaccounted(&self) -> Vec<String> {
        self.remaining
            .keys()
            .filter(|dimension| Draws::default().amount(dimension).is_none())
            .cloned()
            .collect()
    }
}

pub(super) fn frame(hasher: &mut blake3::Hasher, bytes: &[u8]) {
    frame_count(hasher, bytes.len());
    hasher.update(bytes);
}

pub(super) fn frame_count(hasher: &mut blake3::Hasher, count: usize) {
    hasher.update(&(count as u64).to_be_bytes());
}

mod wire;

#[cfg(test)]
pub(crate) mod tests;
