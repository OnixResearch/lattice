use super::*;

const TEST_ITEM: u32 = 42;
const OUT_OF_RANGE_LAYER_INDEX: u32 = 100;
const INITIAL_GENERATION: LayerGeneration = LayerGeneration::new(7);

fn id(value: u128) -> LayerId {
    LayerId::from_uuid(uuid::Uuid::from_u128(value)).expect("nonzero test UUID")
}

fn stack() -> LayerStack<u32> {
    LayerStack::new(id(1), INITIAL_GENERATION)
}

fn add<I>(stack: &mut LayerStack<I>, value: u128, name: &str) -> LayerId
where I: Eq + std::hash::Hash + Copy + std::fmt::Debug {
    stack
        .add_layer(LayerCreation::new(id(value), stack.generation(), name))
        .expect("current unique creation")
}

#[test]
fn layer_id_rejects_nil_and_preserves_supplied_value() {
    assert_eq!(LayerId::from_uuid(uuid::Uuid::nil()), Err(LayerIdError::Nil));
    let supplied = uuid::Uuid::from_u128(42);
    let layer_id = LayerId::from_uuid(supplied).expect("valid supplied ID");
    assert_eq!(layer_id.as_uuid(), supplied);
    assert_eq!(layer_id.to_string().len(), 36);
}

#[test]
fn layer_new_uses_caller_identity_and_defaults() {
    let supplied = id(2);
    let layer = Layer::new(supplied, "Background");
    assert_eq!(layer.id, supplied);
    assert_eq!(layer.name, "Background");
    assert!(layer.visible);
    assert!(!layer.locked);
}

#[test]
fn layer_stack_new_has_supplied_default_layer() {
    let stack = stack();
    assert_eq!(stack.layer_count(), 1);
    assert_eq!(stack.default_layer(), id(1));
    assert_eq!(stack.generation(), INITIAL_GENERATION);
    assert_eq!(stack.layers_bottom_to_top().next().map(|layer| layer.name.as_str()), Some("Layer 1"));
}

#[test]
fn layer_stack_add_layer_advances_generation() {
    let mut stack = stack();
    let added = add(&mut stack, 2, "Layer 2");
    assert_eq!(added, id(2));
    assert_eq!(stack.layer_count(), 2);
    assert_eq!(stack.generation(), LayerGeneration::new(8));
    assert_eq!(stack.get_layer(added).map(|layer| layer.name.as_str()), Some("Layer 2"));
}

#[test]
fn duplicate_and_stale_creations_are_atomic() {
    let mut stack = stack();
    let initial = stack.generation();
    let duplicate = stack.add_layer(LayerCreation::new(id(1), initial, "Duplicate"));
    assert_eq!(duplicate, Err(LayerStackError::DuplicateId(id(1))));
    assert_eq!(stack.layer_count(), 1);
    assert_eq!(stack.generation(), initial);

    let stale = stack.add_layer(LayerCreation::new(id(2), LayerGeneration::new(6), "Stale"));
    assert_eq!(
        stale,
        Err(LayerStackError::StaleGeneration {
            current: initial,
            observed: LayerGeneration::new(6),
        })
    );
    assert_eq!(stack.layer_count(), 1);
    assert!(stack.get_layer(id(2)).is_none());
}

#[test]
fn generation_overflow_rejects_without_change() {
    let mut stack: LayerStack<u32> = LayerStack::new(id(1), LayerGeneration::new(u64::MAX));
    let result = stack.add_layer(LayerCreation::new(id(2), stack.generation(), "Overflow"));
    assert_eq!(result, Err(LayerStackError::GenerationOverflow));
    assert_eq!(stack.layer_count(), 1);
    assert_eq!(stack.generation(), LayerGeneration::new(u64::MAX));
}

#[test]
fn layer_stack_remove_layer_preserves_minimum() {
    let mut stack = stack();
    assert_eq!(stack.remove_layer(stack.default_layer()), Err(LayerStackError::LastLayer));
    assert_eq!(stack.layer_count(), 1);
}

