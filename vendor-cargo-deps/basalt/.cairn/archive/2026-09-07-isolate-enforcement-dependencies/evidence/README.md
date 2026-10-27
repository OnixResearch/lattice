# Historical incomplete profile qualification

This records the earlier blocked draft. See `ADOPTION.md` for the later published
UCAN profile and current qualification evidence. The failures below are preserved.

Base: published Basalt `27547d7870fdb5ca21208d583b58a518ba820b72`.
The 30-test default enforcement baseline passed before edits.

The candidate gates Basalt's own HTTP and durable-state modules and dependencies,
with both features still default. However, an actual pinned Rust 1.95 run still
fails: UCAN itself unconditionally depends on replay-ledger-core and
revocation-view-core, which require Rust 1.96. The HTTP packages no longer cause
that compiler rejection, but this is not complete dependency isolation.

Published UCAN `a7d2c21cbb9a262c048672e09c77c3f0825bdd2d` was also inspected. Its
Cargo manifest still has both unconditional durable dependencies and no feature
split. Merely updating the UCAN snapshot does not resolve the observed boundary.
No vendored file or source pin was changed. No MSRV bypass or toolchain-wide upgrade.

On the owner Rust 1.96 toolchain, the post-change default enforcement tests passed
30/30. The explicit minimal profile passed 109 library tests and 28 enforcement
tests. Eleven release-runner tests passed, including empty/forbidden dependency
rejection. These results do not establish Rust 1.95 compatibility. A broadened
`cargo fmt --all -- --check` found existing vendored UCAN formatting differences;
no vendored file was reformatted. The maintained root-only format check is separate.

Draft source: `987e9e9cce5fc91d5486a135e74684c92c1e8af0`.
Maintained root formatting and strict all-target Clippy passed for both all-features
and no-default profiles on Rust 1.96. Vendor, Cargo lock and Nix lock stayed unchanged.
The complete direct release then stopped at `minimal enforcement dependencies`:
the Cargo command exited zero, but semantic validation failed with
`minimal dependency observation includes replay_ledger_core`. Earlier freshness
checks and minimal tests passed. No release acceptance is inferred from child exit.

All tasks remain unchecked. This change is not ready to publish, sync or archive.
The next owner contract is a selectable UCAN durable-authority adapter, preserving
verification services and default behavior. Qualify and publish that contract before
refreshing Basalt's retained vendor cohort and retrying Rust 1.95.

Logs: initiating Neural checkout `.pi/drains/2026-09-07-basalt-minimal/`.
No operator grant, native execution, or Neural task completion is claimed.
