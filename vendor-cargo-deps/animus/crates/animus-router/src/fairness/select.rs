use alloc::string::ToString;

/// Pure rotation decision for one role.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RouterRotationDecision {
    role: crate::Role,
    policy: super::RouterRotationPolicy,
    cursor_class: super::RouterRotationCursorClass,
    selected_model: alloc::string::String,
    candidate_count: u32,
    catalog_hash: crate::SafeHash,
    next_cursor: super::RouterRotationCursor,
    next_cursor_hash: alloc::string::String,
}

impl RouterRotationDecision {
    /// Returns the rotated role.
    #[must_use]
    pub const fn role(&self) -> crate::Role {
        self.role
    }

    /// Returns the rotation policy used for the role.
    #[must_use]
    pub const fn policy(&self) -> super::RouterRotationPolicy {
        self.policy
    }

    /// Returns the cursor class used for the decision.
    #[must_use]
    pub const fn cursor_class(&self) -> super::RouterRotationCursorClass {
        self.cursor_class
    }

    /// Returns the selected safe model label.
    #[must_use]
    pub fn selected_model(&self) -> &str {
        self.selected_model.as_str()
    }

    /// Returns the number of eligible candidates after filtering.
    #[must_use]
    pub const fn candidate_count(&self) -> u32 {
        self.candidate_count
    }

    /// Returns the catalog hash used for this decision.
    #[must_use]
    pub fn catalog_hash(&self) -> &str {
        self.catalog_hash.as_str()
    }

    /// Returns the next host cursor metadata.
    #[must_use]
    pub const fn next_cursor(&self) -> &super::RouterRotationCursor {
        &self.next_cursor
    }

    /// Returns the deterministic next-cursor BLAKE3 identity.
    #[must_use]
    pub fn next_cursor_hash(&self) -> &str {
        self.next_cursor_hash.as_str()
    }
}

/// Typed rotation validation error.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RouterRotationError {
    /// Generic adapter contract validation failed.
    Contract(crate::AdapterContractError),
    /// No eligible candidates were supplied to the rotation core.
    NoEligibleCandidates,
    /// Cursor catalog generation mismatched and policy denied reset.
    StaleCursor,
    /// Receipt candidate included unsafe material.
    ReceiptLeak(crate::ReceiptLeak),
}

impl From<crate::AdapterContractError> for RouterRotationError {
    fn from(error: crate::AdapterContractError) -> Self {
        Self::Contract(error)
    }
}

/// Rotates already eligible candidates for one role using explicit host cursor state.
///
/// # Errors
///
/// Returns [`RouterRotationError`] when no eligible candidates exist or stale cursor policy denies reset.
///
/// # Panics
///
/// Panics if the validated candidate list becomes empty after the explicit empty check, or if the
/// snapshot catalog hash is not a BLAKE3-sized safe hash. Both conditions indicate an internal
/// contract violation after admission.
pub fn rotate_eligible_candidates(
    role: crate::Role,
    candidates: &[alloc::string::String],
    snapshot: &super::RouterRotationSnapshot,
) -> Result<RouterRotationDecision, RouterRotationError> {
    if candidates.is_empty() {
        return Err(RouterRotationError::NoEligibleCandidates);
    }
    assert!(
        candidates.len() > super::FIRST_CURSOR_INDEX,
        "candidate list must remain non-empty after validation"
    );
    assert!(
        snapshot.catalog_hash().len() == crate::SafeHash::HEX_CHARS,
        "catalog hash must stay BLAKE3-sized"
    );

    let (selected_index, cursor_class) = cursor_start_index(CursorStartRequest {
        role,
        candidate_count: candidates.len(),
        snapshot,
    })?;
    let selected_model = candidates[selected_index].clone();
    let next_index = next_index(CursorWindow {
        selected_index,
        candidate_count: candidates.len(),
    });
    let next_position = u32::try_from(next_index).unwrap_or(super::MAX_CURSOR_POSITION);
    let next_cursor = super::RouterRotationCursor::from_parts(
        role,
        next_position,
        snapshot.catalog_safe_hash().clone(),
    );
    let next_cursor_hash = cursor_hash(&next_cursor, snapshot.catalog_safe_hash());
    let candidate_count = u32::try_from(candidates.len()).unwrap_or(super::MAX_CURSOR_POSITION);

    Ok(RouterRotationDecision {
        role,
        policy: snapshot.policy(),
        cursor_class,
        selected_model,
        candidate_count,
        catalog_hash: snapshot.catalog_safe_hash().clone(),
        next_cursor,
        next_cursor_hash,
    })
}

