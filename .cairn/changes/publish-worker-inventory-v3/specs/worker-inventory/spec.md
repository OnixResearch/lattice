# Worker Inventory Specification

## MODIFIED Requirements

### Requirement: Worker inventory v3 is the only published inventory
r[lattice.worker_inventory.schema] `implementation-attempt inspect` MUST print exactly one `lattice.worker-inventory.v3` document with a `profile` (`lattice.worker-profile-facts.v3`), an `instance` (`lattice.worker-instance-observation.v3`, or `null` when no execution is retained) and `model_requests`. `RuntimeOptions::worker_profile_facts` MUST return `lattice.worker-profile-facts.v3`. These schema names supersede the v2 names of this requirement as stated by `publish-worker-inventory-v2`, and the v1 names stated by the `lattice.execution_binding.profile_facts` and `lattice.execution_binding.instances` requirements of `bind-worker-compute-allocations`; every other obligation of those requirements is unchanged. Lattice MUST NOT emit or decode a v1 or v2 inventory, profile-facts or instance-observation document. The profile MUST NOT carry a top-level `qualification_ref`. Listing grants no execution, credential or compute authority.

#### Scenario: Only v3 is emitted
r[lattice.worker_inventory.schema.scenario.cutover]
- GIVEN a retained implementation attempt and an empty state directory
- WHEN `inspect` projects each
- THEN both documents MUST carry `lattice.worker-inventory.v3` and the v3 profile and instance schemas, and no v1 or v2 schema name MUST appear in any produced document

### Requirement: Revision lifecycle slots are projected from the definition's lifecycle
r[lattice.worker_inventory.revisions] Every profile MUST carry `revisions` with `desired`, `prepared`, `qualified` and `selected_for_new_sessions` slots, and every instance MUST carry a `pinned` slot for its session, whose identifier is the attempt's envelope identity. Each slot MUST be exactly one of `{ "support": "unmanaged" }`, `{ "support": "unknown", "owner" }` or `{ "support": "recorded", "value" }`. Every slot MUST be `unmanaged` when `inspect` has no `--revision-store`, when the store holds no lifecycle for the profile's definition, and for configured Pi, Clankers and Animus routes; `unmanaged` MUST be distinct from `unknown`. For a managed definition each profile slot MUST be `recorded` from `lattice_core::agent_revision::projection::slots`, or `unknown` with owner `lattice#25` for `desired` and `selected_for_new_sessions`, `kiln#49` for `prepared` and `animus#18` for `qualified` when the lifecycle holds no such fact. A recorded `qualified` slot MUST carry each qualification's revision, task family, `qualification_ref`, evidence class, `fixture` or `live` provenance and whether it is revoked. The `pinned` slot MUST follow the admission mode the attempt recorded in `intent.json`, never whether the definition is managed now: `unmanaged` for an attempt admitted unmanaged or whose intent has no admission field, and otherwise the pin with its revision, `qualification_ref`, rollout generation, pin identity, `live`, `closed` or `released` status and whether it is blocked. A named store that does not exist, a lifecycle of another definition, and a managed attempt inspected without its store MUST be errors. Inspection MUST NOT write the store.

#### Scenario: Without a lifecycle every slot is unmanaged
r[lattice.worker_inventory.revisions.scenario.unmanaged]
- GIVEN a profile projected without a revision store, and one whose store holds no lifecycle for its definition
- WHEN its revisions and its instance's `pinned` slot are read
- THEN every slot MUST be `unmanaged`, none MUST be `unknown` or carry a revision, and the profile MUST NOT carry `revision_not_admitted`

#### Scenario: A managed definition publishes recorded facts or owned unknowns
r[lattice.worker_inventory.revisions.scenario.managed]
- GIVEN a definition desired, then prepared through a fixture preparation port, then promoted with an Animus #18 fixture attachment, and then revoked
- WHEN its inventory is projected with the store after each step
- THEN after desire `desired` MUST be recorded while `prepared`, `qualified` and `selected_for_new_sessions` MUST be `unknown` with owners `kiln#49`, `animus#18` and `lattice#25`; each later step MUST record its fact and the qualification's provenance MUST be `fixture`; and after revocation of the revision `capability_qualifications` MUST be empty and the profile MUST carry `revision_not_admitted`

#### Scenario: The pin follows the recorded admission
r[lattice.worker_inventory.revisions.scenario.pinned]
- GIVEN a live admitted session, a terminal managed attempt whose closed session the lifecycle forgot, a managed attempt blocked before spawn, and an attempt admitted unmanaged before its definition became managed
- WHEN each instance is projected with the store
- THEN the live session MUST be recorded `live`, the terminal attempt MUST be recorded `released` with the pin its intent recorded, the blocked attempt MUST be recorded with `is_blocked` true in lifecycle `revision_blocked_before_dispatch`, a later revocation of the pinned revision MUST set `is_blocked` on the released pin, and the unmanaged attempt MUST stay `unmanaged`

### Requirement: Instance liveness never becomes idle or live without an observation
r[lattice.worker_inventory.liveness] Every instance MUST carry `liveness` with `runtime`, `state`, `unknown_cause`, `retained_generation` and `observed_at_epoch_ms`. `runtime` MUST be `single_shot` for implementation-attempt workers; Lattice owns no reusable runtime. `state` MUST be `not_started` for an admitted, rejected or revision-blocked binding without a dispatch (`admitted_not_dispatched`, `binding_rejected_before_dispatch` or `revision_blocked_before_dispatch`), `exited` only with an observed terminal result, and `unknown` otherwise. A `revision_blocked_before_dispatch` instance MUST have an intent and a retained revision block, and no binding refusal or terminal result. An unknown state MUST name `unobserved` while the admitted request window is open and `stale` once the observation time is past it. `retained_generation` MUST count the retained write-once lifecycle artifacts and the terminal result, so it never decreases for one attempt. Lattice MUST NOT publish `idle`, `live`, `running`, `healthy` or any other availability claim as liveness.

