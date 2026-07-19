//! Generic modal keymap implementation.

/// Generic modal keymap that maps key combinations to actions within specific modes.
///
/// Type parameters:
/// - `A`: Action type (must be Clone)
/// - `M`: Mode type (must be Eq + Hash + Clone)
#[derive(Debug, Clone)]
pub struct Map<A, M>
where
    A: Clone,
    M: Eq + std::hash::Hash + Clone,
{
    /// Binding tables per mode.
    modes: std::collections::HashMap<M, std::collections::HashMap<crate::combo::Chord, A>>,
}

impl<A, M> Map<A, M>
where
    A: Clone,
    M: Eq + std::hash::Hash + Clone,
{
    /// Create a new empty keymap.
    pub fn new() -> Self {
        Self {
            modes: std::collections::HashMap::new(),
        }
    }

    /// Resolve a key event in the given mode. Returns `None` for unmapped keys.
    pub fn resolve(&self, mode: &M, event: &crate::combo::Event) -> Option<A> {
        let combo = crate::combo::Chord::from_event(event);
        self.modes
            .get(mode)
            .and_then(|mode_map| mode_map.get(&combo))
            .cloned()
    }

    /// Build a keymap from mode bindings with optional string-based overrides.
    ///
    /// # Parameters
    /// - `mode_bindings`: Initial bindings per mode
    /// - `overrides`: String-based overrides per mode (key_string -> action_string)
    /// - `parse_action`: Function to parse action strings
    pub fn build<F>(
        mode_bindings: Vec<(M, std::collections::HashMap<crate::combo::Chord, A>)>,
        overrides: &[(M, std::collections::HashMap<String, String>)],
        parse_action: F,
    ) -> Self
    where
        F: Fn(&str) -> Option<A>,
    {
        let mut keymap = Self::new();

        // Apply initial bindings.
        for (mode, bindings) in mode_bindings {
            keymap.modes.insert(mode, bindings);
        }

        // Apply overrides.
        for (mode, mode_overrides) in overrides {
            let mode_map = keymap.modes.entry(mode.clone()).or_default();
            for (key_str, action_str) in mode_overrides {
                if let (Some(combo), Some(action)) = (
                    crate::combo::parse_key_string(key_str),
                    parse_action(action_str),
                ) {
                    mode_map.insert(combo, action);
                }
            }
        }

        keymap
    }

    /// List all bindings for a mode (for help display).
    /// Returns tuples of (key_string, action).
    pub fn describe(&self, mode: &M) -> Vec<(String, A)> {
        self.modes
            .get(mode)
            .map(|mode_map| {
                mode_map
                    .iter()
                    .map(|(combo, action)| (crate::combo::format(combo), action.clone()))
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Add or update a single binding.
    pub fn set(&mut self, mode: M, combo: crate::combo::Chord, action: A) {
        self.modes.entry(mode).or_default().insert(combo, action);
    }

    /// Get all modes that have bindings.
    pub fn modes(&self) -> impl Iterator<Item = &M> {
        self.modes.keys()
    }

    /// Check if a mode has any bindings.
    pub fn has_mode(&self, mode: &M) -> bool {
        self.modes.contains_key(mode)
    }

    /// Get the number of bindings in a specific mode.
    pub fn mode_binding_count(&self, mode: &M) -> u64 {
        self.modes
            .get(mode)
            .map(|mode_map| usize_to_public_count(mode_map.len()))
            .unwrap_or(0)
    }

    /// Clear all bindings for a mode.
    pub fn clear_mode(&mut self, mode: &M) {
        self.modes.remove(mode);
    }
}

fn usize_to_public_count(count: usize) -> u64 {
    match u64::try_from(count) {
        Ok(value) => value,
        Err(_) => u64::MAX,
    }
}

impl<A, M> Default for Map<A, M>
where
    A: Clone,
    M: Eq + std::hash::Hash + Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests;
