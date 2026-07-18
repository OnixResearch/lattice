use super::*;

const UUID_STRING_LEN: usize = 36;
const UUID_VERSION_V4: usize = 4;
const TEST_ITEM: u32 = 42;
const OUT_OF_RANGE_LAYER_INDEX: u32 = 100;

#[test]
fn layer_id_uniqueness() {
    let id1 = LayerId::new();
    let id2 = LayerId::new();
    assert_ne!(id1, id2);
}

#[test]
fn layer_id_equality() {
    let uuid = uuid::Uuid::new_v4();
    let id1 = LayerId(uuid);
    let id2 = LayerId(uuid);
    assert_eq!(id1, id2);

    let mut set = std::collections::HashSet::new();
    set.insert(id1);
    assert!(set.contains(&id2));
}

#[test]
fn layer_id_display() {
    let id = LayerId::new();
    let display_str = format!("{}", id);
    assert_eq!(display_str.len(), UUID_STRING_LEN);

    // Should be able to parse back to UUID
    let parsed = uuid::Uuid::parse_str(&display_str);
    assert!(parsed.is_ok());
    assert_eq!(parsed.unwrap(), id.0);
}

#[test]
fn layer_id_default() {
    let id = LayerId::default();
    // Default should create new UUID
    assert_eq!(id.0.get_version_num(), UUID_VERSION_V4);
}

#[test]
fn layer_new_defaults() {
    let layer = Layer::new("Background");
    assert_eq!(layer.name, "Background");
    assert!(layer.visible);
    assert!(!layer.locked);
    assert_eq!(layer.id.0.get_version_num(), UUID_VERSION_V4);
}

#[test]
fn layer_with_id() {
    let id = LayerId::new();
    let layer = Layer::with_id(id, "Overlay");
    assert_eq!(layer.id, id);
    assert_eq!(layer.name, "Overlay");
    assert!(layer.visible);
    assert!(!layer.locked);
}

#[test]
fn layer_stack_new_has_default_layer() {
    let stack: LayerStack<u32> = LayerStack::new();
    assert_eq!(stack.layer_count(), 1);

    let layers: Vec<_> = stack.layers_bottom_to_top().collect();
    assert_eq!(layers.len(), 1);
    assert_eq!(layers[0].name, "Layer 1");
}

#[test]
fn layer_stack_add_layer() {
    let mut stack: LayerStack<u32> = LayerStack::new();
    let id = stack.add_layer("Layer 2");

    assert_eq!(stack.layer_count(), 2);

    let layer = stack.get_layer(id);
    assert!(layer.is_some());
    assert_eq!(layer.unwrap().name, "Layer 2");
}

#[test]
fn layer_stack_remove_layer_preserves_minimum() {
    let mut stack: LayerStack<u32> = LayerStack::new();
    let default_id = stack.default_layer();

    let result = stack.remove_layer(default_id);
    assert!(result.is_err());
    assert_eq!(stack.layer_count(), 1);
}

#[test]
fn layer_stack_remove_layer_reassigns_items() {
    let mut stack: LayerStack<u32> = LayerStack::new();
    let layer2_id = stack.add_layer("Layer 2");
    let default_id = stack.default_layer();

    // Assign item to layer 2
    stack.set_item_layer(TEST_ITEM, layer2_id);
    assert_eq!(stack.get_item_layer(&TEST_ITEM), Some(layer2_id));

    // Remove layer 2
    let result = stack.remove_layer(layer2_id);
    assert!(result.is_ok());

    // Item should be reassigned to default layer
    assert_eq!(stack.get_item_layer(&TEST_ITEM), Some(default_id));
}

#[test]
fn layer_stack_move_layer() {
    let mut stack: LayerStack<u32> = LayerStack::new();
    let layer2_id = stack.add_layer("Layer 2");
    let layer3_id = stack.add_layer("Layer 3");

    // Initially: [Layer 1, Layer 2, Layer 3]
    assert_eq!(stack.z_index(layer2_id), Some(1));
    assert_eq!(stack.z_index(layer3_id), Some(2));

    // Move Layer 3 to position 0
    stack.move_layer(layer3_id, 0);

    // Now: [Layer 3, Layer 1, Layer 2]
    assert_eq!(stack.z_index(layer3_id), Some(0));
    assert_eq!(stack.z_index(layer2_id), Some(2));
}

#[test]
fn layer_stack_move_layer_clamps_index() {
    let mut stack: LayerStack<u32> = LayerStack::new();
    let layer_id = stack.add_layer("Layer 2");

    // Move to index way beyond bounds
    stack.move_layer(layer_id, OUT_OF_RANGE_LAYER_INDEX);

    // Should be clamped to last position
    assert_eq!(stack.z_index(layer_id), Some(1));
}

#[test]
fn layer_stack_rename_layer() {
    let mut stack: LayerStack<u32> = LayerStack::new();
    let layer_id = stack.add_layer("Original");

    stack.rename_layer(layer_id, "Renamed");

    let layer = stack.get_layer(layer_id);
    assert!(layer.is_some());
    assert_eq!(layer.unwrap().name, "Renamed");
}

