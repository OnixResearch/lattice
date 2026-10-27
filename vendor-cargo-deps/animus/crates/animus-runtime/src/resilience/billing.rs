//! Pure cooldown and billing-tier provider transition planning.

const FAILOVER_HASH_DOMAIN: &str = "animus.provider-failover-billing.v1";
const HASH_SEPARATOR: &str = "|";
const BLAKE3_HEX_LENGTH: usize = 64;
const MAX_FAILOVER_CANDIDATES: usize = 32;
const TIER_CHANGE_REASON: &str = "automatic-fallback-tier-change";

/// Provider account availability supplied by the host.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProviderAccountAvailability {
    /// The account can receive an automatic request.
    Ready,
    /// The account is in an observed cooldown period.
    Cooldown,
}

/// Declared billing tier for one provider account.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProviderBillingTier {
    /// Subscription-backed usage.
    Subscription,
    /// Metered pay-per-use API credit.
    PayPerUse,
}

impl ProviderBillingTier {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Subscription => "subscription",
            Self::PayPerUse => "pay_per_use",
        }
    }
}

/// Explicit authority for a metered billing transition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MeteredSpendAuthority {
    /// Metered spend is not admitted.
    Denied,
    /// Metered spend is explicitly admitted for this transition.
    Admitted,
}

/// One automatic failover candidate supplied by the host.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProviderFailoverCandidateDraft<'a> {
    /// Exact provider identity.
    pub provider_id: &'a str,
    /// Current observed account availability.
    pub availability: ProviderAccountAvailability,
    /// Declared billing tier.
    pub billing_tier: ProviderBillingTier,
}

/// Complete provider transition planning input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProviderFailoverTransitionDraft<'a> {
    /// Durable session identity.
    pub session_id: &'a str,
    /// Pending turn identity.
    pub pending_turn_id: &'a str,
    /// Exact pending turn BLAKE3 identity.
    pub pending_turn_hash: &'a str,
    /// Exact prior durable session-head BLAKE3 identity.
    pub durable_session_head_hash: &'a str,
    /// Current provider identity.
    pub current_provider_id: &'a str,
    /// Current billing tier.
    pub current_billing_tier: ProviderBillingTier,
    /// Ordered automatic candidates.
    pub candidates: &'a [ProviderFailoverCandidateDraft<'a>],
    /// Exact metered-spend authority.
    pub metered_spend_authority: MeteredSpendAuthority,
}

/// Stable provider failover failure class.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProviderFailoverIssue {
    /// An ID or hash is malformed.
    MalformedIdentity,
    /// The candidate list is empty, overbound, or duplicated.
    InvalidCandidates,
    /// No ready provider remains after cooldown filtering.
    NoReadyProvider,
    /// A pay-per-use transition lacks explicit authority.
    MeteredSpendDenied,
}

/// Safe provider failover failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProviderFailoverError {
    issue: ProviderFailoverIssue,
}

impl ProviderFailoverError {
    /// Returns the stable issue class.
    #[must_use]
    pub const fn issue(self) -> ProviderFailoverIssue {
        self.issue
    }
}

impl core::fmt::Display for ProviderFailoverError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "provider failover rejected: {:?}", self.issue)
    }
}

impl core::error::Error for ProviderFailoverError {}

/// Metadata-only cooldown skip record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderCooldownSkip {
    provider_id: String,
}

impl ProviderCooldownSkip {
    /// Returns the skipped provider identity.
    #[must_use]
    pub fn provider_id(&self) -> &str {
        self.provider_id.as_str()
    }
}

/// Explicit billing-tier transition record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderTierTransition {
    from: ProviderBillingTier,
    to: ProviderBillingTier,
    reason: &'static str,
}

impl ProviderTierTransition {
    /// Returns the prior tier.
    #[must_use]
    pub const fn from(&self) -> ProviderBillingTier {
        self.from
    }

