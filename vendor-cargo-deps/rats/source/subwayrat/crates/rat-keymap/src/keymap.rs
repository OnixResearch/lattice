//! Generic backend-neutral modal keymap implementation.

use ratcore::action_registry::KeySequence;
use ratcore::action_registry::LogicalKeystroke;

/// Result of offering a validated sequence to the one-stroke compatibility map.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LocalSequenceResolution<A> {
    /// A one-stroke component-local action was found.
    Local(A),
    /// No component-local action was bound.
    Unbound,
    /// Multi-stroke resolution belongs to Ratcore's layered binding catalog.
    LayeredResolutionRequired,
}

/// Generic component-local map from logical keystrokes to actions in modes.
///
/// This compatibility map is intentionally one-stroke and does not dispatch
/// application effects. Generation-bound application action resolution remains
/// owned by Ratcore's layered binding catalog and contextual router.
#[derive(Debug, Clone)]
pub struct Map<A, M>
where
    A: Clone,
    M: Eq + std::hash::Hash + Clone,
{
    modes: std::collections::HashMap<M, std::collections::HashMap<LogicalKeystroke, A>>,
}

impl<A, M> Map<A, M>
where
    A: Clone,
    M: Eq + std::hash::Hash + Clone,
{
    /// Creates an empty keymap.
    #[must_use]
    pub fn new() -> Self {
        Self {
            modes: std::collections::HashMap::new(),
        }
    }

    /// Resolves one already-normalized logical stroke in a mode.
    #[must_use]
    pub fn resolve(&self, mode: &M, stroke: &LogicalKeystroke) -> Option<A> {
        self.modes.get(mode).and_then(|mode_map| mode_map.get(stroke)).cloned()
    }

    /// Resolves only a one-stroke local sequence and explicitly hands longer
    /// application sequences to the layered resolver.
    #[must_use]
    pub fn resolve_local_sequence(&self, mode: &M, sequence: &KeySequence) -> LocalSequenceResolution<A> {
        match sequence.strokes() {
            [stroke] => {
                self.resolve(mode, stroke).map_or(LocalSequenceResolution::Unbound, LocalSequenceResolution::Local)
            }
            _ => LocalSequenceResolution::LayeredResolutionRequired,
        }
    }

    /// Builds a map from logical bindings and deterministic string overrides.
    pub fn build<F>(
        mode_bindings: Vec<(M, std::collections::HashMap<LogicalKeystroke, A>)>,
        overrides: &[(M, std::collections::HashMap<String, String>)],
        parse_action: F,
    ) -> Self
    where
        F: Fn(&str) -> Option<A>,
    {
        let mut keymap = Self::new();
        for (mode, bindings) in mode_bindings {
            keymap.modes.insert(mode, bindings);
        }

        for (mode, mode_overrides) in overrides {
            let mode_map = keymap.modes.entry(mode.clone()).or_default();
            for (key_string, action_string) in mode_overrides {
                if let (Some(stroke), Some(action)) =
                    (crate::combo::parse_key_string(key_string), parse_action(action_string))
                {
                    mode_map.insert(stroke, action);
                }
            }
        }
        keymap
    }

    /// Lists bindings in deterministic formatted-label then logical-key order.
    #[must_use]
    pub fn describe(&self, mode: &M) -> Vec<(String, A)> {
        self.modes
            .get(mode)
            .map(|mode_map| {
                let mut bindings = mode_map
                    .iter()
                    .map(|(stroke, action)| (crate::combo::format(stroke), stroke, action.clone()))
                    .collect::<Vec<_>>();
                bindings.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(right.1)));
                bindings.into_iter().map(|(label, _, action)| (label, action)).collect()
            })
            .unwrap_or_default()
    }

    /// Adds or updates one logical binding.
    pub fn set(&mut self, mode: M, stroke: LogicalKeystroke, action: A) {
        self.modes.entry(mode).or_default().insert(stroke, action);
    }

    /// Returns modes that contain bindings.
    pub fn modes(&self) -> impl Iterator<Item = &M> {
        self.modes.keys()
    }

    /// Reports whether a mode exists.
    #[must_use]
    pub fn has_mode(&self, mode: &M) -> bool {
        self.modes.contains_key(mode)
    }

    /// Returns the bounded public count of bindings in a mode.
    #[must_use]
    pub fn mode_binding_count(&self, mode: &M) -> u64 {
        self.modes.get(mode).map(|mode_map| u64::try_from(mode_map.len()).unwrap_or(u64::MAX)).unwrap_or(0)
    }

    /// Removes one mode and all of its local bindings.
    pub fn clear_mode(&mut self, mode: &M) {
        self.modes.remove(mode);
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
