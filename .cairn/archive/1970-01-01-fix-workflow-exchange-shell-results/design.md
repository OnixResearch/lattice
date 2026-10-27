## Context

The shell executor currently treats every returned `ProcessOutput` as completed node data, even when its exit code is nonzero. That behavior is useful for ordinary data-flow graphs and is covered by `execute_failing_command`. It is not a successful command result for a workflow-exchange consumer. Kiln has reproduced a command exit of 23 with a retained `Completed` node and an incorrect successful terminal observation.

## Decisions

### Decision: Enforce required shell success in immutable exchange execution

**Choice:** Add an executor-internal required-shell-success policy, enabled by the existing immutable snapshot headless execution path used by the workflow-exchange worker. Classify a returned shell status before completing the node. A nonzero or missing successful exit status produces a failed node with a bounded numeric diagnostic. Existing runner failure propagation skips dependent work and persists the failure.

**Rationale:** This corrects execution at its source, rather than interpreting arbitrary output text in Kiln or relabeling an otherwise successful durable run after dependent effects have happened. The only current snapshot request caller is the workflow-exchange worker. No wire schema, route setting, consumer heuristic, provider type, or configuration switch is required.

### Decision: Preserve ordinary exit-as-data graphs

**Choice:** Leave the general executor policy unchanged outside immutable exchange snapshot execution. Keep the existing ordinary-shell regression test and exercise it alongside the new consumer-visible failure regression.

**Rationale:** A global change would break native workflows that branch on an explicit shell exit output. The exchange application instead owns a command-success contract for its mapped nodes.

### Decision: Preserve the published skipped-node receipt invariant

**Choice:** Retain no execution receipt for a skipped dependent. The existing workflow contract requires receipts for completed, failed, and cancelled nodes, and rejects receipts on skipped nodes.

**Rationale:** The paired regression exposed that the producer attached a receipt to every terminalized node. That made a failed command with a skipped dependent inadmissible to its own consumer contract. A skipped node did not execute; omitting its receipt preserves the current contract without changing the wire schema or importing a new contract cohort.

### Decision: Verify persisted results and dependency suppression

**Choice:** Pair a successful shell with a failing shell in native application coverage, and include a dependent marker effect for the failing graph. Assert terminal state, success flag, retained node status, and the absence of the dependent effect. Then run the actual Kiln HTTP/SQLite client, Lattice Unix service, and exact-source helper against isolated local source and GitHub API fixtures.

**Rationale:** In-memory status checks alone would miss false durable success or execution continuing after failure. The cross-repository smoke proves the consumer-visible CI mapping without claiming live GitHub authority.

## Risks / Trade-offs

- Immutable snapshot execution now has stricter shell semantics. Its sole current caller is the exchange worker; ordinary executor behavior stays unchanged.
- Shell failure diagnostics contain only the numeric status, not unredacted command output or credentials.
- Timeouts, cancellation, process errors, and unknown outcomes continue through their existing paths; this change does not add retries or reinterpret uncertainty as command failure.
- Operators must deploy a Lattice runtime with this producer-side correction for GitHub CI failure propagation. Updating only Kiln's contract import cannot repair an older running producer.

## Verification evidence

- Kiln retains `evidence/github-ci-local-runtime.ncl` and its imported raw JSON in
  the Kiln repository. The receipt binds the exercised executables and selected
  producer files by BLAKE3. Actual HTTP/SQLite intake, the native Unix service and
  exact-source jobs produced push success and PR command failure, suppressed
  duplicates across restart, and reconciled a lost status response without an
  additional POST. Source acquisition, SecretSpec and GitHub API inputs were local
  fixtures; no live GitHub write or publication authority is claimed.
- Native focused checks pass: ten workflow-exchange tests, the existing ordinary
  exit-as-data regression, nineteen transport tests, and the Nix transport check.
  The unlisted-peer fixture no longer asserts a socket-close/write race outcome;
  it still requires a failed exchange, one rejected peer and no handler calls.
- Repository-wide gate repairs are tracked in `repair-release-gate-evidence`.
  Reviewed complete warning observations conserve 845 accepted Octet facts with
  zero new or stale identities; current Cairn coverage links all 1,533 accepted
  requirements with zero missing or dangling markers. Kiln's scoped evidence
  records the latest full repository-script and Nix verdicts separately; focused
  checks and declared traceability do not establish release readiness.

