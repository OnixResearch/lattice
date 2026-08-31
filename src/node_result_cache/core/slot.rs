//! Pure slot allocation decision.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Plan {
    UseFree { slot: usize },
    Append { slot: usize },
    Replace { key: super::Key },
}

#[derive(Debug, Clone, Copy)]
pub struct Input<'a> {
    pub capacity_entries: std::num::NonZeroUsize,
    pub active_entries: usize,
    pub free_slots: &'a [usize],
    pub eviction_target: Option<super::Key>,
}

// r[impl lattice.node_result_cache.decision]
pub fn plan(input: Input<'_>) -> Result<Plan, super::DecisionError> {
    let mut selected_free = None;
    for slot in input.free_slots {
        if *slot >= input.capacity_entries.get() {
            return Err(super::DecisionError::InvalidFreeSlot);
        }
        selected_free = Some(selected_free.map_or(*slot, |selected: usize| selected.min(*slot)));
    }
    debug_assert!(selected_free.is_none_or(|slot| slot < input.capacity_entries.get()));
    if let Some(slot) = selected_free {
        return Ok(Plan::UseFree { slot });
    }
    debug_assert!(input.active_entries <= input.capacity_entries.get());
    if input.active_entries < input.capacity_entries.get() {
        return Ok(Plan::Append {
            slot: input.active_entries,
        });
    }
    input
        .eviction_target
        .map(|key| Plan::Replace { key })
        .ok_or(super::DecisionError::MissingEvictionTarget)
}
