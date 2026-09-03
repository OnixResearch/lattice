use super::*;

const CAPACITY: usize = 3;
const LOW_WATERMARK: usize = 1;
const HIGH_WATERMARK: usize = 2;
const MID_PROMOTION_THRESHOLD: u8 = 2;
const ONE_ACCESS: u64 = 1;
const TWO_ACCESSES: u64 = 2;
const HEX_CHARS_PER_BYTE: usize = 2;
const FIRST_KEY_BYTE: u8 = 1;
const SECOND_KEY_BYTE: u8 = 2;
const THIRD_KEY_BYTE: u8 = 3;

fn key(byte: u8) -> Key {
    Key::from_bytes([byte; blake3::OUT_LEN])
}

fn projection(inputs: &[u8]) -> RequestProjection<'_> {
    RequestProjection {
        node_identity: b"node-a",
        operation_contract_revision: b"contract-v1",
        normalized_inputs: inputs,
        capability_context: Some(b"capability-v1"),
    }
}

// r[verify lattice.node_result_cache.projection]
// r[verify lattice.node_result_cache.verification]
#[test]
fn projection_is_stable_and_input_sensitive() -> Result<(), Box<dyn std::error::Error>> {
    let first = project_key(projection(b"input-a"))?;
    let second = project_key(projection(b"input-a"))?;
    let changed = project_key(projection(b"input-b"))?;
    assert_eq!(first, second);
    assert_ne!(first, changed);
    assert_eq!(first.to_hex().len(), blake3::OUT_LEN * HEX_CHARS_PER_BYTE);
    Ok(())
}

// r[verify lattice.node_result_cache.bound]
// r[verify lattice.node_result_cache.verification]
#[test]
fn config_requires_explicit_valid_bounds() -> Result<(), Box<dyn std::error::Error>> {
    let valid = admit_config(Config {
        capacity_entries: Some(CAPACITY),
        promotion_threshold: MID_PROMOTION_THRESHOLD,
        low_watermark_entries: LOW_WATERMARK,
        high_watermark_entries: HIGH_WATERMARK,
    })?;
    assert_eq!(valid.capacity_entries.get(), CAPACITY);

    let missing = Config {
        capacity_entries: None,
        promotion_threshold: STRICT_LRU_PROMOTION_THRESHOLD,
        low_watermark_entries: 0,
        high_watermark_entries: 0,
    };
    assert_eq!(admit_config(missing), Err(ConfigError::MissingCapacity));
    assert_eq!(
        admit_config(Config {
            capacity_entries: Some(0),
            ..missing
        }),
        Err(ConfigError::ZeroCapacity)
    );
    assert_eq!(
        admit_config(Config {
            capacity_entries: Some(CAPACITY),
            low_watermark_entries: HIGH_WATERMARK,
            high_watermark_entries: LOW_WATERMARK,
            ..missing
        }),
        Err(ConfigError::ReversedWatermarks)
    );
    Ok(())
}

// r[verify lattice.node_result_cache.decision]
// r[verify lattice.node_result_cache.verification]
#[test]
fn promotion_boundaries_and_middle_threshold_are_deterministic() {
    assert_eq!(
        promotion_disposition(PromotionInput {
            threshold: STRICT_LRU_PROMOTION_THRESHOLD,
            accesses_since_promotion: 0,
        }),
        Ok(PromotionDisposition::Promote)
    );
    assert_eq!(
        promotion_disposition(PromotionInput {
            threshold: FIFO_PROMOTION_THRESHOLD,
            accesses_since_promotion: u64::MAX,
        }),
        Ok(PromotionDisposition::RetainPosition)
    );
    assert_eq!(
        promotion_disposition(PromotionInput {
            threshold: MID_PROMOTION_THRESHOLD,
            accesses_since_promotion: ONE_ACCESS,
        }),
        Ok(PromotionDisposition::RetainPosition)
    );
    assert_eq!(
        promotion_disposition(PromotionInput {
            threshold: MID_PROMOTION_THRESHOLD,
            accesses_since_promotion: TWO_ACCESSES,
        }),
        Ok(PromotionDisposition::Promote)
    );
}

// r[verify lattice.node_result_cache.decision]
// r[verify lattice.node_result_cache.verification]
#[test]
fn slot_and_trim_plans_cover_positive_and_negative_boundaries() -> Result<(), Box<dyn std::error::Error>> {
    let capacity_entries = std::num::NonZeroUsize::new(CAPACITY).ok_or(ConfigError::ZeroCapacity)?;
    assert_eq!(
        slot::plan(slot::Input {
            capacity_entries,
            active_entries: LOW_WATERMARK,
            free_slots: &[HIGH_WATERMARK],
            eviction_target: None,
        }),
        Ok(slot::Plan::UseFree { slot: HIGH_WATERMARK })
    );
    assert_eq!(
        slot::plan(slot::Input {
            capacity_entries,
            active_entries: CAPACITY,
            free_slots: &[],
            eviction_target: None,
        }),
        Err(DecisionError::MissingEvictionTarget)
    );

    let order = [key(FIRST_KEY_BYTE), key(SECOND_KEY_BYTE), key(THIRD_KEY_BYTE)];
    let trim = watermark_trim_plan(WatermarkInput {
        active_entries: u64::try_from(CAPACITY)?,
        low_watermark_entries: u64::try_from(LOW_WATERMARK)?,
        high_watermark_entries: u64::try_from(HIGH_WATERMARK)?,
        eviction_order: &order,
    })?;
    assert_eq!(trim.evict, vec![key(FIRST_KEY_BYTE), key(SECOND_KEY_BYTE)]);
    assert_eq!(
        watermark_trim_plan(WatermarkInput {
            active_entries: u64::try_from(CAPACITY)?,
            low_watermark_entries: u64::try_from(HIGH_WATERMARK)?,
            high_watermark_entries: u64::try_from(LOW_WATERMARK)?,
            eviction_order: &order,
        }),
        Err(DecisionError::ReversedWatermarks)
    );
    Ok(())
}
