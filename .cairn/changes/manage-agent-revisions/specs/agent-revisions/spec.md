# Agent Revisions Specification

## ADDED Requirements

### Requirement: A pure lifecycle core with closed refusals
r[lattice.agent_revisions.core] `lattice_core::agent_revision` MUST decide every lifecycle transition as a pure function of the current lifecycle and one command, returning a new lifecycle with an outcome or a refusal. The core MUST NOT read files, the environment, a clock, a process or the network. Every refusal MUST come from the closed `Refusal` vocabulary and MUST leave the caller's lifecycle unchanged. Collections MUST be bounded: 64 revisions, 1024 live sessions, a sliding window of the 256 most recent receipts, 256 qualification scopes, 1024 revocations per kind of targets the lifecycle does not reference, and 128 artifacts or holds per revision. A bound MUST count only what is still referenced, so no bound is a lifetime limit: when the revisions or qualification scopes are full, the core MUST first forget every revision that is unprepared, failed or superseded, occupies no artifact, has no hold and is not desired, selected, last usable or pinned, together with its qualifications and selection history; it MUST forget a closed session none of whose checkpoints is still held; and it MUST forget the oldest receipt when the window is full. Revocations MUST never be forgotten.

#### Scenario: Every refusal is typed
r[lattice.agent_revisions.core.scenario.refusals]
- GIVEN a lifecycle and a command that is malformed, names another definition or an unknown revision, exhausts a generation or exceeds a bound
- WHEN the core applies it
- THEN it MUST return the matching refusal and the lifecycle MUST stay unchanged

#### Scenario: Bounds free up instead of exhausting for good
r[lattice.agent_revisions.core.scenario.capacity]
- GIVEN sessions admitted and closed, revisions desired and promotions and rollbacks applied past each bound's size
- WHEN the lifecycle keeps operating
- THEN admission, desire, promotion and rollback MUST keep succeeding, a selected, pinned or revoked revision MUST NOT be forgotten, a forgotten request MUST be refused as `generation_conflict` instead of being applied again, and admission past 1024 live sessions or a desire past 64 revisions that are all still referenced MUST be refused as `capacity_exhausted`

### Requirement: Lifecycle facts are kept separately and revisions coexist
r[lattice.agent_revisions.facts] Per definition the lifecycle MUST keep separately the desired revision with its change generation, the preparation of each revision bound to the desired generation it served, qualification admitted per revision and task-family scope, the last usable revision, the revision selected for new sessions with its rollout generation, and the revision pinned to each session. Several revisions MUST coexist, and no fact MUST be derived from another.

#### Scenario: Old and new revisions coexist
r[lattice.agent_revisions.facts.scenario.coexist]
- GIVEN session A pinned to R1, a failed R2 and a promoted R3
- WHEN session C is admitted
- THEN C MUST pin R3, A MUST stay pinned to R1, and the lifecycle MUST hold R1, R2 and R3 with their own preparation states

### Requirement: Unframed section 1 components frame only when declared
r[lattice.agent_revisions.framing] `lattice.agent-revision-components.v1` MAY declare a memory policy, checkpoint semantics, a tool executor protocol with tool names, per-route model requirement references and producer/consumer compatibility. `framing::digest` MUST return `None` for an absent declaration and a `lattice-agent-revision-components-v1` digest, spelled `blake3:<hex>`, for a valid one. Names MUST follow the capability name grammar, references MUST be canonical, and lists MUST hold 1 to 32 distinct entries.

#### Scenario: Undeclared components keep existing revisions
r[lattice.agent_revisions.framing.scenario.undeclared]
- GIVEN no components declaration
- WHEN the digest is computed
- THEN it MUST be `None`, so a revision that frames the digest only when present keeps its existing bytes

#### Scenario: Declared components are canonical and change identity
r[lattice.agent_revisions.framing.scenario.declared]
- GIVEN a declaration, the same declaration with tool names reordered, and declarations that change the memory policy, the tools or a model requirement, or drop a component
- WHEN their digests are computed
- THEN reordering MUST NOT change the digest, every other change MUST change it, and a wrong schema, an invalid name, a missing reference, an empty list or a duplicate MUST be refused with a typed defect

### Requirement: An inert, material-free bundle per revision
r[lattice.agent_revisions.bundle] `agent-revision resolve` MUST resolve an exported implementation-attempt request into a `lattice.agent-bundle.v1` bundle without admitting, preparing or writing anything. The revision MUST be `Request::worker_profile_identity()` used verbatim, and the definition MUST be the inventory's `lattice-worker-definition-v1` identity. The bundle MUST carry identities and declared names only, never paths, prompts, endpoints, credential values or commands, and its identity MUST be the `lattice-agent-bundle-v1` digest of its canonical bytes.

