//! Preserve actual Wasmtime limit rejection independently of trap strings.
// r[impl onix.wasmtime_runtime.independent_memory_limits]
use wasmtime::ResourceLimiter;

pub(crate) struct Limits {
    pub inner: wasmtime::StoreLimits,
    pub is_denied: bool,
    /// Remaining cumulative allocation budget, independent of per-memory limits.
    pub memory_bytes_remaining: usize,
}
impl ResourceLimiter for Limits {
    fn memory_growing(
        &mut self,
        current: usize,
        desired: usize,
        maximum: Option<usize>,
    ) -> wasmtime::Result<bool> {
        let result = self.inner.memory_growing(current, desired, maximum);
        if !matches!(result, Ok(true)) {
            self.is_denied = true;
            return result;
        }
        let remaining = desired
            .checked_sub(current)
            .and_then(|growth| self.memory_bytes_remaining.checked_sub(growth));
        let Some(bytes) = remaining else {
            self.is_denied = true;
            return Ok(false);
        };
        self.memory_bytes_remaining = bytes;
        Ok(true)
    }
    fn table_growing(
        &mut self,
        current: usize,
        desired: usize,
        maximum: Option<usize>,
    ) -> wasmtime::Result<bool> {
        let result = self.inner.table_growing(current, desired, maximum);
        self.is_denied |= !matches!(result, Ok(true));
        result
    }
    fn memory_grow_failed(&mut self, error: wasmtime::Error) -> wasmtime::Result<()> {
        self.is_denied = true;
        self.inner.memory_grow_failed(error)
    }
    fn table_grow_failed(&mut self, error: wasmtime::Error) -> wasmtime::Result<()> {
        self.is_denied = true;
        self.inner.table_grow_failed(error)
    }
    fn instances(&self) -> usize {
        self.inner.instances()
    }
    fn memories(&self) -> usize {
        self.inner.memories()
    }
    fn tables(&self) -> usize {
        self.inner.tables()
    }
}