#[test]
fn layer_stack_visibility_toggle() {
    let mut stack: LayerStack<u32> = LayerStack::new();
    let layer_id = stack.add_layer("Test Layer");

    // Initially visible
    assert!(stack.is_visible(layer_id));

    // Set invisible
    stack.set_visible(layer_id, false);
    assert!(!stack.is_visible(layer_id));

    // Set visible again
    stack.set_visible(layer_id, true);
    assert!(stack.is_visible(layer_id));
}

#[test]
fn layer_stack_lock_toggle() {
    let mut stack: LayerStack<u32> = LayerStack::new();
    let layer_id = stack.add_layer("Test Layer");

    // Initially unlocked
    assert!(!stack.is_locked(layer_id));

    // Set locked
    stack.set_locked(layer_id, true);
    assert!(stack.is_locked(layer_id));

    // Set unlocked again
    stack.set_locked(layer_id, false);
    assert!(!stack.is_locked(layer_id));
}

#[test]
fn layer_stack_safe_defaults() {
    let stack: LayerStack<u32> = LayerStack::new();
    let nonexistent_id = LayerId::new();

    // Should return safe defaults for nonexistent layers
    assert!(stack.is_visible(nonexistent_id)); // true for safety
    assert!(!stack.is_locked(nonexistent_id)); // false for safety
    assert!(stack.get_layer(nonexistent_id).is_none());
    assert!(stack.z_index(nonexistent_id).is_none());
}

#[test]
fn layer_stack_item_ownership() {
    let mut stack: LayerStack<u32> = LayerStack::new();
    let layer_id = stack.add_layer("Test Layer");

    // Initially unassigned
    assert_eq!(stack.get_item_layer(&TEST_ITEM), None);

    // Assign to layer
    stack.set_item_layer(TEST_ITEM, layer_id);
    assert_eq!(stack.get_item_layer(&TEST_ITEM), Some(layer_id));

    // Remove item
    stack.remove_item(&TEST_ITEM);
    assert_eq!(stack.get_item_layer(&TEST_ITEM), None);
}

#[test]
fn layer_stack_render_order() {
    let mut stack: LayerStack<u32> = LayerStack::new();
    let layer2_id = stack.add_layer("Layer 2");
    let layer3_id = stack.add_layer("Layer 3");

    let layers: Vec<_> = stack.layers_bottom_to_top().collect();
    assert_eq!(layers.len(), 3);
    assert_eq!(layers[0].name, "Layer 1");
    assert_eq!(layers[1].name, "Layer 2");
    assert_eq!(layers[2].name, "Layer 3");

    // Verify z-index matches iteration order
    assert_eq!(stack.z_index(stack.default_layer()), Some(0));
    assert_eq!(stack.z_index(layer2_id), Some(1));
    assert_eq!(stack.z_index(layer3_id), Some(2));
}

#[test]
fn layer_stack_default_layer() {
    let stack: LayerStack<u32> = LayerStack::new();
    let default_id = stack.default_layer();

    let layer = stack.get_layer(default_id);
    assert!(layer.is_some());
    assert_eq!(layer.unwrap().name, "Layer 1");
}

#[test]
fn layer_stack_different_item_types() {
    // Test with string item IDs
    let mut stack: LayerStack<&str> = LayerStack::new();
    let layer_id = stack.add_layer("Test");

    stack.set_item_layer("item1", layer_id);
    assert_eq!(stack.get_item_layer(&"item1"), Some(layer_id));

    stack.remove_item(&"item1");
    assert_eq!(stack.get_item_layer(&"item1"), None);
}

#[cfg(feature = "serde")]
#[test]
fn layer_id_serde() {
    let id = LayerId::new();
    let json = serde_json::to_string(&id).unwrap();
    let deserialized: LayerId = serde_json::from_str(&json).unwrap();
    assert_eq!(id, deserialized);
}

#[cfg(feature = "serde")]
#[test]
fn layer_serde() {
    let layer = Layer::new("Test Layer");
    let json = serde_json::to_string(&layer).unwrap();
    let deserialized: Layer = serde_json::from_str(&json).unwrap();
    assert_eq!(layer.id, deserialized.id);
    assert_eq!(layer.name, deserialized.name);
    assert_eq!(layer.visible, deserialized.visible);
    assert_eq!(layer.locked, deserialized.locked);
}

#[cfg(feature = "serde")]
#[test]
fn layer_stack_serde() {
    let mut stack: LayerStack<u32> = LayerStack::new();
    let layer_id = stack.add_layer("Test Layer");
    stack.set_item_layer(TEST_ITEM, layer_id);

    let json = serde_json::to_string(&stack).unwrap();
    let deserialized: LayerStack<u32> = serde_json::from_str(&json).unwrap();

    assert_eq!(stack.layer_count(), deserialized.layer_count());
    assert_eq!(
        stack.get_item_layer(&TEST_ITEM),
        deserialized.get_item_layer(&TEST_ITEM)
    );
}