    /// Returns the selected tier.
    #[must_use]
    pub const fn to(&self) -> ProviderBillingTier {
        self.to
    }

    /// Returns the stable transition reason.
    #[must_use]
    pub const fn reason(&self) -> &'static str {
        self.reason
    }
}

/// Admitted provider failover plan preserving the pending turn and session head.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderFailoverPlan {
    session_id: String,
    pending_turn_id: String,
    pending_turn_hash: String,
    durable_session_head_hash: String,
    selected_provider_id: String,
    selected_billing_tier: ProviderBillingTier,
    cooldown_skips: Vec<ProviderCooldownSkip>,
    tier_transition: Option<ProviderTierTransition>,
    plan_hash: String,
}

impl ProviderFailoverPlan {
    /// Returns the durable session identity.
    #[must_use]
    pub fn session_id(&self) -> &str {
        self.session_id.as_str()
    }

    /// Returns the pending turn identity.
    #[must_use]
    pub fn pending_turn_id(&self) -> &str {
        self.pending_turn_id.as_str()
    }

    /// Returns the pending turn content identity.
    #[must_use]
    pub fn pending_turn_hash(&self) -> &str {
        self.pending_turn_hash.as_str()
    }

    /// Returns the prior durable session-head identity.
    #[must_use]
    pub fn durable_session_head_hash(&self) -> &str {
        self.durable_session_head_hash.as_str()
    }

    /// Returns the selected provider identity.
    #[must_use]
    pub fn selected_provider_id(&self) -> &str {
        self.selected_provider_id.as_str()
    }

    /// Returns the selected billing tier.
    #[must_use]
    pub const fn selected_billing_tier(&self) -> ProviderBillingTier {
        self.selected_billing_tier
    }

    /// Returns ordered cooldown skips.
    #[must_use]
    pub fn cooldown_skips(&self) -> &[ProviderCooldownSkip] {
        self.cooldown_skips.as_slice()
    }

    /// Returns the optional explicit tier transition.
    #[must_use]
    pub const fn tier_transition(&self) -> Option<&ProviderTierTransition> {
        self.tier_transition.as_ref()
    }

    /// Returns the deterministic transition plan identity.
    #[must_use]
    pub fn plan_hash(&self) -> &str {
        self.plan_hash.as_str()
    }
}

/// Plans one cooldown-aware and fail-closed provider transition.
///
/// # Errors
///
/// Returns [`ProviderFailoverError`] for malformed identities, invalid
/// candidates, exhausted ready providers, or an unapproved metered transition.
///
/// # Panics
///
/// Panics if validated planning fails to preserve required identities or emits
/// an empty deterministic plan identity.
// r[impl animus.billing.failover]
// r[impl animus.billing.tier]
// r[impl animus.billing.pending]
pub fn plan_provider_failover(
    draft: &ProviderFailoverTransitionDraft<'_>,
) -> Result<ProviderFailoverPlan, ProviderFailoverError> {
    validate_label(draft.session_id)?;
    validate_label(draft.pending_turn_id)?;
    validate_label(draft.current_provider_id)?;
    validate_hash(draft.pending_turn_hash)?;
    validate_hash(draft.durable_session_head_hash)?;
    validate_candidates(draft.candidates)?;

    let mut cooldown_skips = Vec::with_capacity(draft.candidates.len());
    let mut selected = None;
    for candidate in draft.candidates {
        if candidate.availability == ProviderAccountAvailability::Cooldown {
            cooldown_skips.push(ProviderCooldownSkip {
                provider_id: candidate.provider_id.to_string(),
            });
        } else if selected.is_none() {
            selected = Some(*candidate);
        }
    }
    let selected = selected.ok_or(ProviderFailoverError {
        issue: ProviderFailoverIssue::NoReadyProvider,
    })?;
    let tier_transition = if draft.current_billing_tier == ProviderBillingTier::Subscription
        && selected.billing_tier == ProviderBillingTier::PayPerUse
    {
        if draft.metered_spend_authority != MeteredSpendAuthority::Admitted {
            return Err(ProviderFailoverError {
                issue: ProviderFailoverIssue::MeteredSpendDenied,
            });
        }
        Some(ProviderTierTransition {
            from: draft.current_billing_tier,
            to: selected.billing_tier,
            reason: TIER_CHANGE_REASON,
        })
    } else {
        None
    };
    let plan_hash = plan_hash(
        draft,
        selected,
        cooldown_skips.as_slice(),
        tier_transition.as_ref(),
    );
    let plan = ProviderFailoverPlan {
        session_id: draft.session_id.to_string(),
        pending_turn_id: draft.pending_turn_id.to_string(),
        pending_turn_hash: draft.pending_turn_hash.to_string(),
        durable_session_head_hash: draft.durable_session_head_hash.to_string(),
        selected_provider_id: selected.provider_id.to_string(),
        selected_billing_tier: selected.billing_tier,
        cooldown_skips,
        tier_transition,
        plan_hash,
    };
    assert_eq!(plan.pending_turn_hash(), draft.pending_turn_hash);
    assert_eq!(
        plan.durable_session_head_hash(),
        draft.durable_session_head_hash
    );
    assert!(!plan.plan_hash().is_empty());
    Ok(plan)
}