#### Scenario: Resolution is opaque and redacted
r[lattice.agent_revisions.bundle.scenario.inert]
- GIVEN requests that differ only in prompt, declared tools, executable pin or broker model
- WHEN each is resolved
- THEN each revision MUST equal its `worker_profile_identity`, the revisions MUST differ, the definition MUST stay the same, and no path, endpoint, credential key or prompt MUST appear in the bundle

#### Scenario: Hosted routes keep unknown weight identity
r[lattice.agent_revisions.bundle.scenario.hosted]
- GIVEN a brokered hosted model route
- WHEN its bundle is resolved and a bundle claiming a weight identity is decoded
- THEN the route MUST carry the provider-exposed model with `weight_identity` and `model_version` `unknown`, and the claiming bundle MUST be refused

### Requirement: Exact lifecycle identities
r[lattice.agent_revisions.identities] Preparation, promotion policy, promotion and session pin identities MUST be `blake3:` digests under the domains `lattice-agent-revision-preparation-v1`, `lattice-agent-revision-promotion-policy-v1`, `lattice-agent-revision-promotion-v1` and `lattice-agent-revision-session-pin-v1`, framed by `content_identity_core`. A preparation identity MUST bind definition, revision and desired generation.

#### Scenario: Fixture identities are recorded
r[lattice.agent_revisions.identities.scenario.recorded]
- GIVEN the fixture R1, R2 and R3 lifecycle
- WHEN its identities are computed twice
- THEN they MUST be equal, canonical and distinct per subject, and the change evidence MUST record them with `live_qualification: false`

### Requirement: Preparation is bound to its desired generation
r[lattice.agent_revisions.preparation] Desiring a different revision MUST increment the change generation. Preparation MUST start only for the current desired revision that is not revoked, and its ticket MUST bind the desired generation. A pending or unknown attempt for the current generation MUST be reissued with the same ticket, even when its revision is revoked, a prepared revision MUST NOT be prepared again, and an attempt for an older generation MUST be reconciled before a new one starts. `prepare::reissue` MUST return the ticket of any pending or unknown attempt at the generation it is bound to, whether or not its revision is still desired, and `agent-revision prepare` MUST fall back to it when preparation cannot start, so such an attempt settles as superseded, failed or unknown. A preparation result MUST settle only an attempt with the same generation, the same result delivered again MUST be unchanged (a `prepared` result matches an attempt recorded prepared or superseded), a different result MUST be refused, and reported artifacts MUST stay recorded until an observed cleanup releases them.

#### Scenario: A failed preparation keeps the last usable revision
r[lattice.agent_revisions.preparation.scenario.failed]
- GIVEN R1 selected with session A pinned to it
- WHEN R2 is desired and its preparation fails
- THEN R1 MUST stay last usable and selected, A MUST keep effects under R1, a new session B MUST pin R1, and R2 MUST be refused for promotion and preparation

#### Scenario: A late result cannot replace newer state
r[lattice.agent_revisions.preparation.scenario.late]
- GIVEN R2's preparation pending at generation 2, then R3 desired, prepared and promoted
- WHEN R2's preparation result arrives
- THEN R2 MUST become `superseded`, the desired revision, selection, rollout generation and last usable revision MUST be unchanged, R2 MUST NOT be promotable, and its artifacts MUST be planned for cleanup

#### Scenario: An abandoned older attempt is reissued and settled
r[lattice.agent_revisions.preparation.scenario.reissue]
- GIVEN R1's preparation unknown at generation 1, then R2 desired, then R1 desired again at generation 3
- WHEN R1's preparation is started, reissued and its result recorded
- THEN starting MUST be refused as `preparation_in_flight`, reissue MUST return the generation 1 ticket, the result MUST settle it as `superseded`, and desiring R1 again MUST start a new attempt that prepares it

#### Scenario: A revoked revision never starts preparation
r[lattice.agent_revisions.preparation.scenario.revoked]
- GIVEN a desired revision that is revoked, once unprepared and once with a pending attempt
- WHEN preparation is started
- THEN the unprepared revision MUST be refused as `revision_revoked`, and the pending attempt MUST be reissued and its result recorded without making the revision last usable

