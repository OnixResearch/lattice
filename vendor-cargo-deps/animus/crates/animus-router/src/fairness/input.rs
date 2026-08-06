/// Router round-robin policy supplied by the host.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RouterRotationPolicy {
    /// Rotation is disabled; catalog order remains the deterministic tie-breaker.
    Disabled,
    /// Rotate primary recipe-role candidates only.
    RoleCandidates,
    /// Rotate primary recipe-role candidates and fallback-chain candidates.
    RoleCandidatesAndFallback,
}

impl RouterRotationPolicy {
    /// Parses a stable router rotation policy label.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] when the label is empty or unsupported.
    pub fn parse(value: &str) -> Result<Self, crate::AdapterContractError> {
        crate::parse_value(super::POLICY_FIELD, value, |normalized| match normalized {
            super::POLICY_DISABLED => Some(Self::Disabled),
            super::POLICY_ROLE_CANDIDATES => Some(Self::RoleCandidates),
            super::POLICY_ROLE_CANDIDATES_AND_FALLBACK => Some(Self::RoleCandidatesAndFallback),
            _unsupported => None,
        })
    }

    /// Returns the stable policy label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Disabled => super::POLICY_DISABLED,
            Self::RoleCandidates => super::POLICY_ROLE_CANDIDATES,
            Self::RoleCandidatesAndFallback => super::POLICY_ROLE_CANDIDATES_AND_FALLBACK,
        }
    }

    pub(crate) const fn rotates_role(self, role: crate::Role) -> bool {
        match (self, role) {
            (Self::Disabled, _) | (Self::RoleCandidates, crate::Role::Fallback) => false,
            (Self::RoleCandidates | Self::RoleCandidatesAndFallback, _) => true,
        }
    }
}

/// Host policy for stale rotation cursors.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RouterRotationStaleCursorPolicy {
    /// Reset stale cursors to the first eligible candidate.
    Reset,
    /// Deny selection when a cursor's catalog hash is stale.
    Deny,
}

impl RouterRotationStaleCursorPolicy {
    /// Parses a stable stale-cursor policy label.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] when the label is empty or unsupported.
    pub fn parse(value: &str) -> Result<Self, crate::AdapterContractError> {
        crate::parse_value(super::STALE_POLICY_FIELD, value, |normalized| match normalized {
            super::STALE_RESET => Some(Self::Reset),
            super::STALE_DENY => Some(Self::Deny),
            _unsupported => None,
        })
    }

    /// Returns the stable stale-cursor policy label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Reset => super::STALE_RESET,
            Self::Deny => super::STALE_DENY,
        }
    }
}

/// Rotation cursor classification recorded in decisions and receipts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RouterRotationCursorClass {
    /// Rotation policy was disabled.
    Disabled,
    /// No matching host cursor was supplied.
    Absent,
    /// A matching host cursor was used directly.
    Matched,
    /// A matching cursor wrapped around the eligible candidate list.
    Wrapped,
    /// A stale cursor was reset by explicit host policy.
    StaleReset,
}

impl RouterRotationCursorClass {
    /// Parses a stable cursor-class label.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] when the label is empty or unsupported.
    pub fn parse(value: &str) -> Result<Self, crate::AdapterContractError> {
        crate::parse_value(super::CURSOR_CLASS_FIELD, value, |normalized| match normalized {
            super::CURSOR_DISABLED => Some(Self::Disabled),
            super::CURSOR_ABSENT => Some(Self::Absent),
            super::CURSOR_MATCHED => Some(Self::Matched),
            super::CURSOR_WRAPPED => Some(Self::Wrapped),
            super::CURSOR_STALE_RESET => Some(Self::StaleReset),
            _unsupported => None,
        })
    }

    /// Returns the stable cursor-class label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Disabled => super::CURSOR_DISABLED,
            Self::Absent => super::CURSOR_ABSENT,
            Self::Matched => super::CURSOR_MATCHED,
            Self::Wrapped => super::CURSOR_WRAPPED,
            Self::StaleReset => super::CURSOR_STALE_RESET,
        }
    }
}

/// Host-owned rotation cursor for one router role.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RouterRotationCursor {
    role: crate::Role,
    position: u32,
    catalog_hash: Option<crate::SafeHash>,
}

