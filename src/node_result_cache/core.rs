//! Pure identity and retention decisions for advisory memoization.

pub mod release;
pub mod slot;

const KEY_DOMAIN: &[u8] = b"lattice-node-result-cache:v1";
const NODE_IDENTITY_LABEL: &[u8] = b"node-identity";
const CONTRACT_REVISION_LABEL: &[u8] = b"operation-contract-revision";
const NORMALIZED_INPUTS_LABEL: &[u8] = b"normalized-inputs";
const CAPABILITY_CONTEXT_LABEL: &[u8] = b"capability-context";
pub const STRICT_LRU_PROMOTION_THRESHOLD: u8 = 0;
pub const FIFO_PROMOTION_THRESHOLD: u8 = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Key([u8; blake3::OUT_LEN]);

impl Key {
    pub const fn from_bytes(bytes: [u8; blake3::OUT_LEN]) -> Self {
        Self(bytes)
    }

    pub const fn as_bytes(&self) -> &[u8; blake3::OUT_LEN] {
        &self.0
    }

    pub fn to_hex(self) -> String {
        blake3::Hash::from_bytes(self.0).to_hex().to_string()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RequestProjection<'a> {
    pub node_identity: &'a [u8],
    pub operation_contract_revision: &'a [u8],
    pub normalized_inputs: &'a [u8],
    pub capability_context: Option<&'a [u8]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectionError {
    ComponentTooLarge,
}

impl std::fmt::Display for ProjectionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ComponentTooLarge => formatter.write_str("cache key component exceeds the framed u64 length domain"),
        }
    }
}

impl std::error::Error for ProjectionError {}

// r[impl lattice.node_result_cache.projection]
pub fn project_key(request: RequestProjection<'_>) -> Result<Key, ProjectionError> {
    let mut hasher = blake3::Hasher::new();
    hasher.update(KEY_DOMAIN);
    update_component(&mut hasher, NODE_IDENTITY_LABEL, request.node_identity)?;
    update_component(&mut hasher, CONTRACT_REVISION_LABEL, request.operation_contract_revision)?;
    update_component(&mut hasher, NORMALIZED_INPUTS_LABEL, request.normalized_inputs)?;
    match request.capability_context {
        Some(context) => {
            hasher.update(&[1]);
            update_component(&mut hasher, CAPABILITY_CONTEXT_LABEL, context)?;
        }
        None => {
            hasher.update(&[0]);
        }
    }
    Ok(Key::from_bytes(*hasher.finalize().as_bytes()))
}

