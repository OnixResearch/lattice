//! Ordered layer stack with caller-owned identities, visibility, locking, and item ownership.
//!
//! The reusable model never manufactures authoritative identities. Applications
//! create IDs at their native or browser shell edge and pass the resulting plain
//! [`LayerId`] values into this crate.

/// Error returned when an external value is not an admissible layer identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayerIdError {
    /// The all-zero UUID is reserved and cannot identify a layer.
    Nil,
}

impl std::fmt::Display for LayerIdError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Nil => formatter.write_str("layer identity must not be nil"),
        }
    }
}

impl std::error::Error for LayerIdError {}

/// Caller-supplied layer identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct LayerId(uuid::Uuid);

impl LayerId {
    /// Validates a UUID supplied by the owning application or adapter.
    pub fn from_uuid(value: uuid::Uuid) -> Result<Self, LayerIdError> {
        if value.is_nil() {
            Err(LayerIdError::Nil)
        } else {
            Ok(Self(value))
        }
    }

    /// Returns the serialization-compatible backing UUID.
    #[must_use]
    pub const fn as_uuid(self) -> uuid::Uuid {
        self.0
    }
}

impl TryFrom<uuid::Uuid> for LayerId {
    type Error = LayerIdError;

    fn try_from(value: uuid::Uuid) -> Result<Self, Self::Error> {
        Self::from_uuid(value)
    }
}

impl From<LayerId> for uuid::Uuid {
    fn from(value: LayerId) -> Self {
        value.0
    }
}

impl std::fmt::Display for LayerId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(formatter)
    }
}

#[cfg(feature = "serde")]
impl serde::Serialize for LayerId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where S: serde::Serializer {
        self.0.serialize(serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for LayerId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: serde::Deserializer<'de> {
        use serde::de::Error as _;
        let value = uuid::Uuid::deserialize(deserializer)?;
        Self::from_uuid(value).map_err(D::Error::custom)
    }
}

/// Current application-observed layer model generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LayerGeneration(u64);

impl LayerGeneration {
    /// Creates a generation value supplied by the model owner.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the plain generation number.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }

    fn checked_next(self) -> Option<Self> {
        self.0.checked_add(1).map(Self)
    }
}

/// Caller-prepared request to create a layer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LayerCreation {
    /// Identity allocated in the caller's authority domain.
    pub id: LayerId,
    /// Generation on which the caller based the request.
    pub observed_generation: LayerGeneration,
    /// Finite display name retained by the reusable model.
    pub name: String,
}

impl LayerCreation {
    /// Creates a plain creation request without retaining a generator or callback.
    #[must_use]
    pub fn new(id: LayerId, observed_generation: LayerGeneration, name: impl Into<String>) -> Self {
        Self {
            id,
            observed_generation,
            name: name.into(),
        }
    }
}

/// Rejected layer-stack transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayerStackError {
    /// A creation request used an identity already present in the stack.
    DuplicateId(LayerId),
    /// A creation request was prepared against an old or future generation.
    StaleGeneration {
        /// Current authoritative model generation.
        current: LayerGeneration,
        /// Generation observed by the rejected request.
        observed: LayerGeneration,
    },
    /// The model generation cannot advance without overflow.
    GenerationOverflow,
    /// The stack must retain at least one layer.
    LastLayer,
}

impl std::fmt::Display for LayerStackError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateId(id) => write!(formatter, "duplicate layer identity: {id}"),
            Self::StaleGeneration { current, observed } => {
                write!(formatter, "stale layer generation: current={}, observed={}", current.get(), observed.get())
            }
            Self::GenerationOverflow => formatter.write_str("layer generation overflow"),
            Self::LastLayer => formatter.write_str("cannot remove the last layer"),
        }
    }
}

impl std::error::Error for LayerStackError {}

/// Layer data with caller-owned identity, name, visibility, and lock state.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Layer {
    /// Stable caller-owned identity.
    pub id: LayerId,
    /// Display name.
    pub name: String,
    /// Whether the layer participates in rendering.
    pub visible: bool,
    /// Whether mutations to layer-owned items are disabled.
    pub locked: bool,
}

impl Layer {
    /// Creates a layer from a caller-supplied identity.
    #[must_use]
    pub fn new(id: LayerId, name: impl Into<String>) -> Self {
        Self {
            id,
            name: name.into(),
            visible: true,
            locked: false,
        }
    }
}

/// Ordered layer stack with item ownership tracking.
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LayerStack<I>
where I: Eq + std::hash::Hash + Copy + std::fmt::Debug
{
    layers: Vec<Layer>,
    item_layers: std::collections::HashMap<I, LayerId>,
    generation: LayerGeneration,
}

fn u32_to_layer_index(index: u32) -> usize {
    usize::try_from(index).unwrap_or(usize::MAX)
}

fn usize_to_layer_index(index: usize) -> u32 {
    u32::try_from(index).unwrap_or(u32::MAX)
}

