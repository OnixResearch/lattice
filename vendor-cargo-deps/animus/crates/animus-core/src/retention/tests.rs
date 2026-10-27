#[cfg_attr(
    animus_octet,
    allow(
        tigerstyle::non_trait_imports,
        reason = "tests exercise the public retention facade exactly as consumers do"
    )
)]
use super::{
    DIGEST_BYTES, DecisionError, Entry, InsertDisposition, Policy, Projection, PromotionFacts,
    TrimPlan, decide_promotion, plan_insert, plan_trim, project_key, select_oldest,
};

const ZERO: u32 = 0;
const ONE: u32 = 1;
const TWO: u32 = 2;
const THREE: u32 = 3;
const FOUR: u32 = 4;
const THRESHOLD_LRU: u8 = 0;
const BELOW_LAZY_THRESHOLD: u8 = 49;
const THRESHOLD_LAZY: u8 = 50;
const THRESHOLD_FIFO: u8 = 100;
const ABOVE_PERCENT_MAX: u8 = 101;

fn policy(capacity: u32, threshold: u8) -> Policy {
    Policy::new(capacity, capacity, capacity, threshold).expect("valid policy")
}

#[test]
fn projection_is_stable_and_field_sensitive() {
    let request = Projection {
        tool_identity: "search.v1",
        normalized_arguments: "{\"query\":\"cache\"}",
        capability_context: Some("workspace:read"),
    };
    let first = project_key(request).expect("first key");
    let second = project_key(request).expect("second key");
    let distinct = project_key(Projection {
        normalized_arguments: "{\"query\":\"other\"}",
        ..request
    })
    .expect("distinct key");
    let digest_bytes = u32::try_from(first.as_bytes().len()).expect("digest length");

    assert_eq!(first, second);
    assert_ne!(first, distinct);
    assert_eq!(digest_bytes, DIGEST_BYTES);
}

#[test]
fn projection_rejects_blank_identity_and_context() {
    let blank_identity = project_key(Projection {
        tool_identity: " ",
        normalized_arguments: "{}",
        capability_context: None,
    });
    let blank_context = project_key(Projection {
        tool_identity: "search.v1",
        normalized_arguments: "{}",
        capability_context: Some(" "),
    });

    assert_eq!(blank_identity, Err(DecisionError::EmptyToolIdentity));
    assert_eq!(blank_context, Err(DecisionError::EmptyCapabilityContext));
}

#[test]
fn policy_rejects_reversed_and_out_of_range_bounds() {
    assert_eq!(
        Policy::new(TWO, TWO, ONE, THRESHOLD_LRU),
        Err(DecisionError::ReversedWatermarks)
    );
    assert_eq!(
        Policy::new(ONE, ZERO, TWO, THRESHOLD_LRU),
        Err(DecisionError::WatermarkExceedsCapacity)
    );
    assert_eq!(
        Policy::new(ONE, ONE, ONE, ABOVE_PERCENT_MAX),
        Err(DecisionError::InvalidPercentage)
    );
}

#[test]
fn zero_and_single_slot_insertions_are_explicit() {
    let zero_plan =
        plan_insert(policy(ZERO, THRESHOLD_LRU), ZERO, &[], &[]).expect("zero-capacity plan");
    let free_plan =
        plan_insert(policy(ONE, THRESHOLD_LRU), ZERO, &[ZERO], &[]).expect("free-slot plan");
    let eviction_plan = plan_insert(
        policy(ONE, THRESHOLD_LRU),
        ONE,
        &[],
        &[Entry {
            slot: ZERO,
            promotion_generation: u64::from(ONE),
        }],
    )
    .expect("eviction plan");

    assert_eq!(zero_plan.disposition, InsertDisposition::DoNotRetain);
    assert!(zero_plan.release.slots().is_empty());
    assert_eq!(free_plan.disposition, InsertDisposition::UseFreeSlot(ZERO));
    assert_eq!(
        eviction_plan.disposition,
        InsertDisposition::EvictSlot(ZERO)
    );
    assert_eq!(eviction_plan.release.slots(), &[ZERO]);
}

#[test]
fn eviction_uses_oldest_generation_then_smallest_slot() {
    let entries = [
        Entry {
            slot: TWO,
            promotion_generation: u64::from(THREE),
        },
        Entry {
            slot: ONE,
            promotion_generation: u64::from(ONE),
        },
        Entry {
            slot: ZERO,
            promotion_generation: u64::from(ONE),
        },
    ];

    assert_eq!(select_oldest(&entries), Some(ZERO));
}

#[test]
fn promotion_boundaries_cover_lru_lazy_and_fifo() {
    let below_threshold = PromotionFacts {
        age_percent: BELOW_LAZY_THRESHOLD,
    };
    let at_threshold = PromotionFacts {
        age_percent: THRESHOLD_LAZY,
    };

    assert_eq!(
        decide_promotion(policy(ONE, THRESHOLD_LRU), below_threshold),
        Ok(true)
    );
    assert_eq!(
        decide_promotion(policy(ONE, THRESHOLD_LAZY), below_threshold),
        Ok(false)
    );
    assert_eq!(
        decide_promotion(policy(ONE, THRESHOLD_LAZY), at_threshold),
        Ok(true)
    );
    assert_eq!(
        decide_promotion(policy(ONE, THRESHOLD_FIFO), at_threshold),
        Ok(false)
    );
}

#[test]
fn watermark_trim_is_bounded_and_rejects_impossible_count() {
    let trim_policy = Policy::new(THREE, ONE, TWO, THRESHOLD_LRU).expect("watermark policy");

    assert_eq!(
        plan_trim(trim_policy, TWO),
        Ok(TrimPlan {
            release_count: ZERO
        })
    );
    assert_eq!(
        plan_trim(trim_policy, THREE),
        Ok(TrimPlan { release_count: TWO })
    );
    assert_eq!(
        plan_trim(trim_policy, FOUR),
        Err(DecisionError::ActiveCountExceedsCapacity)
    );
}

#[test]
fn insertion_rejects_missing_and_out_of_range_state() {
    assert_eq!(
        plan_insert(policy(TWO, THRESHOLD_LRU), ONE, &[], &[]),
        Err(DecisionError::MissingFreeSlot)
    );
    assert_eq!(
        plan_insert(policy(ONE, THRESHOLD_LRU), ONE, &[], &[]),
        Err(DecisionError::MissingEvictionCandidate)
    );
    assert_eq!(
        plan_insert(policy(ONE, THRESHOLD_LRU), ZERO, &[ONE], &[]),
        Err(DecisionError::SlotOutsideCapacity)
    );
}