#### Scenario: A redelivered result is recorded once
r[lattice.agent_revisions.preparation.scenario.redelivered]
- GIVEN R2 recorded `prepared` at generation 2, then R3 desired
- WHEN the same `prepared` result for generation 2 is delivered again, and then a different result
- THEN the first MUST be unchanged and the second MUST be refused as `result_conflict`

### Requirement: Explicit, idempotent promotion
r[lattice.agent_revisions.promotion] Promotion MUST name a request identifier, the exact prepared revision, a `qualification_ref`, a promotion policy (task family and required evidence class) and the expected rollout generation. It MUST be refused unless the revision is prepared and not revoked, the reference is not revoked and not bound to another revision or task family, the revision is not already selected, the qualification port's current answer for the exact definition, revision, task family and class is `qualified` with a canonical attachment of the required class that names the same reference, and the expected generation is current. A qualification answer MUST NOT be promotion authority by itself. Applying MUST increment the rollout generation and record the selection, the qualification and a receipt. A receipt MUST answer only a request with the same identifier, kind, revision, reference, policy and expected rollout generation.

#### Scenario: Duplicate requests and lost acknowledgements reconcile
r[lattice.agent_revisions.promotion.scenario.idempotent]
- GIVEN an applied promotion, later superseded by a rollback
- WHEN the same request identifier is applied again, with any answer, or reused for another command or another expected rollout generation
- THEN the same request MUST return its receipt without selecting again, a reused identifier MUST be refused as `request_conflict`, and repeating preparation MUST NOT prepare again

#### Scenario: Concurrent promotions select once
r[lattice.agent_revisions.promotion.scenario.concurrent]
- GIVEN two prepared, qualified revisions and two promotions expecting the same rollout generation
- WHEN both are applied in turn
- THEN exactly one MUST be selected and the other MUST be refused as `generation_conflict`

#### Scenario: Mismatched qualification is refused
r[lattice.agent_revisions.promotion.scenario.mismatch]
- GIVEN a reference admitted for R1, and a changed revision R6
- WHEN R6 is promoted with R1's reference, with a `not_qualified` answer, with an answer naming another reference, of a weaker class or with a malformed attachment
- THEN each MUST be refused as `qualification_mismatch`, `not_qualified` with the producer's reason, `qualification_mismatch`, `insufficient_class` or `not_qualified` with reason `malformed`

#### Scenario: The standalone CLI refuses promotion
r[lattice.agent_revisions.promotion.scenario.unavailable]
- GIVEN the standalone CLI with its unavailable qualification port
- WHEN a prepared revision is promoted
- THEN the command MUST fail with refusal `qualification_unavailable` and owner `animus#18`, and nothing MUST be selected

### Requirement: Rollback selects an earlier revision for future sessions
r[lattice.agent_revisions.rollback] Rollback MUST obey every promotion check, and MUST additionally name a revision that was selected before, is not currently selected and still has the same qualification recorded for the policy's task family. It MUST change only the selection for new sessions and MUST NOT rewrite any pin, receipt or history.

#### Scenario: Rollback leaves past sessions alone
r[lattice.agent_revisions.rollback.scenario.history]
- GIVEN R1 selected before, R3 selected now and sessions pinned to each
- WHEN rollback to R1 is applied, and rollback to a failed, never-selected, differently qualified or current revision is attempted
- THEN R1 MUST be selected with kind `rollback` and the next rollout generation, every pin MUST be unchanged, and the others MUST be refused

### Requirement: Session admission pins the selection
r[lattice.agent_revisions.sessions] A session MUST be admitted only for the revision selected for new sessions, and admission MUST pin that revision, its qualification reference and the rollout generation. Re-admitting a session under its pinned revision MUST return the existing pin only while the session is live and its revision and qualification are not revoked; a closed session MUST be refused as `session_closed`, and under another revision it MUST be refused as `session_conflict`. Closing a session MUST hold its checkpoints on the pinned revision and keep its pin only while one of them is held; a session closed without a held checkpoint, or whose last held checkpoint is released, MUST be forgotten.

#### Scenario: Pins survive new selections
r[lattice.agent_revisions.sessions.scenario.pinned]
- GIVEN A pinned to R1
- WHEN R3 is promoted and C is admitted
- THEN A MUST keep R1 and its effects, C MUST pin R3, and admitting a new session under R1 MUST be refused as `revision_not_selected`