impl<I> LayerStack<I>
where I: Eq + std::hash::Hash + Copy + std::fmt::Debug
{
    /// Creates a stack with one explicitly identified default layer.
    #[must_use]
    pub fn new(default_id: LayerId, generation: LayerGeneration) -> Self {
        Self {
            layers: vec![Layer::new(default_id, "Layer 1")],
            item_layers: std::collections::HashMap::new(),
            generation,
        }
    }

    /// Returns the generation against which creation requests are validated.
    #[must_use]
    pub const fn generation(&self) -> LayerGeneration {
        self.generation
    }

    /// Adds a layer atomically from a current caller-prepared request.
    pub fn add_layer(&mut self, creation: LayerCreation) -> Result<LayerId, LayerStackError> {
        if creation.observed_generation != self.generation {
            return Err(LayerStackError::StaleGeneration {
                current: self.generation,
                observed: creation.observed_generation,
            });
        }
        if self.layers.iter().any(|layer| layer.id == creation.id) {
            return Err(LayerStackError::DuplicateId(creation.id));
        }
        let next_generation = self.generation.checked_next().ok_or(LayerStackError::GenerationOverflow)?;
        let id = creation.id;
        self.layers.push(Layer::new(id, creation.name));
        self.generation = next_generation;
        Ok(id)
    }

    /// Removes a layer. Items are reassigned to the first remaining layer.
    pub fn remove_layer(&mut self, id: LayerId) -> Result<(), LayerStackError> {
        if self.layers.len() <= 1 {
            return Err(LayerStackError::LastLayer);
        }

        if let Some(index) = self.layers.iter().position(|layer| layer.id == id) {
            self.layers.remove(index);
            let first_layer_id = self.layers[0].id;
            for layer_id in self.item_layers.values_mut() {
                if *layer_id == id {
                    *layer_id = first_layer_id;
                }
            }
        }
        Ok(())
    }

    /// Moves a layer to a clamped position in the stack.
    pub fn move_layer(&mut self, id: LayerId, new_index: u32) {
        if let Some(current_index) = self.layers.iter().position(|layer| layer.id == id) {
            let bounded_index = u32_to_layer_index(new_index).min(self.layers.len().saturating_sub(1));
            let layer = self.layers.remove(current_index);
            self.layers.insert(bounded_index, layer);
        }
    }

    /// Renames a layer.
    pub fn rename_layer(&mut self, id: LayerId, name: &str) {
        if let Some(layer) = self.layers.iter_mut().find(|layer| layer.id == id) {
            layer.name = name.to_string();
        }
    }

    /// Sets layer visibility.
    pub fn set_visible(&mut self, id: LayerId, visible: bool) {
        if let Some(layer) = self.layers.iter_mut().find(|layer| layer.id == id) {
            layer.visible = visible;
        }
    }

    /// Sets layer lock state.
    pub fn set_locked(&mut self, id: LayerId, locked: bool) {
        if let Some(layer) = self.layers.iter_mut().find(|layer| layer.id == id) {
            layer.locked = locked;
        }
    }

    /// Returns whether a layer is visible, defaulting safely for an unknown ID.
    #[must_use]
    pub fn is_visible(&self, id: LayerId) -> bool {
        self.layers.iter().find(|layer| layer.id == id).map(|layer| layer.visible).unwrap_or(true)
    }

    /// Returns whether a layer is locked, defaulting safely for an unknown ID.
    #[must_use]
    pub fn is_locked(&self, id: LayerId) -> bool {
        self.layers.iter().find(|layer| layer.id == id).map(|layer| layer.locked).unwrap_or(false)
    }

    /// Gets a layer by ID.
    #[must_use]
    pub fn get_layer(&self, id: LayerId) -> Option<&Layer> {
        self.layers.iter().find(|layer| layer.id == id)
    }

    /// Iterates over layers in render order, bottom to top.
    pub fn layers_bottom_to_top(&self) -> impl Iterator<Item = &Layer> {
        self.layers.iter()
    }

    /// Returns the number of layers.
    #[must_use]
    pub fn layer_count(&self) -> u32 {
        usize_to_layer_index(self.layers.len())
    }

    /// Returns the zero-based stack position of a layer.
    #[must_use]
    pub fn z_index(&self, id: LayerId) -> Option<u32> {
        self.layers.iter().position(|layer| layer.id == id).map(usize_to_layer_index)
    }

    /// Returns the first layer ID.
    #[must_use]
    pub fn default_layer(&self) -> LayerId {
        self.layers[0].id
    }

    /// Assigns an item to a layer.
    pub fn set_item_layer(&mut self, item: I, layer: LayerId) {
        self.item_layers.insert(item, layer);
    }

    /// Returns the layer that owns an item.
    #[must_use]
    pub fn get_item_layer(&self, item: &I) -> Option<LayerId> {
        self.item_layers.get(item).copied()
    }

    /// Removes an item from the ownership map.
    pub fn remove_item(&mut self, item: &I) {
        self.item_layers.remove(item);
    }
}

#[cfg(test)]
mod tests;