fn update_component(hasher: &mut blake3::Hasher, label: &[u8], value: &[u8]) -> Result<(), ProjectionError> {
    let label_len = u64::try_from(label.len()).map_err(|_| ProjectionError::ComponentTooLarge)?;
    let value_len = u64::try_from(value.len()).map_err(|_| ProjectionError::ComponentTooLarge)?;
    hasher.update(&label_len.to_be_bytes());
    hasher.update(label);
    hasher.update(&value_len.to_be_bytes());
    hasher.update(value);
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Config {
    pub capacity_entries: Option<usize>,
    pub promotion_threshold: u8,
    pub low_watermark_entries: usize,
    pub high_watermark_entries: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdmittedConfig {
    pub capacity_entries: std::num::NonZeroUsize,
    pub promotion_threshold: u8,
    pub low_watermark_entries: usize,
    pub high_watermark_entries: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigError {
    MissingCapacity,
    ZeroCapacity,
    PromotionThresholdOutOfRange,
    ReversedWatermarks,
    HighWatermarkExceedsCapacity,
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::MissingCapacity => "node-result cache capacity is required",
            Self::ZeroCapacity => "node-result cache capacity must be positive",
            Self::PromotionThresholdOutOfRange => "promotion threshold must be between 0 and 100",
            Self::ReversedWatermarks => "low watermark must not exceed high watermark",
            Self::HighWatermarkExceedsCapacity => "high watermark must not exceed capacity",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for ConfigError {}

// r[impl lattice.node_result_cache.bound]
pub fn admit_config(config: Config) -> Result<AdmittedConfig, ConfigError> {
    let capacity_entries = std::num::NonZeroUsize::new(config.capacity_entries.ok_or(ConfigError::MissingCapacity)?)
        .ok_or(ConfigError::ZeroCapacity)?;
    if config.promotion_threshold > FIFO_PROMOTION_THRESHOLD {
        return Err(ConfigError::PromotionThresholdOutOfRange);
    }
    if config.low_watermark_entries > config.high_watermark_entries {
        return Err(ConfigError::ReversedWatermarks);
    }
    if config.high_watermark_entries > capacity_entries.get() {
        return Err(ConfigError::HighWatermarkExceedsCapacity);
    }
    Ok(AdmittedConfig {
        capacity_entries,
        promotion_threshold: config.promotion_threshold,
        low_watermark_entries: config.low_watermark_entries,
        high_watermark_entries: config.high_watermark_entries,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PromotionDisposition {
    Promote,
    RetainPosition,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PromotionInput {
    pub threshold: u8,
    pub accesses_since_promotion: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecisionError {
    PromotionThresholdOutOfRange,
    InvalidFreeSlot,
    MissingEvictionTarget,
    ReversedWatermarks,
    IncompleteEvictionOrder,
    DuplicateEvictionKey,
    CountOverflow,
}

impl std::fmt::Display for DecisionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::PromotionThresholdOutOfRange => "promotion threshold must be between 0 and 100",
            Self::InvalidFreeSlot => "free-list slot is outside the declared capacity",
            Self::MissingEvictionTarget => "a full cache requires an eviction target",
            Self::ReversedWatermarks => "low watermark must not exceed high watermark",
            Self::IncompleteEvictionOrder => "eviction order is shorter than the trim requirement",
            Self::DuplicateEvictionKey => "eviction order contains a duplicate key",
            Self::CountOverflow => "cache entry count cannot be represented on this platform",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for DecisionError {}

// r[impl lattice.node_result_cache.decision]
pub fn promotion_disposition(input: PromotionInput) -> Result<PromotionDisposition, DecisionError> {
    if input.threshold > FIFO_PROMOTION_THRESHOLD {
        return Err(DecisionError::PromotionThresholdOutOfRange);
    }
    if input.threshold == FIFO_PROMOTION_THRESHOLD {
        return Ok(PromotionDisposition::RetainPosition);
    }
    if input.threshold == STRICT_LRU_PROMOTION_THRESHOLD || input.accesses_since_promotion >= u64::from(input.threshold)
    {
        Ok(PromotionDisposition::Promote)
    } else {
        Ok(PromotionDisposition::RetainPosition)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetentionRecord {
    pub key: Key,
    pub insertion_generation: u64,
    pub last_use_generation: u64,
}

// r[impl lattice.node_result_cache.decision]
pub fn select_eviction(records: &[RetentionRecord], promotion_threshold: u8) -> Result<Option<Key>, DecisionError> {
    if promotion_threshold > FIFO_PROMOTION_THRESHOLD {
        return Err(DecisionError::PromotionThresholdOutOfRange);
    }
    let selected = records.iter().min_by_key(|record| {
        let policy_generation = if promotion_threshold == FIFO_PROMOTION_THRESHOLD {
            record.insertion_generation
        } else {
            record.last_use_generation
        };
        (policy_generation, record.insertion_generation, record.key)
    });
    Ok(selected.map(|record| record.key))
}

#[derive(Debug, Clone, Copy)]
pub struct WatermarkInput<'a> {
    pub active_entries: u64,
    pub low_watermark_entries: u64,
    pub high_watermark_entries: u64,
    pub eviction_order: &'a [Key],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrimPlan {
    pub evict: Vec<Key>,
}

// r[impl lattice.node_result_cache.decision]
pub fn watermark_trim_plan(input: WatermarkInput<'_>) -> Result<TrimPlan, DecisionError> {
    if input.low_watermark_entries > input.high_watermark_entries {
        return Err(DecisionError::ReversedWatermarks);
    }
    if input.active_entries <= input.high_watermark_entries {
        return Ok(TrimPlan { evict: Vec::new() });
    }
    debug_assert!(input.active_entries > input.high_watermark_entries);
    debug_assert!(input.low_watermark_entries <= input.active_entries);
    let remove_count = input
        .active_entries
        .checked_sub(input.low_watermark_entries)
        .ok_or(DecisionError::ReversedWatermarks)?;
    let remove_count = usize::try_from(remove_count).map_err(|_| DecisionError::CountOverflow)?;
    if input.eviction_order.len() < remove_count {
        return Err(DecisionError::IncompleteEvictionOrder);
    }
    let mut seen = std::collections::BTreeSet::new();
    let mut evict = Vec::with_capacity(remove_count);
    for key in input.eviction_order.iter().take(remove_count) {
        if !seen.insert(*key) {
            return Err(DecisionError::DuplicateEvictionKey);
        }
        evict.push(*key);
    }
    Ok(TrimPlan { evict })
}

#[cfg(test)]
mod tests;
