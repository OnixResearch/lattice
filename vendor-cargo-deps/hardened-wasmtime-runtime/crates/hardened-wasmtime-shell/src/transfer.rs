//! Transfer-limit compatibility for the exact Wasmtime 45.0.0 cohort.
//! Wasmtime keeps HostcallFuelExhausted private. Match only its root diagnostic;
//! unknown errors fail closed as RuntimeFailed. Behavioral transfer-bound tests,
//! not wording tests, must pass when changing the pinned Wasmtime source.

const EXHAUSTED: &str = "too much data is being copied between the host and the guest: fuel allocated for hostcalls has been exhausted";

struct Diagnostic<'a> {
    remaining: &'a str,
}
impl std::fmt::Write for Diagnostic<'_> {
    fn write_str(&mut self, text: &str) -> std::fmt::Result {
        self.remaining = self.remaining.strip_prefix(text).ok_or(std::fmt::Error)?;
        Ok(())
    }
}

pub(crate) fn is_exhausted(error: &wasmtime::Error) -> bool {
    let mut diagnostic = Diagnostic {
        remaining: EXHAUSTED,
    };
    if std::fmt::write(&mut diagnostic, format_args!("{}", error.root_cause())).is_err() {
        return false;
    }
    diagnostic.remaining.is_empty()
}
