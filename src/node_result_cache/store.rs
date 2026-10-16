//! Imperative bounded memoizer over pure retention decisions.

const GENERATION_INCREMENT: u64 = 1;
const ACCESS_INCREMENT: u64 = 1;

#[derive(Debug)]
pub enum Error {
    Config(super::core::ConfigError),
    Decision(super::core::DecisionError),
    LockPoisoned,
    GenerationExhausted,
    AccessCounterExhausted,
    PlannedEntryMissing,
    CountOverflow,
}

impl std::fmt::Display for Error {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Config(error) => write!(formatter, "invalid node-result cache config: {error}"),
            Self::Decision(error) => write!(formatter, "node-result cache decision failed: {error}"),
            Self::LockPoisoned => formatter.write_str("node-result cache lock is poisoned"),
            Self::GenerationExhausted => formatter.write_str("node-result cache generation is exhausted"),
            Self::AccessCounterExhausted => formatter.write_str("node-result cache access counter is exhausted"),
            Self::PlannedEntryMissing => formatter.write_str("node-result cache decision selected a missing entry"),
            Self::CountOverflow => formatter.write_str("node-result cache entry count cannot be represented as u64"),
        }
    }
}

impl std::error::Error for Error {}

impl From<super::core::ConfigError> for Error {
    fn from(error: super::core::ConfigError) -> Self {
        Self::Config(error)
    }
}