#[derive(Clone, Copy)]
struct CursorStartRequest<'a> {
    role: crate::Role,
    candidate_count: usize,
    snapshot: &'a super::RouterRotationSnapshot,
}

fn cursor_start_index(
    request: CursorStartRequest<'_>,
) -> Result<(usize, super::RouterRotationCursorClass), RouterRotationError> {
    assert!(
        request.candidate_count > super::FIRST_CURSOR_INDEX,
        "candidate count must be non-zero"
    );
    assert!(
        request.snapshot.catalog_hash().len() == crate::SafeHash::HEX_CHARS,
        "catalog hash must stay BLAKE3-sized"
    );
    if !request.snapshot.policy().rotates_role(request.role) {
        return Ok((
            super::FIRST_CURSOR_INDEX,
            super::RouterRotationCursorClass::Disabled,
        ));
    }

    let Some(cursor) = request.snapshot.cursor_for(request.role) else {
        return Ok((
            super::FIRST_CURSOR_INDEX,
            super::RouterRotationCursorClass::Absent,
        ));
    };

    if cursor
        .catalog_hash()
        .is_some_and(|hash| hash != request.snapshot.catalog_hash())
    {
        if request.snapshot.stale_cursor_policy() == super::RouterRotationStaleCursorPolicy::Deny {
            return Err(RouterRotationError::StaleCursor);
        }
        return Ok((
            super::FIRST_CURSOR_INDEX,
            super::RouterRotationCursorClass::StaleReset,
        ));
    }

    let raw_index = match usize::try_from(cursor.position()) {
        Ok(index) => index,
        Err(_overflow) => {
            return Ok((
                super::FIRST_CURSOR_INDEX,
                super::RouterRotationCursorClass::Wrapped,
            ));
        }
    };
    if raw_index >= request.candidate_count {
        return Ok((
            wrapped_index(CursorWindow {
                selected_index: raw_index,
                candidate_count: request.candidate_count,
            }),
            super::RouterRotationCursorClass::Wrapped,
        ));
    }

    Ok((raw_index, super::RouterRotationCursorClass::Matched))
}

#[derive(Clone, Copy)]
struct CursorWindow {
    selected_index: usize,
    candidate_count: usize,
}

fn wrapped_index(window: CursorWindow) -> usize {
    let divisor = window.candidate_count;
    assert!(divisor != 0, "divisor must be non-zero");
    assert!(
        window.selected_index >= window.candidate_count,
        "wrapped index must start outside the candidate window"
    );
    window.selected_index % divisor
}

fn next_index(window: CursorWindow) -> usize {
    let divisor = window.candidate_count;
    assert!(divisor != 0, "divisor must be non-zero");
    assert!(
        window.selected_index < window.candidate_count,
        "selected index must be inside the candidate window"
    );
    let advanced_index = match window.selected_index.checked_add(super::CURSOR_ADVANCE) {
        Some(index) => index,
        None => super::FIRST_CURSOR_INDEX,
    };
    advanced_index % divisor
}

fn cursor_hash(
    cursor: &super::RouterRotationCursor,
    catalog_hash: &crate::SafeHash,
) -> alloc::string::String {
    let position = cursor.position().to_string();
    crate::hash_parts(&[
        super::CURSOR_HASH_SCHEMA,
        cursor.role().as_str(),
        position.as_str(),
        catalog_hash.as_str(),
    ])
}