#[test]
fn layer_stack_remove_layer_reassigns_items() {
    let mut stack = stack();
    let layer2_id = add(&mut stack, 2, "Layer 2");
    let default_id = stack.default_layer();
    stack.set_item_layer(TEST_ITEM, layer2_id);
    stack.remove_layer(layer2_id).expect("remove second layer");
    assert_eq!(stack.get_item_layer(&TEST_ITEM), Some(default_id));
}

#[test]
fn layer_stack_move_layer_and_clamp() {
    let mut stack = stack();
    let layer2_id = add(&mut stack, 2, "Layer 2");
    let layer3_id = add(&mut stack, 3, "Layer 3");
    stack.move_layer(layer3_id, 0);
    assert_eq!(stack.z_index(layer3_id), Some(0));
    assert_eq!(stack.z_index(layer2_id), Some(2));
    stack.move_layer(layer3_id, OUT_OF_RANGE_LAYER_INDEX);
    assert_eq!(stack.z_index(layer3_id), Some(2));
}

#[test]
fn layer_metadata_mutations_and_safe_unknown_defaults() {
    let mut stack = stack();
    let layer_id = add(&mut stack, 2, "Original");
    stack.rename_layer(layer_id, "Renamed");
    stack.set_visible(layer_id, false);
    stack.set_locked(layer_id, true);
    assert_eq!(stack.get_layer(layer_id).map(|layer| layer.name.as_str()), Some("Renamed"));
    assert!(!stack.is_visible(layer_id));
    assert!(stack.is_locked(layer_id));

    let unknown = id(99);
    assert!(stack.is_visible(unknown));
    assert!(!stack.is_locked(unknown));
    assert!(stack.get_layer(unknown).is_none());
    assert!(stack.z_index(unknown).is_none());
}

#[test]
fn layer_stack_item_ownership_and_render_order() {
    let mut stack = stack();
    let layer2_id = add(&mut stack, 2, "Layer 2");
    let layer3_id = add(&mut stack, 3, "Layer 3");
    stack.set_item_layer(TEST_ITEM, layer2_id);
    assert_eq!(stack.get_item_layer(&TEST_ITEM), Some(layer2_id));
    stack.remove_item(&TEST_ITEM);
    assert_eq!(stack.get_item_layer(&TEST_ITEM), None);
    assert_eq!(stack.layers_bottom_to_top().map(|layer| layer.id).collect::<Vec<_>>(), vec![
        id(1),
        layer2_id,
        layer3_id
    ]);
}

#[test]
fn layer_stack_supports_other_item_identity_types() {
    let mut stack: LayerStack<&str> = LayerStack::new(id(1), INITIAL_GENERATION);
    let layer_id = add(&mut stack, 2, "Test");
    stack.set_item_layer("item1", layer_id);
    assert_eq!(stack.get_item_layer(&"item1"), Some(layer_id));
}

#[cfg(feature = "serde")]
#[test]
fn serde_preserves_backing_uuid_and_rejects_nil() {
    let layer_id = id(42);
    let json = serde_json::to_string(&layer_id).expect("serialize ID");
    let deserialized: LayerId = serde_json::from_str(&json).expect("deserialize ID");
    assert_eq!(layer_id, deserialized);
    assert!(serde_json::from_str::<LayerId>("\"00000000-0000-0000-0000-000000000000\"").is_err());
}

#[cfg(feature = "serde")]
#[test]
fn stack_serialization_round_trip_preserves_generation_and_ownership() {
    let mut stack = stack();
    let layer_id = add(&mut stack, 2, "Test Layer");
    stack.set_item_layer(TEST_ITEM, layer_id);
    let json = serde_json::to_string(&stack).expect("serialize stack");
    let deserialized: LayerStack<u32> = serde_json::from_str(&json).expect("deserialize stack");
    assert_eq!(stack.layer_count(), deserialized.layer_count());
    assert_eq!(stack.generation(), deserialized.generation());
    assert_eq!(stack.get_item_layer(&TEST_ITEM), deserialized.get_item_layer(&TEST_ITEM));
}
