# Worker Inventory Specification

## ADDED Requirements

### Requirement: Worker inventory v2 is the only published inventory
r[lattice.worker_inventory.schema] `implementation-attempt inspect` MUST print exactly one `lattice.worker-inventory.v2` document with a `profile` (`lattice.worker-profile-facts.v2`), an `instance` (`lattice.worker-instance-observation.v2`, or `null` when no execution is retained) and `model_requests`. `RuntimeOptions::worker_profile_facts` MUST return `lattice.worker-profile-facts.v2`. These schema names supersede the v1 names stated by the `lattice.execution_binding.profile_facts` and `lattice.execution_binding.instances` requirements of `bind-worker-compute-allocations`; every other obligation of those requirements is unchanged. Lattice MUST NOT emit or decode a v1 inventory, profile-facts or instance-observation document. Listing grants no execution, credential or compute authority.

#### Scenario: Only v2 is emitted
r[lattice.worker_inventory.schema.scenario.cutover]
- GIVEN a retained implementation attempt and an empty state directory
- WHEN `inspect` projects each
- THEN both documents MUST carry `lattice.worker-inventory.v2` and the v2 profile and instance schemas, and no v1 schema name MUST appear in any produced document

### Requirement: Definition identity separate from the profile revision
r[lattice.worker_inventory.definition] Every profile MUST carry a `definition` with `identity`, `name` and `basis`. The basis MUST be `declared` when the worker configuration declares a definition name, and `owner_default` otherwise. The owner default name MUST be `omp-attempt-worker` for implementation attempts and the target label (`pi`, `clankers`, `animus`) for configured routes. The identity MUST be `blake3:` plus the hex digest of the `lattice-worker-definition-v1` content identity over the profile owner, the basis and the name, and MUST NOT depend on the profile revision. Every instance MUST carry the `definition_identity` of the profile it was admitted under.

#### Scenario: Definition identity survives a new revision
r[lattice.worker_inventory.definition.scenario.stable]
- GIVEN two implementation-attempt requests that declare the same definition name but differ in executable, tools or task families
- WHEN their profiles are projected
- THEN their profile revisions MUST differ and their definition identities MUST be equal

#### Scenario: Declared and default definitions never collide
r[lattice.worker_inventory.definition.scenario.basis]
- GIVEN a declared definition named `omp-attempt-worker` and an undeclared implementation-attempt profile
- WHEN both are projected
- THEN both names MUST be `omp-attempt-worker`, the bases MUST differ and the identities MUST differ

### Requirement: Bounded, redacted declared capability
r[lattice.worker_inventory.capability] A worker configuration MAY carry one `lattice.worker-capability.v1` declaration with a required `definition` name and optional `task_families` and `tools` sets. A set MUST be `{ kind = "declared", names }` with 1 to 32 distinct names or `{ kind = "unavailable" }`. A name MUST be 1 to 64 bytes, start with a lowercase ASCII letter or digit, and contain only lowercase ASCII letters, digits, `.`, `_`, `:` and `-`, so it can carry no path, whitespace, command line, prompt or URL. The Nickel contract, the strict Rust decoder and pre-effect request validation MUST refuse a malformed declaration. The profile MUST publish `task_families` and `tools` as facts: `declared` (or `qualified` only with bound qualification evidence) with the sorted names, `unavailable` with no value, or `unknown` with no value when the set or the whole declaration is absent. An absent declaration MUST NOT be published as an empty list. A present declaration MUST be part of the profile revision in its canonical sorted form; an absent declaration MUST leave the revision bytes of existing requests unchanged.

#### Scenario: Existing configurations report undeclared capability
r[lattice.worker_inventory.capability.scenario.undeclared]
- GIVEN a configuration or request without `worker_capability`
- WHEN it is decoded, validated and projected
- THEN it MUST be accepted with its previous profile revision, both capability facts MUST be `unknown` with no value, and the profile MUST carry a `capability_undeclared` wait reason for each

#### Scenario: Declared capability is canonical and bounded
r[lattice.worker_inventory.capability.scenario.declared]
- GIVEN a declaration whose names are listed out of order
- WHEN the profile is projected and the request identity is computed
- THEN the published names MUST be sorted, the same names in another order MUST give the same revision, and an `unavailable` set MUST publish `unavailable` with no value

