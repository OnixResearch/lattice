//! Shared engine concurrency permits release on every return or unwind.
pub(crate) struct Permit<'a>(pub &'a std::sync::atomic::AtomicU32);
impl Drop for Permit<'_> {
    fn drop(&mut self) {
        self.0.fetch_sub(1, std::sync::atomic::Ordering::Release);
    }
}

/// Ownership transferred into a retained invocation, including before its first poll.
pub(crate) struct OwnedPermit(pub std::sync::Arc<std::sync::atomic::AtomicU32>);
impl Drop for OwnedPermit {
    fn drop(&mut self) {
        self.0.fetch_sub(1, std::sync::atomic::Ordering::Release);
    }
}

/// Reserve one shared engine slot for either borrowed or owned permit lifetime.
pub(crate) fn reserve(counter: &std::sync::atomic::AtomicU32, limit: u32) -> bool {
    counter
        .fetch_update(
            std::sync::atomic::Ordering::AcqRel,
            std::sync::atomic::Ordering::Acquire,
            |active| {
                if active < limit {
                    active.checked_add(1)
                } else {
                    None
                }
            },
        )
        .is_ok()
}