impl RouterRotationCursor {
    /// Builds a validated host-owned rotation cursor.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] when role or hash labels are malformed.
    pub fn new(draft: RouterRotationCursorDraft<'_>) -> Result<Self, crate::AdapterContractError> {
        Ok(Self {
            role: crate::Role::parse(draft.role)?,
            position: draft.position,
            catalog_hash: draft.catalog_hash.map(crate::SafeHash::new).transpose()?,
        })
    }

    /// Returns the role this cursor applies to.
    #[must_use]
    pub const fn role(&self) -> crate::Role {
        self.role
    }

    /// Returns the next host cursor position.
    #[must_use]
    pub const fn position(&self) -> u32 {
        self.position
    }

    /// Returns the optional catalog-generation hash bound to the cursor.
    #[must_use]
    pub fn catalog_hash(&self) -> Option<&str> {
        self.catalog_hash.as_ref().map(crate::SafeHash::as_str)
    }

    pub(super) fn from_parts(role: crate::Role, position: u32, catalog_hash: crate::SafeHash) -> Self {
        Self {
            role,
            position,
            catalog_hash: Some(catalog_hash),
        }
    }
}

/// Borrowed fields used to validate a router rotation cursor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RouterRotationCursorDraft<'a> {
    /// Role label for this cursor.
    pub role: &'a str,
    /// Zero-based next candidate position.
    pub position: u32,
    /// Optional BLAKE3 catalog generation hash.
    pub catalog_hash: Option<&'a str>,
}

/// Host-owned rotation snapshot supplied as explicit router input.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RouterRotationSnapshot {
    policy: RouterRotationPolicy,
    stale_cursor_policy: RouterRotationStaleCursorPolicy,
    catalog_hash: crate::SafeHash,
    cursors: alloc::vec::Vec<RouterRotationCursor>,
}

impl RouterRotationSnapshot {
    /// Builds a validated host-owned rotation snapshot.
    ///
    /// # Errors
    ///
    /// Returns [`crate::AdapterContractError`] when labels, hashes, or cursors are malformed.
    pub fn new(draft: RouterRotationSnapshotDraft<'_>) -> Result<Self, crate::AdapterContractError> {
        let cursors = draft
            .cursors
            .iter()
            .map(|cursor| RouterRotationCursor::new(*cursor))
            .collect::<Result<alloc::vec::Vec<_>, _>>()?;
        if draft.policy != super::POLICY_DISABLED && cursors.is_empty() {
            return Err(crate::AdapterContractError::EmptyCollection {
                field: super::EMPTY_CURSORS_FIELD,
            });
        }
        Ok(Self {
            policy: RouterRotationPolicy::parse(draft.policy)?,
            stale_cursor_policy: RouterRotationStaleCursorPolicy::parse(draft.stale_cursor_policy)?,
            catalog_hash: crate::SafeHash::new(draft.catalog_hash)?,
            cursors,
        })
    }

    /// Returns the active rotation policy.
    #[must_use]
    pub const fn policy(&self) -> RouterRotationPolicy {
        self.policy
    }

    /// Returns the active stale-cursor policy.
    #[must_use]
    pub const fn stale_cursor_policy(&self) -> RouterRotationStaleCursorPolicy {
        self.stale_cursor_policy
    }

    /// Returns the catalog-generation hash.
    #[must_use]
    pub fn catalog_hash(&self) -> &str {
        self.catalog_hash.as_str()
    }

    /// Returns the host-owned cursor list.
    #[must_use]
    pub fn cursors(&self) -> &[RouterRotationCursor] {
        self.cursors.as_slice()
    }

    pub(super) const fn catalog_safe_hash(&self) -> &crate::SafeHash {
        &self.catalog_hash
    }

    pub(super) fn cursor_for(&self, role: crate::Role) -> Option<&RouterRotationCursor> {
        self.cursors.iter().find(|cursor| cursor.role() == role)
    }
}

/// Borrowed fields used to validate a router rotation snapshot.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RouterRotationSnapshotDraft<'a> {
    /// Rotation policy label.
    pub policy: &'a str,
    /// Stale cursor policy label.
    pub stale_cursor_policy: &'a str,
    /// BLAKE3 catalog generation hash.
    pub catalog_hash: &'a str,
    /// Host-owned cursors.
    pub cursors: &'a [RouterRotationCursorDraft<'a>],
}