impl From<super::core::DecisionError> for Error {
    fn from(error: super::core::DecisionError) -> Self {
        Self::Decision(error)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InsertReport {
    pub active_entries: u64,
    pub retained: bool,
    pub released: super::core::release::Plan,
}

#[derive(Debug)]
pub struct Inserted<V> {
    pub value: std::sync::Arc<V>,
    pub report: InsertReport,
}

#[derive(Debug)]
pub enum LookupOrExecuteError<E> {
    Store(Error),
    Executor(E),
}

struct Entry<V> {
    value: std::sync::Arc<V>,
    insertion_generation: u64,
    last_use_generation: u64,
    accesses_since_promotion: u64,
}

struct State<V> {
    entries: std::collections::HashMap<super::core::Key, Entry<V>>,
    generation: u64,
}

pub struct Memoizer<V> {
    config: super::core::AdmittedConfig,
    state: std::sync::Arc<std::sync::Mutex<State<V>>>,
}

impl<V> Memoizer<V> {
    // r[impl lattice.node_result_cache.bound]
    pub fn new(config: super::core::Config) -> Result<Self, Error> {
        let config = super::core::admit_config(config)?;
        Ok(Self {
            config,
            state: std::sync::Arc::new(std::sync::Mutex::new(State {
                entries: std::collections::HashMap::with_capacity(config.capacity_entries.get()),
                generation: 0,
            })),
        })
    }

    pub fn capacity_entries(&self) -> Result<u64, Error> {
        usize_to_u64(self.config.capacity_entries.get())
    }

    pub fn len_entries(&self) -> Result<u64, Error> {
        let guard = self.state.lock().map_err(|_| Error::LockPoisoned)?;
        usize_to_u64(guard.entries.len())
    }

    pub fn is_empty(&self) -> Result<bool, Error> {
        self.len_entries().map(|length| length == 0)
    }

    pub fn contains(&self, key: super::core::Key) -> Result<bool, Error> {
        let guard = self.state.lock().map_err(|_| Error::LockPoisoned)?;
        Ok(guard.entries.contains_key(&key))
    }

    // r[impl lattice.node_result_cache.decision]
    pub fn get(&self, key: super::core::Key) -> Result<Option<std::sync::Arc<V>>, Error> {
        let mut guard = self.state.lock().map_err(|_| Error::LockPoisoned)?;
        let generation = next_generation(guard.generation)?;
        guard.generation = generation;
        let Some(entry) = guard.entries.get_mut(&key) else {
            return Ok(None);
        };
        entry.accesses_since_promotion =
            entry.accesses_since_promotion.checked_add(ACCESS_INCREMENT).ok_or(Error::AccessCounterExhausted)?;
        if super::core::promotion_disposition(super::core::PromotionInput {
            threshold: self.config.promotion_threshold,
            accesses_since_promotion: entry.accesses_since_promotion,
        })? == super::core::PromotionDisposition::Promote
        {
            entry.last_use_generation = generation;
            entry.accesses_since_promotion = 0;
        }
        Ok(Some(std::sync::Arc::clone(&entry.value)))
    }

    // r[impl lattice.node_result_cache.bound]
    // r[impl lattice.node_result_cache.deferral]
    pub fn insert(&self, key: super::core::Key, value: V) -> Result<Inserted<V>, Error> {
        let value = std::sync::Arc::new(value);
        let (report, deferred_values) = {
            let mut guard = self.state.lock().map_err(|_| Error::LockPoisoned)?;
            let generation = next_generation(guard.generation)?;
            guard.generation = generation;
            let mut deferred = Vec::with_capacity(self.config.capacity_entries.get());

            if let Some(replaced) = guard.entries.remove(&key) {
                deferred.push((key, replaced.value));
            } else if guard.entries.len() >= self.config.capacity_entries.get() {
                let eviction =
                    select_from_state(&guard, self.config.promotion_threshold)?.ok_or(Error::PlannedEntryMissing)?;
                let removed = guard.entries.remove(&eviction).ok_or(Error::PlannedEntryMissing)?;
                deferred.push((eviction, removed.value));
            }

            guard.entries.insert(key, Entry {
                value: std::sync::Arc::clone(&value),
                insertion_generation: generation,
                last_use_generation: generation,
                accesses_since_promotion: 0,
            });

            let order = eviction_order(&guard, self.config.promotion_threshold)?;
            let trim = super::core::watermark_trim_plan(super::core::WatermarkInput {
                active_entries: usize_to_u64(guard.entries.len())?,
                low_watermark_entries: usize_to_u64(self.config.low_watermark_entries)?,
                high_watermark_entries: usize_to_u64(self.config.high_watermark_entries)?,
                eviction_order: &order,
            })?;
            for evicted_key in trim.evict {
                let removed = guard.entries.remove(&evicted_key).ok_or(Error::PlannedEntryMissing)?;
                deferred.push((evicted_key, removed.value));
            }

            let released_keys = deferred.iter().map(|(released_key, _)| *released_key).collect();
            let report = InsertReport {
                active_entries: usize_to_u64(guard.entries.len())?,
                retained: guard.entries.contains_key(&key),
                released: super::core::release::plan(released_keys),
            };
            (report, deferred)
        };

        release_after_guard(deferred_values);
        Ok(Inserted { value, report })
    }

    /// Returns the stored value or the supplied executor's result.
    ///
    /// A hit is advisory: it establishes neither node correctness nor input
    /// freshness. Workflow and effect authority remain with the executor.
    // r[impl lattice.node_result_cache.boundary]
    pub fn get_or_execute<E, F>(
        &self,
        key: super::core::Key,
        execute: F,
    ) -> Result<std::sync::Arc<V>, LookupOrExecuteError<E>>
    where
        F: FnOnce() -> Result<V, E>,
    {
        if let Some(value) = self.get(key).map_err(LookupOrExecuteError::Store)? {
            return Ok(value);
        }
        let value = execute().map_err(LookupOrExecuteError::Executor)?;
        self.insert(key, value).map(|inserted| inserted.value).map_err(LookupOrExecuteError::Store)
    }
}

fn usize_to_u64(value: usize) -> Result<u64, Error> {
    u64::try_from(value).map_err(|_| Error::CountOverflow)
}

fn next_generation(current: u64) -> Result<u64, Error> {
    current.checked_add(GENERATION_INCREMENT).ok_or(Error::GenerationExhausted)
}

fn records<V>(state: &State<V>) -> Vec<super::core::RetentionRecord> {
    state
        .entries
        .iter()
        .map(|(key, entry)| super::core::RetentionRecord {
            key: *key,
            insertion_generation: entry.insertion_generation,
            last_use_generation: entry.last_use_generation,
        })
        .collect()
}

fn select_from_state<V>(state: &State<V>, promotion_threshold: u8) -> Result<Option<super::core::Key>, Error> {
    super::core::select_eviction(&records(state), promotion_threshold).map_err(Error::from)
}

fn eviction_order<V>(state: &State<V>, promotion_threshold: u8) -> Result<Vec<super::core::Key>, Error> {
    let mut candidates = records(state);
    let mut order = Vec::with_capacity(candidates.len());
    while !candidates.is_empty() {
        let selected =
            super::core::select_eviction(&candidates, promotion_threshold)?.ok_or(Error::PlannedEntryMissing)?;
        order.push(selected);
        let Some(index) = candidates.iter().position(|record| record.key == selected) else {
            return Err(Error::PlannedEntryMissing);
        };
        candidates.remove(index);
    }
    Ok(order)
}

// r[impl lattice.node_result_cache.deferral]
fn release_after_guard<V>(values: Vec<(super::core::Key, std::sync::Arc<V>)>) {
    drop(values);
}

#[cfg(test)]
mod tests;