fn validate_candidates(
    candidates: &[ProviderFailoverCandidateDraft<'_>],
) -> Result<(), ProviderFailoverError> {
    if candidates.is_empty() || candidates.len() > MAX_FAILOVER_CANDIDATES {
        return Err(ProviderFailoverError {
            issue: ProviderFailoverIssue::InvalidCandidates,
        });
    }
    for (index, candidate) in candidates.iter().enumerate() {
        validate_label(candidate.provider_id)?;
        if candidates
            .iter()
            .skip(index.saturating_add(1))
            .any(|other| other.provider_id == candidate.provider_id)
        {
            return Err(ProviderFailoverError {
                issue: ProviderFailoverIssue::InvalidCandidates,
            });
        }
    }
    assert!(!candidates.is_empty());
    assert!(candidates.len() <= MAX_FAILOVER_CANDIDATES);
    Ok(())
}

fn validate_label(value: &str) -> Result<(), ProviderFailoverError> {
    let is_safe = !value.is_empty()
        && value.len() <= 128
        && value.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.' | ':')
        });
    if is_safe {
        Ok(())
    } else {
        Err(ProviderFailoverError {
            issue: ProviderFailoverIssue::MalformedIdentity,
        })
    }
}

fn validate_hash(value: &str) -> Result<(), ProviderFailoverError> {
    if value.len() == BLAKE3_HEX_LENGTH
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        Ok(())
    } else {
        Err(ProviderFailoverError {
            issue: ProviderFailoverIssue::MalformedIdentity,
        })
    }
}

