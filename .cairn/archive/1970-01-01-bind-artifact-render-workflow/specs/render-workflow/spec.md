# Artifact-bound render consumer specification

## ADDED Requirements

### Requirement: Consume the exact public producer contract

r[lattice.render_binding.producer] Lattice MUST consume the immutable public
Aspen render/batch contract closure through a repository-owned import and MUST
use canonical `Manifest`, `prepare`, content identity and `verify_final`. It MUST
NOT introduce a renderer, tile scheduler, interchangeable component ABI or
sibling-working-tree production dependency.

#### Scenario: Public immutable source is acquired

- GIVEN the exact reviewed producer revision and anonymous archive URL
- WHEN the repository importer acquires and selects source
- THEN archive/file BLAKE3, selected paths, wrapper ownership and licenses MUST be retained.

#### Scenario: Archive or imported bytes differ

- GIVEN archive identity or an existing selected imported file differs
- WHEN import runs
- THEN it MUST fail rather than overwrite local/imported repairs or follow a branch.

### Requirement: Exact graph and execution admission precede effects

r[lattice.render_binding.admission] The opt-in closed operation MUST bind canonical
request identity, logical execution/attempt/fence/budget and actual host-issued
run/node/revision/invocation provenance before possible submission. Existing graph
and effect authority MUST remain authoritative; read-only automatic grants MUST
NOT authorize rendering mutations. The job including assembly/retries MUST fit
canonical task/input/object bounds and supplied reservation ceilings.

#### Scenario: Current exact render graph is dispatched

- GIVEN an admitted immutable request, actual host context and current owner
- WHEN the exact configured graph authority dispatches its bounded process
- THEN the journal MAY bind that compute owner and submit through the existing producer CLI.

#### Scenario: Request, attempt or actual compute run is crossed

- GIVEN changed request bytes, stale fence/budget or another actual Advance run
- WHEN dispatch preparation or acceptance runs
- THEN it MUST reject without a new batch or substituted identity.

#### Scenario: A standalone caller supplies only a run number

- GIVEN no retained engine-owned dispatch provenance
- WHEN the runtime entrypoint is invoked
- THEN dispatch MUST be unsupported/denied rather than fabricate engine-issued binding.

### Requirement: Accepted content, not terminal shape, determines completion

r[lattice.render_binding.acceptance] Completion MUST require exact producer
run/spec/cohort and complete task inventory, canonical accepted task
assignment/request/result fences, content-manifest identity, exact lengths,
canonical tile coverage/coordinates/encoding/digests/pixels and strict final
verification. The operation classifier MUST independently verify retained
completion and owner currentness before producing completed workflow output.

#### Scenario: Valid completion observations are reordered or duplicated

- GIVEN all exact authoritative accepted artifacts for the same qualified cohort
- WHEN snapshots are restored or observed repeatedly
- THEN the accepted manifest/image MUST remain identical and duplicates MUST NOT inflate progress.

#### Scenario: Malformed, crossed or partial results report success

- GIVEN missing/unexpected/overlapping tiles, wrong sizes/digests, rehashed wrong pixels, crossed attempts or exit-zero partial output
- WHEN final acceptance runs
- THEN Completed MUST be impossible.

### Requirement: Uncertainty never resubmits possible computation

r[lattice.render_binding.recovery] Durable intent MUST precede possible Submit.
Any possible-dispatch record MUST permanently prevent automatic resubmission.
Recovery MUST observe the producer's exact deterministic run and accepted content;
unavailable observations/content MUST remain uncertain without new keys, budgets
or attempts. Explicit new observers MUST retain the original compute host context.

#### Scenario: Submit ACK or observer/controller/worker observation is lost

- GIVEN a durable possible-dispatch record and its original exact identities
- WHEN a caller reconnects
- THEN it MUST use Observe and MUST NOT call Submit again.

#### Scenario: Durable intent exists but no producer run can be recovered

- GIVEN the caller cannot establish whether submission crossed the boundary
- WHEN the producer does not supply an accepted run/artifact
- THEN Lattice MUST retain Unknown rather than retry to force progress.

### Requirement: Cancellation, supersession and accounting remain truthful

r[lattice.render_binding.truthfulness] Cancellation intent MUST be retained before
requesting cancellation and MUST block late completion. Only producer-observed
terminal cancellation MAY establish stop. Stale/superseded owners MUST NOT accept
late results. Recovery MUST preserve original budget/deadline and distinguish
reservation, measured adapter time and unavailable remote usage.

#### Scenario: Cancellation races with successful remote completion

- GIVEN a retained cancellation request or superseded owner
- WHEN a late valid image is observed
- THEN it MUST NOT complete the render, and uncertain stop MUST NOT be reported as stopped.

#### Scenario: Producer supplies no metered usage

- GIVEN only reserved task resources and client timings
- WHEN a receipt is emitted
- THEN remote usage MUST be unavailable and MUST NOT be fabricated from reservations or elapsed time.

### Requirement: Retain actual consumer qualification, not producer claims

r[lattice.render_binding.qualification] The opt-in Nix fixture MUST retain exact
source/dependency/runtime/component/graph/operation identities, actual Lattice
workflow and accepted-artifact receipts, comparison method and measured/unavailable
timings. Requested fault controls MUST actually cross their declared boundaries
or fail. Same-backend comparison MUST be exercised independently of producer
native tests; cross-backend equality MUST NOT be assumed.

#### Scenario: Qualified local fixture executes

- GIVEN a built exact Lattice binary and explicitly selected fixture authority
- WHEN the real admitted graph and same-host producer processes execute
- THEN completion/repetition and requested lost-ACK/interruption/cancellation controls MUST retain actual outcomes.

#### Scenario: Only authored fixtures or producer tests exist

- GIVEN no retained successful Lattice fixture result
- WHEN readiness is assessed
- THEN the existence of code, native contract tests or producer campaigns MUST NOT satisfy consumer execution qualification.
