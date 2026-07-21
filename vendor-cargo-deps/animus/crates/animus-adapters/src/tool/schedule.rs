/// Borrowed tool call mutation target declaration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MutationCallDraft<'a> {
    /// Optional call identity, required by validation.
    pub call_id: Option<&'a str>,
    /// Opaque mutation targets declared by the tool.
    pub mutation_targets: &'a [&'a str],
}

/// Mutation window scheduling status.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MutationWindowStatus {
    /// The call has no target conflict with previous windows.
    Independent,
    /// The call waits for an earlier conflicting mutation window.
    Serialized,
}

/// Planned mutation window for a host-owned coordinator.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MutationWindow {
    call_id: super::Text,
    status: MutationWindowStatus,
    waits_for: Option<super::Text>,
}

impl MutationWindow {
    /// Returns the call identity.
    #[must_use]
    pub fn call_id(&self) -> &str {
        self.call_id.as_str()
    }

    /// Returns the mutation scheduling status.
    #[must_use]
    pub const fn status(&self) -> MutationWindowStatus {
        self.status
    }

    /// Returns the previous conflicting call identity, if serialized.
    #[must_use]
    pub fn waits_for(&self) -> Option<&str> {
        self.waits_for.as_deref()
    }
}

struct MutationCallState {
    call_id: super::Text,
    mutation_targets: super::Collection<super::Text>,
}

/// Plans deterministic mutation windows for a host-owned coordinator.
///
/// # Errors
///
/// Returns [`super::ContractError`] when call identities or target labels are
/// malformed or collection sizes exceed the bounded contract size.
pub fn plan_mutation_windows(
    calls: &[MutationCallDraft<'_>],
) -> Result<alloc::vec::Vec<MutationWindow>, super::ContractError> {
    let mutation_states = normalized_mutation_calls(calls)?;
    let mut windows = alloc::vec::Vec::with_capacity(mutation_states.len());
    for state in &mutation_states {
        let waits_for = latest_conflicting_call(state, &mutation_states, windows.len());
        let window_status = if waits_for.is_some() {
            MutationWindowStatus::Serialized
        } else {
            MutationWindowStatus::Independent
        };
        windows.push(MutationWindow {
            call_id: state.call_id.clone(),
            status: window_status,
            waits_for,
        });
    }
    Ok(windows)
}

fn normalized_mutation_calls(
    calls: &[MutationCallDraft<'_>],
) -> Result<super::Collection<MutationCallState>, super::ContractError> {
    super::validate_item_count(super::MUTATION_CALL_FIELD, calls.len())?;
    let mut states = super::Collection::with_capacity(calls.len());
    for call in calls {
        states.push(MutationCallState {
            call_id: super::required_text(super::MUTATION_CALL_FIELD, call.call_id)?,
            mutation_targets: super::normalized_label_set(
                super::MUTATION_TARGET_FIELD,
                call.mutation_targets,
            )?,
        });
    }
    Ok(states)
}

fn latest_conflicting_call(
    current: &MutationCallState,
    states: &[MutationCallState],
    current_index: usize,
) -> Option<super::Text> {
    for previous in states[..current_index].iter().rev() {
        if has_target_overlap(&current.mutation_targets, &previous.mutation_targets) {
            return Some(previous.call_id.clone());
        }
    }
    None
}

fn has_target_overlap(left: &[super::Text], right: &[super::Text]) -> bool {
    left.iter()
        .any(|left_target| right.iter().any(|right_target| right_target == left_target))
}
