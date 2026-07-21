# Post-failure run-store recovery evidence

Date: 2026-07-21

## Root cause

The live restart drill observed one `runs show` failure immediately after a writer panicked and success after a normal writable reopen. redb 3.1.1 source establishes the mechanism:

- `ReadOnlyDatabase` returns `DatabaseError::RepairAborted` when allocator state is missing because read-only mode cannot repair the file.
- `Database::drop` skips allocator-state finalization while `thread::panicking()` is true, while still closing the underlying memory/file state.
- `Builder::open` opens an existing file read/write without create and invokes its repair callback at least once only when repair is needed.

The failure was therefore not treated as a transient lock by assumption. Lattice now matches redb's typed `DatabaseAlreadyOpen` and `RepairAborted` variants separately.

## Selected behavior

- `runs show` remains exclusively backed by `ReadOnlyDatabase`.
- Active ownership renders a fixed retry-after-exit diagnostic.
- Recovery-required state renders a fixed instruction to run `runs repair` with the same configuration.
- Every other open failure renders fixed unavailable text.
- No raw redb, I/O, lock, process, or path text is included.
- `runs repair` uses `Builder::open`, never create, and opens no Lattice tables.
- A repair callback records whether redb actually repaired the file, allowing distinct fixed `recovery completed` and `already clean` receipts.

## Process fixtures

`tests/test_run_store_recovery.rs` self-spawns the compiled test executable across real process boundaries:

1. Clean writer exit persists one run; immediate safe inspection succeeds; explicit repair reports `already clean`; inspection remains valid.
2. Panic unwind with `Store` still in scope persists one run but skips clean allocator finalization; read-only inspection reports recovery required.
3. Abrupt process abort persists one run and produces the same recovery-required classification.
4. A holding writer owns the file while parent inspection and repair both fail as active-owner; after explicit release, inspection succeeds.
5. Unknown child phase fails before opening or creating a store.

For panic and abort, BLAKE3 identity of the store was equal before and after failed `runs show`, proving the inspection attempt did not change file content. Explicit repair then succeeded, safe JSON inspection validated, and exactly the original `RunRecord` remained. For active ownership, BLAKE3 identity stayed equal across rejected inspection and repair attempts while the child remained idle.

Missing data directories remained absent after `runs repair`. A malformed file retained the same BLAKE3 identity, its private marker was omitted from stderr, and repair returned fixed unavailable text. Fixed-error scans also rejected workflow markers, raw `Database repair aborted`, raw `Cannot acquire lock`, and configured store paths.

## Validation evidence

- Pre-change safe-inspection baseline: 11 passed.
- Post-change safe-inspection suite: 11 passed.
- New process recovery suite: 5 passed, including positive and negative cases.
- Broad host-style all-target Clippy remains blocked by pre-existing vendored Rats wildcard-import and Animus lint-expectation findings; repository-owned strict checks are run separately through `scripts/check.rs` and Nix.

## Approach registry result

The classified read-only plus explicit-repair family validated. Automatic writable fallback was rejected for violating inspection authority; retry-only was falsified because recovery-required state persists without a writer; snapshot repair was rejected because an unlocked copy can race and is not the authoritative store.

## Non-claims

Successful `runs repair` proves only that redb opened the existing file and completed any redb-owned recovery. It does not prove Lattice semantic correctness, complete history, application-evidence validity, external anchoring, provider truth, remote exactly-once effects, arbitrary corruption recovery, or production readiness. Subsequent `runs show` independently validates the requested Lattice evidence.