#### Scenario: Missing or stale data stays unknown
r[lattice.worker_inventory.liveness.scenario.unknown]
- GIVEN a dispatched attempt without a terminal result, observed once inside and once after its admitted window
- WHEN the instance is projected
- THEN liveness MUST be `unknown` with cause `unobserved` and then `stale`, and never `idle` or `live`

#### Scenario: Observed lifecycle maps to single-shot liveness
r[lattice.worker_inventory.liveness.scenario.single_shot]
- GIVEN an admitted-only, a rejected, a revision-blocked and a terminal attempt
- WHEN each instance is projected
- THEN liveness MUST be `not_started`, `not_started`, `not_started` and `exited`, with `runtime` `single_shot` and a generation that grows with the retained artifacts

### Requirement: Closed-vocabulary wait reasons that preserve hidden names
r[lattice.worker_inventory.wait_reasons] Profiles and instances MUST carry `wait_reasons`: at most 32 entries, sorted and deduplicated, each with a closed `code`, a closed `subject` and the `owner` that can resolve it. A subject MUST be a fact field from a closed set or a route by a name already published in the same document's model routes. Reasons MUST NOT carry free text and MUST NOT name anything inventory withholds: paths, endpoints, prompts, commands, credential references, grant references or backend handles. Profiles MUST report undeclared capability (`configuration`), a `task_families` fact that is not `qualified` as `qualification_unbound` with subject `capability_qualifications` (`animus#18`), a lifecycle-managed definition that would refuse a new session of the profile's revision as `revision_not_admitted` with subject `revisions` (`lattice#25`), and serving readiness causes. An unmanaged profile MUST NOT carry `revision_not_admitted`, and no document MUST carry `revision_selection_unpublished`. Non-terminal instances MUST report their dispatch, admission-window, cancellation, liveness, host, allocation and serving causes; terminal, rejected and revision-blocked instances MUST carry none.

#### Scenario: Serving and lifecycle causes are explicit
r[lattice.worker_inventory.wait_reasons.scenario.causes]
- GIVEN a dispatched instance with no serving-owner port, and a profile whose served route is draining or not current
- WHEN they are projected
- THEN the instance MUST report `dispatch_outcome_unknown`, `liveness_unobserved` and `serving_port_absent`, and the profile MUST report the route by its published name with `serving_snapshot_not_current` or `serving_not_ready`

#### Scenario: Reasons are deterministic and redacted
r[lattice.worker_inventory.wait_reasons.scenario.disclosure]
- GIVEN the same retained state projected twice, and inventory whose request carries withheld material
- WHEN wait reasons are produced
- THEN both projections MUST give byte-identical reasons in sorted order, every route subject MUST be a published route name, and no withheld value MUST appear

### Requirement: Deterministic captures for the consumer
r[lattice.worker_inventory.captures] The change MUST retain a reproducible capture procedure that writes v3 `inspect` outputs for the scenarios of the v2 captures (terminal failed, admitted not dispatched, dispatched outcome unknown, profile without instance, a second provider allocation, a declared capability, and a served route with a ready and an unavailable serving port), unmanaged, plus a managed lifecycle: desired, prepared through a fixture preparation port, promoted with an Animus #18 fixture attachment, a live admitted session, a terminal attempt whose pin was released, the same after revocation, and the profile after an attempt refused after revocation. Captures are synthetic fixture outputs, not live observations.

#### Scenario: Captures are reproducible fixtures
r[lattice.worker_inventory.captures.scenario.fixture]
- GIVEN the capture tests and an output directory
- WHEN they run with `LATTICE_INVENTORY_CAPTURE_DIR` set
- THEN they MUST write one v3 document per scenario, and without the variable they MUST write nothing

## ADDED Requirements

### Requirement: Capability qualification comes from the revision lifecycle
r[lattice.worker_inventory.qualification] Every profile MUST carry `capability_qualifications`: sorted by task family, one entry per qualification the definition's lifecycle records for the profile's own revision whose attachment is not revoked, each with `task_family`, the Animus #18 attachment identity as `qualification_ref` verbatim, `evidence_class` and the producer's `fixture` or `live` provenance. It MUST be empty without a lifecycle and when the profile's revision is revoked. `task_families` MUST be `qualified`, with the sorted declared names as value, exactly when the profile declares a `task_families` set and every declared family has an entry; otherwise it MUST keep its declared, unavailable or unknown support. Provenance MUST be published and MUST NOT decide qualification. No other profile fact MUST be `qualified`.

#### Scenario: Every declared family qualified makes the set qualified
r[lattice.worker_inventory.qualification.scenario.qualified]
- GIVEN a profile that declares one task family, and a lifecycle that promoted its revision with a fixture attachment for that family
- WHEN the profile is projected with the store
- THEN `task_families` MUST be `qualified`, `capability_qualifications` MUST list that family with the attachment identity, evidence class and `fixture` provenance, and the profile MUST NOT carry `qualification_unbound`

#### Scenario: Missing, foreign or revoked qualification stays unbound
r[lattice.worker_inventory.qualification.scenario.unbound]
- GIVEN a profile without a store, one with a declared family that has no qualification, one whose lifecycle qualifies another revision, one whose attachment or revision is revoked, and a lifecycle of another definition
- WHEN each profile is projected
- THEN `task_families` MUST NOT be `qualified`, `capability_qualifications` MUST omit the unqualified, foreign or revoked entries, the profile MUST carry `qualification_unbound` naming `capability_qualifications`, and the foreign lifecycle MUST be refused with a typed defect
