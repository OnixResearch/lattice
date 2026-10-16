use std::sync::Weak;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use super::*;

const CAPACITY: usize = 2;
const WATERMARK_CAPACITY: usize = 3;
const SINGLE_SLOT_CAPACITY: usize = 1;
const LOW_WATERMARK: usize = 1;
const HIGH_WATERMARK: usize = 2;
const MID_PROMOTION_THRESHOLD: u8 = 2;
const FIRST_KEY_BYTE: u8 = 1;
const SECOND_KEY_BYTE: u8 = 2;
const THIRD_KEY_BYTE: u8 = 3;

fn key(byte: u8) -> super::super::core::Key {
    super::super::core::Key::from_bytes([byte; blake3::OUT_LEN])
}

fn config(capacity_entries: Option<usize>, threshold: u8) -> super::super::core::Config {
    super::super::core::Config {
        capacity_entries,
        promotion_threshold: threshold,
        low_watermark_entries: LOW_WATERMARK,
        high_watermark_entries: capacity_entries.unwrap_or(HIGH_WATERMARK).min(HIGH_WATERMARK),
    }
}

// r[verify lattice.node_result_cache.bound]
// r[verify lattice.node_result_cache.decision]
// r[verify lattice.node_result_cache.verification]
#[test]
fn strict_lru_and_fifo_choose_different_evictions() -> Result<(), Error> {
    let lru = Memoizer::new(config(Some(CAPACITY), super::super::core::STRICT_LRU_PROMOTION_THRESHOLD))?;
    drop(lru.insert(key(FIRST_KEY_BYTE), "one")?.value);
    drop(lru.insert(key(SECOND_KEY_BYTE), "two")?.value);
    drop(lru.get(key(FIRST_KEY_BYTE))?);
    drop(lru.insert(key(THIRD_KEY_BYTE), "three")?.value);
    // r[verify lattice.node_result_cache.bound.scenario.full]
    assert!(lru.contains(key(FIRST_KEY_BYTE))?);
    assert!(!lru.contains(key(SECOND_KEY_BYTE))?);
    assert_eq!(lru.len_entries()?, u64::try_from(CAPACITY).map_err(|_| Error::CountOverflow)?);

    let fifo = Memoizer::new(config(Some(CAPACITY), super::super::core::FIFO_PROMOTION_THRESHOLD))?;
    drop(fifo.insert(key(FIRST_KEY_BYTE), "one")?.value);
    drop(fifo.insert(key(SECOND_KEY_BYTE), "two")?.value);
    drop(fifo.get(key(FIRST_KEY_BYTE))?);
    drop(fifo.insert(key(THIRD_KEY_BYTE), "three")?.value);
    assert!(!fifo.contains(key(FIRST_KEY_BYTE))?);
    assert!(fifo.contains(key(SECOND_KEY_BYTE))?);
    Ok(())
}

// r[verify lattice.node_result_cache.decision]
// r[verify lattice.node_result_cache.verification]
#[test]
fn lazy_promotion_occurs_at_but_not_below_threshold() -> Result<(), Error> {
    let below = Memoizer::new(config(Some(CAPACITY), MID_PROMOTION_THRESHOLD))?;
    drop(below.insert(key(FIRST_KEY_BYTE), "one")?.value);
    drop(below.insert(key(SECOND_KEY_BYTE), "two")?.value);
    drop(below.get(key(FIRST_KEY_BYTE))?);
    drop(below.insert(key(THIRD_KEY_BYTE), "three")?.value);
    assert!(!below.contains(key(FIRST_KEY_BYTE))?);

    let at = Memoizer::new(config(Some(CAPACITY), MID_PROMOTION_THRESHOLD))?;
    drop(at.insert(key(FIRST_KEY_BYTE), "one")?.value);
    drop(at.insert(key(SECOND_KEY_BYTE), "two")?.value);
    drop(at.get(key(FIRST_KEY_BYTE))?);
    drop(at.get(key(FIRST_KEY_BYTE))?);
    drop(at.insert(key(THIRD_KEY_BYTE), "three")?.value);
    assert!(at.contains(key(FIRST_KEY_BYTE))?);
    assert!(!at.contains(key(SECOND_KEY_BYTE))?);
    Ok(())
}