#### Scenario: Re-admission rechecks closure and revocation
r[lattice.agent_revisions.sessions.scenario.readmitted]
- GIVEN A closed with a held checkpoint and C pinned to R3
- WHEN A is admitted again, and C is admitted again after R3 or its qualification is revoked
- THEN A MUST be refused as `session_closed`, C as `revision_revoked` or `qualification_revoked`, and an unrevoked live C MUST get its existing pin

### Requirement: Pinning preserves configuration, not privileges
r[lattice.agent_revisions.revocation] Revocation of a revision, a qualification reference or a session MUST be recorded monotonically and idempotently. Revocation of a target the lifecycle references (a retained revision, a recorded or pinned qualification reference, a pinned session) MUST never be refused; the per-kind bound MUST apply only to targets it does not reference. `effect_permitted` MUST refuse any further effect of a session that is closed, revoked, or pinned to a revoked revision or qualification, and admission MUST refuse a revoked selection. A revoked last usable revision MUST stop being last usable. Revocation MUST NOT rewrite pins.

#### Scenario: Revocation blocks old and new sessions
r[lattice.agent_revisions.revocation.scenario.effects]
- GIVEN A pinned to R1 and C pinned to R3
- WHEN R1's qualification, then R3, then session C are revoked
- THEN A's effects MUST be refused as `qualification_revoked`, C's as `revision_revoked` and then `session_revoked`, new admissions under R3 MUST be refused, and the pins MUST be unchanged

#### Scenario: A full revocation set never fails open
r[lattice.agent_revisions.revocation.scenario.capacity]
- GIVEN 1024 recorded revocations of each kind
- WHEN an unreferenced target, then a pinned session, the selected revision and a pinned qualification are revoked
- THEN the unreferenced target MUST be refused as `capacity_exhausted`, and each referenced target MUST be recorded and then block the effects it covers

### Requirement: Retention and uncertain cleanup
r[lattice.agent_revisions.retention] A revision's artifacts MUST be retained while it is desired (unless its preparation failed), selected or last usable, while a live session pins it, while a checkpoint or recovery evidence holds it, or while its preparation is pending or unknown. Cleanup MUST be planned only for unretained revisions with recorded artifacts, MUST be refused for a retained revision, and MUST release artifacts only on an observed `released` cleanup. Releasing a prepared revision's artifacts MUST make it unprepared.

#### Scenario: Uncertain cleanup keeps resources occupied
r[lattice.agent_revisions.retention.scenario.uncertain]
- GIVEN a failed R2 with an occupied artifact
- WHEN its cleanup is observed `unknown` and later `released`
- THEN the artifact MUST stay recorded and planned after `unknown`, and MUST be cleared only after `released`

#### Scenario: Holds keep artifacts
r[lattice.agent_revisions.retention.scenario.holds]
- GIVEN R1 selected with a live session, then a closed session with a checkpoint, then recovery evidence, and an unknown preparation
- WHEN retention is evaluated and R1's cleanup is recorded
- THEN each MUST retain its revision with the matching reason, and cleanup of a retained revision MUST be refused as `retained`

### Requirement: Bounded slots projection for inventory
r[lattice.agent_revisions.projection] `projection::slots` MUST project `lattice.agent-revision-slots.v1` with the desired revision and generation, sorted prepared revisions, qualified scopes with class and revocation, the last usable revision, the selection for new sessions with its rollout generation, and each pinned session with its revision, status and whether its effects are blocked. It MUST NOT project bundles, artifacts, holds or receipts.

#### Scenario: Slots reflect the lifecycle
r[lattice.agent_revisions.projection.scenario.slots]
- GIVEN the fixture lifecycle with R1 and R3 selected in turn and a revoked qualification
- WHEN slots are projected
- THEN they MUST name the desired, prepared, qualified, selected and pinned facts, and a session pinned to the revoked qualification MUST be blocked

### Requirement: Version-checked persistence in the existing store
r[lattice.agent_revisions.persistence] The store MUST keep one `agent_revision_lifecycles_v1` row per definition with a version that increases by one per commit. A commit MUST name the version it read and MUST be refused as a conflict otherwise. The shell MUST reload and reapply a pure transition after a conflict, a bounded number of times, and MUST NOT write when the transition changes nothing.

#### Scenario: A stale version conflicts
r[lattice.agent_revisions.persistence.scenario.conflict]
- GIVEN a committed row at version 1
- WHEN a commit names version 0 and then version 1
- THEN the first MUST be a conflict and the second MUST produce version 2

