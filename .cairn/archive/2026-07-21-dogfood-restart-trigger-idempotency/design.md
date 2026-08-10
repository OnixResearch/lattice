## Context

`orchestration::shell::start_from_trigger` persists the scoped trigger key, immutable instance, and empty event stream atomically before executing a ready round. A duplicate consults the persisted admission map and returns the associated instance. Existing core, store, and shell tests exercise these facts in one process. The missing evidence is a real process boundary: the first redb owner must exit, a new process must reopen the file, and the duplicate must remain idempotent.

## Success Contract

**Goal:** Validate duplicate orchestration-trigger admission across an actual process restart.

**Completion evidence:**

- two separately invoked operating-system processes open the same fresh redb store in sequence;
- process one admits one trigger fact and records one completed run;
- process two submits the exact same definition revision, trigger identity, and idempotency key and returns the exact original instance identity;
- final durable state contains one trigger admission, one orchestration instance, one run, one settled round charge, no outstanding reservation, and a valid replay chain; and
- an invalid child phase exits unsuccessfully without creating the durable store.

**False completion:** Reopening two handles inside one process, mocking persistence, comparing only returned IDs, accepting a second run or budget charge, retaining private values, or claiming remote exactly-once effects does not satisfy the goal.

**Audit risks:** Test-harness recursion, stale redb locks, accidental parent-process reuse, unbounded diagnostic capture, hidden second effects, sequence-reservation overclaims, and evidence containing workflow output or environment secrets.

**Budget:** At most eight targeted source/spec reads, four approach families, one secondary audit, three implementation/audit rounds, focused tests plus repository/Cairn/Nix rails, and no live provider dispatch.

## Decisions

### Decision: Self-spawn the focused integration-test binary

**Choice:** The parent test creates a fresh directory and invokes its own compiled test binary twice with an exact test filter and explicit phase/path environment variables. Each child independently opens and drops `Store`; the parent opens the store only after both children exit and validates bounded durable facts.

**Rationale:** This crosses a real operating-system process and redb lock-owner boundary without adding a product-only debug command, a temporary example binary, external network dependency, or shell-script parsing. The child phase dispatcher is a thin imperative shell; deterministic durable assertions remain ordinary Rust assertions over loaded records.

### Decision: Keep the drill provider-free and content-omitting

**Choice:** Reuse the bounded local shell-node orchestration fixture. Persist only the normal redacted run history and write no auxiliary payload dump. The parent reports assertions through the test harness rather than retaining raw database exports.

**Rationale:** Trigger durability does not require a model provider. Avoiding provider transport makes the process boundary deterministic and keeps prompts, credentials, reasoning, and remote-effect uncertainty outside the evidence.

### Decision: Validate exact durable cardinality and settlement

**Choice:** The final parent assertion checks the original instance identity, one admission, one instance, one run, consumed round count, empty reservations, completed status, and replay validity.

**Rationale:** Matching only the returned ID can hide duplicate effects. Cardinality, budget, and replay checks directly test the accepted idempotency contract.

## Approach Registry

| Family | Mechanism | State | Evidence / blocker |
|---|---|---|---|
| self-spawned integration test | Two exact-filter child test processes reopen one redb file | active | Direct process and lock-owner boundary with repository-native assertions |
| external probe example | Add and invoke a dedicated example binary twice | rejected | Adds a durable debug surface after prior temporary probes were intentionally removed |
| same-process reopen | Drop and reopen two `Store` values in one test process | falsified | Tests persistence but not an actual process restart |
| product CLI trigger command | Add an operator command solely to drive the drill | rejected | Expands public authority and CLI scope without a product requirement |

## Risks / Trade-offs

- Test binaries vary by harness, so child invocation uses the current executable plus an exact test name and fails closed on unknown phases.
- The drill proves local redb persistence and exact Lattice admission reuse after a clean process exit. It does not prove behavior for a kill at every instruction boundary, provider cancellation, remote exactly-once execution, external event-ledger anchoring, or production readiness.
- Atomic ID counters may contain gaps after reservations; the contract is one durable admission/instance/run/budget charge, not contiguous sequence allocation.