// r[verify lattice.node_result_cache.bound]
// r[verify lattice.node_result_cache.verification]
#[test]
fn watermark_overflow_trims_to_the_declared_low_mark() -> Result<(), Error> {
    let memoizer = Memoizer::new(config(Some(WATERMARK_CAPACITY), super::super::core::STRICT_LRU_PROMOTION_THRESHOLD))?;
    drop(memoizer.insert(key(FIRST_KEY_BYTE), "one")?.value);
    drop(memoizer.insert(key(SECOND_KEY_BYTE), "two")?.value);
    let inserted = memoizer.insert(key(THIRD_KEY_BYTE), "three")?;
    drop(inserted.value);
    // r[verify lattice.node_result_cache.bound.scenario.configured]
    assert_eq!(inserted.report.active_entries, u64::try_from(LOW_WATERMARK).map_err(|_| Error::CountOverflow)?);
    assert_eq!(memoizer.len_entries()?, u64::try_from(LOW_WATERMARK).map_err(|_| Error::CountOverflow)?);
    assert!(memoizer.contains(key(THIRD_KEY_BYTE))?);
    Ok(())
}

// r[verify lattice.node_result_cache.boundary]
// r[verify lattice.node_result_cache.verification]
#[test]
fn misses_execute_once_and_executor_errors_are_preserved() -> Result<(), Error> {
    let memoizer =
        Memoizer::new(config(Some(SINGLE_SLOT_CAPACITY), super::super::core::STRICT_LRU_PROMOTION_THRESHOLD))?;
    let calls = AtomicUsize::new(0);
    let first = memoizer
        .get_or_execute(key(FIRST_KEY_BYTE), || {
            calls.fetch_add(1, Ordering::SeqCst);
            Ok::<_, &'static str>("value")
        })
        .map_err(map_execution_error)?;
    let second = memoizer
        .get_or_execute(key(FIRST_KEY_BYTE), || {
            calls.fetch_add(1, Ordering::SeqCst);
            Ok::<_, &'static str>("other")
        })
        .map_err(map_execution_error)?;
    assert_eq!(*first, "value");
    assert_eq!(*second, "value");
    assert_eq!(calls.load(Ordering::SeqCst), 1);

    let error = memoizer.get_or_execute(key(SECOND_KEY_BYTE), || Err::<&str, _>("executor-error"));
    assert!(matches!(error, Err(LookupOrExecuteError::Executor("executor-error"))));
    Ok(())
}

fn map_execution_error(error: LookupOrExecuteError<&'static str>) -> Error {
    match error {
        LookupOrExecuteError::Store(store_error) => store_error,
        LookupOrExecuteError::Executor(_) => Error::PlannedEntryMissing,
    }
}

struct DropProbe {
    state: Weak<std::sync::Mutex<State<DropProbe>>>,
    dropped_after_unlock: std::sync::Arc<AtomicBool>,
}

impl Drop for DropProbe {
    fn drop(&mut self) {
        let unlocked = self.state.upgrade().is_some_and(|state| state.try_lock().is_ok());
        self.dropped_after_unlock.store(unlocked, Ordering::SeqCst);
    }
}

// r[verify lattice.node_result_cache.deferral]
// r[verify lattice.node_result_cache.verification]
#[test]
fn evicted_value_drops_after_guard_release() -> Result<(), Error> {
    let memoizer =
        Memoizer::new(config(Some(SINGLE_SLOT_CAPACITY), super::super::core::STRICT_LRU_PROMOTION_THRESHOLD))?;
    let observed = std::sync::Arc::new(AtomicBool::new(false));
    let first = DropProbe {
        state: std::sync::Arc::downgrade(&memoizer.state),
        dropped_after_unlock: std::sync::Arc::clone(&observed),
    };
    drop(memoizer.insert(key(FIRST_KEY_BYTE), first)?.value);

    let second = DropProbe {
        state: std::sync::Arc::downgrade(&memoizer.state),
        dropped_after_unlock: std::sync::Arc::new(AtomicBool::new(false)),
    };
    drop(memoizer.insert(key(SECOND_KEY_BYTE), second)?.value);
    // r[verify lattice.node_result_cache.deferral.scenario.evicted]
    assert!(observed.load(Ordering::SeqCst));
    Ok(())
}

// r[verify lattice.node_result_cache.bound]
// r[verify lattice.node_result_cache.verification]
#[test]
fn invalid_bounds_fail_before_store_creation() {
    assert!(matches!(
        Memoizer::<String>::new(config(None, super::super::core::STRICT_LRU_PROMOTION_THRESHOLD)),
        Err(Error::Config(super::super::core::ConfigError::MissingCapacity))
    ));
    assert!(matches!(
        Memoizer::<String>::new(super::super::core::Config {
            capacity_entries: Some(CAPACITY),
            promotion_threshold: super::super::core::STRICT_LRU_PROMOTION_THRESHOLD,
            low_watermark_entries: HIGH_WATERMARK,
            high_watermark_entries: LOW_WATERMARK,
        }),
        Err(Error::Config(super::super::core::ConfigError::ReversedWatermarks))
    ));
}
