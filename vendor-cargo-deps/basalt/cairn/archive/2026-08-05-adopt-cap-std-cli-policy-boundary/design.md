## Design

Basalt will open ambient policy input, generated policy, and receipt/output directories only in the CLI feature path. The CLI converts those directories into typed wrappers around `cap_std::fs::Dir`, then passes loaded policy bytes, parsed DTOs, or validated relative locators into library functions.

The library enforcement core, UCAN validation, Nickel/Steel backend DTOs, and packaged `DEFAULT_POLICY_JSON` helpers remain independent of `cap-std`. No-default-feature consumers should not compile CLI filesystem boundary code.

The first conversion targets are CLI policy-file reads and receipt/output writes used by `basalt enforce` and release/verification helpers where they are owned by the crate rather than external shell scripts.

Tests should cover valid policy artifacts, packaged policy fallback, traversal, absolute paths, symlink escapes, missing roots, and output escape attempts. Documentation should state that capability roots bound local CLI filesystem authority only; they do not prove policy truth, UCAN issuer trust, caveat semantics, or downstream deployment readiness.
