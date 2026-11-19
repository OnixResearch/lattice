//! The explicitly enabled capacity-aware selection profile.
//!
//! Routes that are not screened under a profile keep their declared policy.
//! Every profile requires serving-owner request admission before dispatch.

/// How a service's model identity must be established.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IdentityRule {
    /// A source claim is accepted; the receipt records it as claimed.
    ClaimedAccepted,
    /// Only a host-observed identity is accepted.
    ObservedRequired,
}

impl IdentityRule {
    /// Stable label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ClaimedAccepted => "claimed-accepted",
            Self::ObservedRequired => "observed-required",
        }
    }
}

/// What capacity evidence makes a service eligible.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CapacityRule {
    /// A fresh, known, non-exhausted request-gating counter from the serving
    /// owner or provider API is required.
    KnownRequired,
    /// Unknown capacity stays eligible; the owner's admission decides.
    OwnerAdmissionDecides,
}

impl CapacityRule {
    /// Stable label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::KnownRequired => "known-required",
            Self::OwnerAdmissionDecides => "owner-admission-decides",
        }
    }
}

/// Whether a same-model replica change between selection and dispatch is allowed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReplicaRule {
    /// The owner may place the request on another replica of the same model and
    /// configuration; the receipt records the change.
    SameModelAllowed,
    /// The observed replica must not change.
    Pinned,
}

impl ReplicaRule {
    /// Stable label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SameModelAllowed => "same-model-allowed",
            Self::Pinned => "pinned",
        }
    }
}

/// Borrowed profile fields, usually decoded from the generated Nickel profile.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProfileDraft<'a> {
    /// Profile identifier.
    pub profile_id: &'a str,
    /// Model-identity rule.
    pub identity: IdentityRule,
    /// Capacity-evidence rule.
    pub capacity: CapacityRule,
    /// Replica-change rule.
    pub replica: ReplicaRule,
    /// Maximum snapshot age in logical ticks (> 0).
    pub max_snapshot_age_ticks: u64,
}

/// Validated capacity-aware profile with a BLAKE3 identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Profile {
    id: super::Token,
    identity_rule: IdentityRule,
    capacity: CapacityRule,
    replica: ReplicaRule,
    max_snapshot_age_ticks: u64,
    identity: super::ContentRef,
}

impl Profile {
    /// Validates a profile.
    ///
    /// # Errors
    ///
    /// Returns [`super::ContractIssue`] for an unsafe identifier or a zero age bound.
    pub fn new(draft: ProfileDraft<'_>) -> Result<Self, super::ContractIssue> {
        let id = super::Token::new("profile_id", draft.profile_id)?;
        let max_snapshot_age_ticks =
            super::basis::positive("max_snapshot_age_ticks", draft.max_snapshot_age_ticks)?;
        let mut framer = super::basis::Framer::new(super::PROFILE_DOMAIN);
        framer.text(super::PROFILE_SCHEMA);
        framer.text(id.as_str());
        framer.text(draft.identity.as_str());
        framer.text(draft.capacity.as_str());
        framer.text(draft.replica.as_str());
        framer.number(max_snapshot_age_ticks);
        Ok(Self {
            id,
            identity_rule: draft.identity,
            capacity: draft.capacity,
            replica: draft.replica,
            max_snapshot_age_ticks,
            identity: framer.finish(),
        })
    }

    /// BLAKE3 profile identity.
    #[must_use]
    pub fn identity(&self) -> &str {
        self.identity.as_str()
    }

    /// Profile identifier.
    #[must_use]
    pub fn profile_id(&self) -> &str {
        self.id.as_str()
    }

    /// Model-identity rule.
    #[must_use]
    pub const fn identity_rule(&self) -> IdentityRule {
        self.identity_rule
    }

    /// Capacity-evidence rule.
    #[must_use]
    pub const fn capacity_rule(&self) -> CapacityRule {
        self.capacity
    }

    /// Replica-change rule.
    #[must_use]
    pub const fn replica_rule(&self) -> ReplicaRule {
        self.replica
    }

    /// Maximum snapshot age in logical ticks.
    #[must_use]
    pub const fn max_snapshot_age_ticks(&self) -> u64 {
        self.max_snapshot_age_ticks
    }
}
