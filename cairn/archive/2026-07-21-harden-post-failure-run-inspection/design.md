## Context

`run::shell::inspect_path` opens redb through `ReadOnlyDatabase`, preserving the content-omitting command's non-mutation contract. redb 3.1.1 returns `DatabaseAlreadyOpen` while another owner holds the file and `RepairAborted` when a prior unclean shutdown requires allocator-state repair that read-only mode cannot perform. Its `Database::drop` deliberately skips allocator-state finalization while the thread is panicking, which explains the live drill: inspection failed after panic unwind, then succeeded after a normal writable `Store` reopen repaired the file. Lattice currently erases these distinct states behind one generic error.

## Success Contract

**Goal:** Make post-failure run inspection diagnosable and recoverable without allowing `runs show` to mutate durable state.

**Completion evidence:** clean-exit inspection succeeds; active ownership and recovery-required stores produce distinct fixed diagnostics; hashing proves failed `runs show` does not change a recovery-required file; explicit `runs repair` refuses missing/owned stores, repairs an existing unclean store, creates no application records, and enables subsequent safe inspection; positive and negative process fixtures pass.

**False completion:** exposing raw redb/I/O/path text, automatically repairing from `runs show`, retrying a permanently recovery-required read-only open, copying and repairing an unlocked snapshot, creating a missing database, or proving only same-process behavior does not satisfy the goal.

**Audit risks:** TOCTOU between diagnosis and repair, accidental store creation, active-writer interference, repair presented as semantic validation, secret-bearing backend errors, test-child recursion, and unsupported corruption recovery claims.

**Budget:** At most ten targeted source/spec reads, four mechanism families, one secondary audit, three implementation/audit rounds, focused tests plus repository/Cairn/Nix rails, and no provider dispatch.

## Decisions

### Decision: Typed open-failure classification

**Choice:** Map redb `DatabaseAlreadyOpen` to `active_owner`, `RepairAborted` to `recovery_required`, and every other open failure to `unavailable`. Render fixed operator text and retain no backend error string in CLI output.

**Rationale:** Variant matching is deterministic and authority-backed by redb's public error contract. Fixed messages distinguish actionable states without leaking paths, OS errors, or database content.

### Decision: Inspection remains read-only

**Choice:** `runs show` continues to use `ReadOnlyDatabase` exclusively. It never retries through writable open and never repairs either the original or a copy.

**Rationale:** Automatic write-mode fallback would violate the command's accepted contract. Snapshot repair can race with a newly started writer and would inspect an authority-ambiguous copy.

### Decision: Recovery is explicit and existing-store-only

**Choice:** Add `runs repair`, which calls `redb::Database::open` rather than create, closes the recovered handle, and prints a fixed success receipt. It refuses active ownership and missing/unavailable files and does not open application tables.

**Rationale:** redb owns its storage repair algorithm. Separating mutation behind an explicit command makes operator intent visible and prevents missing-store creation or hidden Lattice-record changes.

### Decision: Process fixtures exercise redb lifecycle states

**Choice:** A self-spawned integration test child creates one real run record and exits cleanly, panics, aborts, or holds the store. Parent CLI invocations classify each state, compare BLAKE3 file identities around failed inspection, invoke explicit repair where applicable, and verify post-repair inspection.

**Rationale:** The defect is process-lifecycle-specific; mocks and same-process handles cannot establish it.

## Approach Registry

| Family | Mechanism | State | Evidence / blocker |
|---|---|---|---|
| classified read-only inspection plus explicit repair | Match redb variants; mutate only through `runs repair` | active | Preserves command authority and supplies bounded recovery |
| automatic writable fallback | Let `runs show` repair original store | rejected | Violates the accepted read-only contract |
| bounded retry only | Retry read-only open after delay | falsified | `RepairAborted` persists until write-mode recovery |
| repair a temporary snapshot | Copy then repair and inspect | rejected | Races with writers and reports from a non-authoritative copy |

## Risks / Trade-offs

- `runs repair` may change redb storage metadata because that is its explicit purpose; it does not prove application-level correctness or preserve byte identity.
- `active_owner` can become stale immediately after diagnosis. Operators retry after the owner exits; recovery remains a separate explicit command.
- `unavailable` intentionally combines unsupported format, permissions, corruption outside redb's repair path, and other storage failures to avoid exposing raw host details.
- Successful redb recovery proves only that redb reopened the existing file. Subsequent `runs show` still validates Lattice evidence independently.
