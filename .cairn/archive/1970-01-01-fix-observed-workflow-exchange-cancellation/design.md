## Context

The bounded local process runner returns `ProcessError::Cancelled` only for its observed cancellation disposition. Credential dispatch currently redacts every process error to AdapterRejected, the executor keeps ordinary Failed node details, and workflow-exchange completion always says not cancelled. Existing pure completion already supports durable Cancelled. Kiln's failed-before receipt is retained in its operate-single-tenant-web-service native-api evidence and must not be rerun merely to confirm the baseline.

## Decisions

### Decision: Preserve typed owned cancellation as a separate runtime fact

**Choice:** Record cancellation only at the typed process-result boundary before redaction, propagate that observation through executor and snapshot outcome, and supply it to the existing pure CompletionFacts decision. Do not inspect error text or use the atomic cancellation request as outcome evidence.

**Rationale:** Credential redaction must stay intact while cancellation remains distinguishable from command failure, spawn errors and timeouts. An observation is not a new wire field or configuration switch.

### Decision: Preserve completion winners

**Choice:** A completed successful or failed process without an observed cancellation retains its ordinary result even when durable cancellation intent exists. Active observed cancellation produces durable Cancelled through the existing core and blocks pending dependent execution through existing failure/cancellation propagation.

**Rationale:** Accepting intent cannot retroactively stop an already completed effect. The local process runner owns the termination race; the application must not invent another race policy.

### Decision: Exercise the real producer application

**Choice:** Cover a started child, exact admitted cancel, observed child stop, no dependent marker, durable Cancelled, ordinary success/failure, and deterministic completion-before-cancel classification using actual process execution and pure completion facts. Keep the strict Kiln assertion unchanged. Parent owns integrated execution of all checks and immutable producer qualification.

**Rationale:** Merely asserting cancellation intent misses both producer misclassification and effects continuing after cancellation.

### Decision: Restore the application's explicit dependency closure

**Choice:** Complete the existing Tile and component-runtime manifest/cohort integration using exact source revisions and Cargo-owned lock updates. Preserve those integrations rather than removing modules, disabling behavior, adding sibling-path overrides or inventing unavailable runtimes. Keep runtime/verifier identities consistent with the compiled dependencies and qualify the resulting application.

**Rationale:** The retained 55-error build proves that application integration, not the cancellation regression itself, blocks executable qualification. Kiln is the immediate consumer; Lattice maintains the dependency and cancellation contracts.

## Risks / Trade-offs

- Local owned-process cancellation does not prove cancellation of remote effects or escaped processes; retain the bounded runner's existing guarantees.
- Run-detail Failed remains the adapter's redacted process-error detail; canonical exchange cancellation comes from a separate typed runtime observation, not error strings.
- Suspension, persistence errors and lost workers retain existing unknown/recovery behavior; no retries or relabeling are introduced.
- Builds require the pinned Rust/Nickel/native closure and exact SSH-backed sources or caches described in README.