#### Scenario: Malformed declarations are refused
r[lattice.worker_inventory.capability.scenario.malformed]
- GIVEN a declaration with a wrong schema, an invalid definition name, an empty or 33-name set, a duplicate name, a 65-byte name, an uppercase, path, whitespace or URL name, or an unknown field
- WHEN Nickel evaluates it, the strict decoder reads it, or a request carrying it is validated or inspected
- THEN it MUST be refused with a typed defect before any binding, intent or worker effect

#### Scenario: Inventory carries no material
r[lattice.worker_inventory.capability.scenario.redacted]
- GIVEN an attempt whose request carries program, sandbox and relay paths, a workspace root, a prompt, a model endpoint and a credential reference
- WHEN `inspect` prints its inventory
- THEN none of those values MUST appear in the output

### Requirement: Instance liveness never becomes idle or live without an observation
r[lattice.worker_inventory.liveness] Every instance MUST carry `liveness` with `runtime`, `state`, `unknown_cause`, `retained_generation` and `observed_at_epoch_ms`. `runtime` MUST be `single_shot` for implementation-attempt workers; Lattice owns no reusable runtime. `state` MUST be `not_started` for an admitted or rejected binding without a dispatch, `exited` only with an observed terminal result, and `unknown` otherwise. An unknown state MUST name `unobserved` while the admitted request window is open and `stale` once the observation time is past it. `retained_generation` MUST count the retained write-once lifecycle artifacts and the terminal result, so it never decreases for one attempt. Lattice MUST NOT publish `idle`, `live`, `running`, `healthy` or `available` for any instance.

#### Scenario: Missing or stale data stays unknown
r[lattice.worker_inventory.liveness.scenario.unknown]
- GIVEN a dispatched attempt without a terminal result, observed once inside and once after its admitted window
- WHEN the instance is projected
- THEN liveness MUST be `unknown` with cause `unobserved` and then `stale`, and never `idle` or `live`

#### Scenario: Observed lifecycle maps to single-shot liveness
r[lattice.worker_inventory.liveness.scenario.single_shot]
- GIVEN an admitted-only, a rejected, and a terminal attempt
- WHEN each instance is projected
- THEN liveness MUST be `not_started`, `not_started` and `exited`, with `runtime` `single_shot` and a generation that grows with the retained artifacts

### Requirement: Closed-vocabulary wait reasons that preserve hidden names
r[lattice.worker_inventory.wait_reasons] Profiles and instances MUST carry `wait_reasons`: at most 32 entries, sorted and deduplicated, each with a closed `code`, a closed `subject` and the `owner` that can resolve it. A subject MUST be a fact field from a closed set or a route by a name already published in the same document's model routes. Reasons MUST NOT carry free text and MUST NOT name anything inventory withholds: paths, endpoints, prompts, commands, credential references, grant references or backend handles. Profiles MUST report undeclared capability (`configuration`), missing qualification evidence (`animus#18`), the unpublished revision selection (`lattice#25`) and serving readiness causes. Non-terminal instances MUST report their dispatch, admission-window, cancellation, liveness, host and allocation unknowns and serving readiness causes. Terminal and rejected instances MUST carry no wait reasons.

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

### Requirement: Revision lifecycle slots are explicit owned unknowns
r[lattice.worker_inventory.revisions] Every profile MUST carry `revisions` with `desired`, `prepared`, `qualified`, `selected_for_new_sessions` and `pinned` slots. Until Lattice #25 implements that lifecycle, each slot MUST be `{ "support": "unknown", "owner": "lattice#25" }`. A consumer MUST be able to distinguish these slots from an empty value.

#### Scenario: Slots are not published
r[lattice.worker_inventory.revisions.scenario.unpublished]
- GIVEN any projected profile
- WHEN its revisions are read
- THEN every slot MUST be unknown with owner `lattice#25` and none MUST carry a revision

### Requirement: Deterministic captures for the consumer
r[lattice.worker_inventory.captures] The change MUST retain a reproducible capture procedure that writes v2 `inspect` outputs for the scenarios of Kiln's v1 capture (terminal failed, admitted not dispatched, dispatched outcome unknown, profile without instance, a second provider allocation, and a served route with a ready and an unavailable serving port) plus one declared-capability scenario. Captures are synthetic fixture outputs, not live observations.

#### Scenario: Captures are reproducible fixtures
r[lattice.worker_inventory.captures.scenario.fixture]
- GIVEN the capture tests and an output directory
- WHEN they run with `LATTICE_INVENTORY_CAPTURE_DIR` set
- THEN they MUST write one v2 document per scenario, and without the variable they MUST write nothing
