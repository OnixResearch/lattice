//! Ordered layer stack with visibility, locking, and item ownership.
//!
//! Provides a generic layer management system with:
//! - Layer ordering (bottom to top)
//! - Visibility and lock controls
//! - Item-to-layer ownership mapping
//! - UUID-based layer identifiers for CRDT compatibility

/// Layer identifier backed by UUID v4 for globally-unique identification
/// suitable for CRDT merging.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LayerId(pub uuid::Uuid);

impl LayerId {
    /// Create a new LayerId with a fresh UUID v4.
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }
}

impl Default for LayerId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for LayerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Layer data structure with id, name, visibility, and lock state.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Layer {
    pub id: LayerId,
    pub name: String,
    pub visible: bool,
    pub locked: bool,
}

impl Layer {
    /// Create a new layer with a fresh id, given name, visible=true, locked=false.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: LayerId::new(),
            name: name.into(),
            visible: true,
            locked: false,
        }
    }

    /// Create a new layer with the specified id, given name, visible=true, locked=false.
    pub fn with_id(id: LayerId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            visible: true,
            locked: false,
        }
    }
}

/// Ordered layer stack with item ownership tracking.
///
/// Generic over `I: Eq + Hash + Copy + std::fmt::Debug` which is the item identifier type.
/// Items can be assigned to layers and are tracked via a HashMap.
/// Layers are ordered bottom to top for rendering.
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LayerStack<I>
where
    I: Eq + std::hash::Hash + Copy + std::fmt::Debug,
{
    layers: Vec<Layer>,
    item_layers: std::collections::HashMap<I, LayerId>,
}

fn u32_to_layer_index(index: u32) -> usize {
    match usize::try_from(index) {
        Ok(value) => value,
        Err(_) => usize::MAX,
    }
}

fn usize_to_layer_index(index: usize) -> u32 {
    match u32::try_from(index) {
        Ok(value) => value,
        Err(_) => u32::MAX,
    }
}

impl<I> LayerStack<I>
where
    I: Eq + std::hash::Hash + Copy + std::fmt::Debug,
{
    /// Create a new layer stack with one default layer "Layer 1".
    pub fn new() -> Self {
        Self {
            layers: vec![Layer::new("Layer 1")],
            item_layers: std::collections::HashMap::new(),
        }
    }

    /// Add a new layer to the top of the stack and return its id.
    pub fn add_layer(&mut self, name: &str) -> LayerId {
        let layer = Layer::new(name.to_string());
        let id = layer.id;
        self.layers.push(layer);
        id
    }

    /// Remove a layer by id. Returns an error if it's the last layer.
    /// Items owned by the removed layer are reassigned to the first remaining layer.
    pub fn remove_layer(&mut self, id: LayerId) -> Result<(), String> {
        if self.layers.len() <= 1 {
            return Err("Cannot remove the last layer".to_string());
        }

        if let Some(index) = self.layers.iter().position(|layer| layer.id == id) {
            self.layers.remove(index);

            // Reassign items from the removed layer to the first layer
            let first_layer_id = self.layers[0].id;
            for (_, layer_id) in self.item_layers.iter_mut() {
                if *layer_id == id {
                    *layer_id = first_layer_id;
                }
            }
        }

        Ok(())
    }

    /// Move a layer to a new position in the stack.
    /// Index is clamped to valid range [0, layer_count).
    pub fn move_layer(&mut self, id: LayerId, new_index: u32) {
        if let Some(current_index) = self.layers.iter().position(|layer| layer.id == id) {
            let requested_index = u32_to_layer_index(new_index);
            let bounded_index = requested_index.min(self.layers.len().saturating_sub(1));
            let layer = self.layers.remove(current_index);
            self.layers.insert(bounded_index, layer);
        }
    }

    /// Rename a layer.
    pub fn rename_layer(&mut self, id: LayerId, name: &str) {
        if let Some(layer) = self.layers.iter_mut().find(|layer| layer.id == id) {
            layer.name = name.to_string();
        }
    }

    /// Set layer visibility.
    pub fn set_visible(&mut self, id: LayerId, visible: bool) {
        if let Some(layer) = self.layers.iter_mut().find(|layer| layer.id == id) {
            layer.visible = visible;
        }
    }

    /// Set layer lock state.
    pub fn set_locked(&mut self, id: LayerId, locked: bool) {
        if let Some(layer) = self.layers.iter_mut().find(|layer| layer.id == id) {
            layer.locked = locked;
        }
    }

    /// Check if a layer is visible. Returns true if layer not found (safe default).
    pub fn is_visible(&self, id: LayerId) -> bool {
        self.layers
            .iter()
            .find(|layer| layer.id == id)
            .map(|layer| layer.visible)
            .unwrap_or(true)
    }

    /// Check if a layer is locked. Returns false if layer not found (safe default).
    pub fn is_locked(&self, id: LayerId) -> bool {
        self.layers
            .iter()
            .find(|layer| layer.id == id)
            .map(|layer| layer.locked)
            .unwrap_or(false)
    }

    /// Get a layer by id.
    pub fn get_layer(&self, id: LayerId) -> Option<&Layer> {
        self.layers.iter().find(|layer| layer.id == id)
    }

    /// Iterate over layers in render order (bottom to top).
    pub fn layers_bottom_to_top(&self) -> impl Iterator<Item = &Layer> {
        self.layers.iter()
    }

    /// Get the number of layers.
    pub fn layer_count(&self) -> u32 {
        usize_to_layer_index(self.layers.len())
    }

    /// Get the z-index (0-based position) of a layer.
    /// Returns None if the layer is not found.
    pub fn z_index(&self, id: LayerId) -> Option<u32> {
        self.layers
            .iter()
            .position(|layer| layer.id == id)
            .map(usize_to_layer_index)
    }

    /// Get the id of the default (first) layer.
    pub fn default_layer(&self) -> LayerId {
        self.layers[0].id
    }

    /// Assign an item to a layer.
    pub fn set_item_layer(&mut self, item: I, layer: LayerId) {
        self.item_layers.insert(item, layer);
    }

    /// Get the layer that owns an item.
    /// Returns None if the item is not assigned to any layer.
    pub fn get_item_layer(&self, item: &I) -> Option<LayerId> {
        self.item_layers.get(item).copied()
    }

    /// Remove an item from the ownership map.
    pub fn remove_item(&mut self, item: &I) {
        self.item_layers.remove(item);
    }
}

impl<I> Default for LayerStack<I>
where
    I: Eq + std::hash::Hash + Copy + std::fmt::Debug,
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests;