fn plan_hash(
    draft: &ProviderFailoverTransitionDraft<'_>,
    selected: ProviderFailoverCandidateDraft<'_>,
    skips: &[ProviderCooldownSkip],
    tier_transition: Option<&ProviderTierTransition>,
) -> String {
    assert!(!draft.session_id.is_empty());
    assert!(!selected.provider_id.is_empty());
    let mut hasher = blake3::Hasher::new();
    for part in [
        FAILOVER_HASH_DOMAIN,
        draft.session_id,
        draft.pending_turn_id,
        draft.pending_turn_hash,
        draft.durable_session_head_hash,
        draft.current_provider_id,
        draft.current_billing_tier.as_str(),
        selected.provider_id,
        selected.billing_tier.as_str(),
        tier_transition.map_or("none", ProviderTierTransition::reason),
    ] {
        hasher.update(HASH_SEPARATOR.as_bytes());
        hasher.update(part.as_bytes());
    }
    for skip in skips {
        hasher.update(HASH_SEPARATOR.as_bytes());
        hasher.update(skip.provider_id().as_bytes());
    }
    let hash = hasher.finalize().to_hex().to_string();
    assert!(!hash.is_empty());
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    const HASH_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const HASH_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

    fn draft<'a>(
        candidates: &'a [ProviderFailoverCandidateDraft<'a>],
        authority: MeteredSpendAuthority,
    ) -> ProviderFailoverTransitionDraft<'a> {
        ProviderFailoverTransitionDraft {
            session_id: "session-1",
            pending_turn_id: "turn-7",
            pending_turn_hash: HASH_A,
            durable_session_head_hash: HASH_B,
            current_provider_id: "subscription-provider",
            current_billing_tier: ProviderBillingTier::Subscription,
            candidates,
            metered_spend_authority: authority,
        }
    }

    #[test]
    fn cooldown_skip_and_recorded_tier_transition_preserve_pending_state() {
        // r[verify animus.billing.failover.cooldown_skip]
        // r[verify animus.billing.tier.recorded]
        // r[verify animus.billing.pending.survives]
        // r[verify animus.billing.verification]
        // r[verify animus.billing.verification.matrix]
        let candidates = [
            ProviderFailoverCandidateDraft {
                provider_id: "cooldown-provider",
                availability: ProviderAccountAvailability::Cooldown,
                billing_tier: ProviderBillingTier::Subscription,
            },
            ProviderFailoverCandidateDraft {
                provider_id: "metered-provider",
                availability: ProviderAccountAvailability::Ready,
                billing_tier: ProviderBillingTier::PayPerUse,
            },
        ];
        let plan = plan_provider_failover(&draft(
            candidates.as_slice(),
            MeteredSpendAuthority::Admitted,
        ))
        .expect("admitted failover");

        assert_eq!(plan.selected_provider_id(), "metered-provider");
        assert_eq!(plan.cooldown_skips()[0].provider_id(), "cooldown-provider");
        assert_eq!(plan.pending_turn_hash(), HASH_A);
        assert_eq!(plan.durable_session_head_hash(), HASH_B);
        let tier = plan.tier_transition().expect("tier transition");
        assert_eq!(tier.from(), ProviderBillingTier::Subscription);
        assert_eq!(tier.to(), ProviderBillingTier::PayPerUse);
        assert_eq!(tier.reason(), TIER_CHANGE_REASON);
    }

    #[test]
    fn cooldown_selection_silent_tier_and_turn_drift_fail_closed() {
        // r[verify animus.billing.failover.cooldown_selected]
        // r[verify animus.billing.tier.silent]
        // r[verify animus.billing.pending.loss]
        let cooldown_only = [ProviderFailoverCandidateDraft {
            provider_id: "cooldown-provider",
            availability: ProviderAccountAvailability::Cooldown,
            billing_tier: ProviderBillingTier::Subscription,
        }];
        let no_ready = plan_provider_failover(&draft(
            cooldown_only.as_slice(),
            MeteredSpendAuthority::Denied,
        ))
        .expect_err("cooldown-only candidates");
        assert_eq!(no_ready.issue(), ProviderFailoverIssue::NoReadyProvider);

        let metered = [ProviderFailoverCandidateDraft {
            provider_id: "metered-provider",
            availability: ProviderAccountAvailability::Ready,
            billing_tier: ProviderBillingTier::PayPerUse,
        }];
        let denied =
            plan_provider_failover(&draft(metered.as_slice(), MeteredSpendAuthority::Denied))
                .expect_err("silent metered transition");
        assert_eq!(denied.issue(), ProviderFailoverIssue::MeteredSpendDenied);

        let mut malformed = draft(metered.as_slice(), MeteredSpendAuthority::Admitted);
        malformed.pending_turn_hash = "changed-turn";
        let turn_error = plan_provider_failover(&malformed).expect_err("turn identity drift");
        assert_eq!(turn_error.issue(), ProviderFailoverIssue::MalformedIdentity);
    }
}