#### Scenario: An interleaved writer is reloaded and reapplied
r[lattice.agent_revisions.persistence.scenario.retry]
- GIVEN a transition whose commit is preceded by another writer's commit after its load
- WHEN the shell commits it, and when another writer always commits first
- THEN the transition MUST be reapplied to the reloaded lifecycle and keep both changes, and after four attempts the shell MUST report contention without committing it

### Requirement: Injected preparation and qualification ports
r[lattice.agent_revisions.ports] The shell MUST obtain preparation and cleanup observations only through `PreparationPort` (`prepared` with artifacts, `failed` with occupied artifacts or `unknown`; `released`, `failed` or `unknown`) and qualification answers only through `QualificationPort` (`qualified`, `not_qualified` with a reason or `unavailable`) with the exact subject keys, task family, required class and the shell's clock reading. The standalone CLI MUST install the unavailable ports. Fixture ports MUST be deterministic and record every call.

#### Scenario: Fixture qualification answers only the exact subject
r[lattice.agent_revisions.ports.scenario.exact_subject]
- GIVEN a fixture qualification port with an answer for one definition, revision, task family and class
- WHEN another prepared revision of the definition is promoted
- THEN the port MUST be asked about the promoted revision and answer `not_qualified` with reason `subject_mismatch`, and the promotion MUST be refused with that reason

### Requirement: The `agent-revision` CLI
r[lattice.agent_revisions.shell] `lattice agent-revision` MUST provide `resolve`, `desire`, `prepare`, `promote`, `rollback`, `revoke`, `admit-session`, `cleanup` and `inspect`, each printing one `lattice.agent-revision-outcome.v1` JSON document with an outcome or a refusal. A refusal MUST exit non-zero, including a refused `admit-session`. Only `desire` MAY create a lifecycle. `resolve` and `inspect` MUST be inert, and `inspect` MUST NOT create a missing store.

#### Scenario: Standalone preparation reports unknown
r[lattice.agent_revisions.shell.scenario.standalone]
- GIVEN the standalone CLI
- WHEN a request is desired and prepared, and a missing store is inspected
- THEN preparation MUST record `unknown` and select nothing, and the missing store MUST NOT be created

#### Scenario: The CLI exit status follows the refusal
r[lattice.agent_revisions.shell.scenario.exit_status]
- GIVEN the `lattice` binary with its standalone ports
- WHEN `resolve`, `admit-session`, `rollback` and `cleanup` succeed, reconcile a receipt or are refused
- THEN every outcome MUST exit zero and every refusal or error MUST exit non-zero, with `admit-session` refusing `no_selection` and `revision_not_selected`, and `rollback` refusing `qualification_unavailable` and `request_conflict`

### Requirement: A session-admission seam that leaves unmanaged definitions unchanged
r[lattice.agent_revisions.seam] `agent_revision::seam::admit` MUST return `unmanaged` and write nothing when the request's definition has no lifecycle, and otherwise MUST pin the session or return the refusal; a managed definition MUST admit a new session only for its selection. The caller MUST record the session's admission mode durably at admission: `managed` for `admitted`, `unmanaged` for `unmanaged`; a `refused` session MUST NOT start. `seam::effect` and `seam::close` MUST take that recorded mode. For an `unmanaged` session they MUST answer `unmanaged` without reading or writing the store, even after its definition becomes managed. For a `managed` session `seam::effect` MUST answer `permitted` with the pin or `blocked` with the refusal, and MUST block a session the lifecycle does not know, and `seam::close` MUST close the pin or return the refusal. Implementation attempts MUST keep their current behavior until an embedder wires the seam.

#### Scenario: Unmanaged definitions behave as before
r[lattice.agent_revisions.seam.scenario.unmanaged]
- GIVEN a store without a lifecycle for the request's definition
- WHEN the seam admits a session and checks an effect
- THEN both MUST answer `unmanaged` and no row MUST be written; after the definition is managed and promoted, admission MUST pin, a changed revision MUST be refused as `revision_not_selected`, and a revoked qualification MUST block effects

#### Scenario: The recorded admission mode decides
r[lattice.agent_revisions.seam.scenario.admission_mode]
- GIVEN a session admitted `unmanaged` before its definition became managed, and a definition that is then desired, prepared and promoted
- WHEN the unmanaged session acts and closes, a session claimed `managed` but unknown acts, a new session is admitted before and after promotion, and a managed session closes
- THEN the unmanaged session MUST answer `unmanaged` and write nothing, the unknown managed session MUST be blocked as `unknown_session`, admission before promotion MUST be refused as `no_selection`, and the closed managed session MUST be blocked and refused re-admission as `session_closed`
