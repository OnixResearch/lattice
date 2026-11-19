# Projected role-protocol execution

Lattice executes one frozen Choregraph role-projection cohort. The cohort source is Choregraph revision `fcbae0aa17bc0eab1345529adb9ff104a46540ef`.

## Intake

The intake adapter receives the Choregraph global protocol, complete ordered local set, and projection bundle. It checks these items:

- exact global, local, local-set, schema-set, proof, and receipt identities;
- exact packed Preserves bytes for each supplied DTO;
- the frozen Choregraph and Trellis revisions;
- one local artifact for each declared role; and
- normalized role order.

The adapter maps accepted producer data into Lattice-owned types. It does not compile or project a second protocol.

## Placement

A session plan contains one placement for each role. A native placement binds one runtime route and adapter cohort. An external placement also binds its contract, schemas, destination policy, credential reference, limits, and adapter cohort.

A missing, extra, reordered, mixed-kind, or mismatched placement fails before session persistence. Placement data does not grant authority.

## Published runtime profile

r[impl lattice.role_protocol.runtime_profile]

`profiles/role-protocol-runtime-v1.ncl` defines the reviewed consumer profile. Its canonical JSON and BLAKE3 sidecar are under `profiles/generated/`.

The profile binds runtime revision `cbb5db0e4844b466dcd1877218eb1f00a2a7caf2`. It also binds the selected source manifest, package, binary, command prefix, state backend, and authority contract.

The Nix profile check verifies that manifest against the separately pinned published runtime source, not the evolving checkout. Current protocol CLI boundaries and effect-authority behavior are checked independently.

The profile supports create, inspect, recover, retry, abandon, and replay. It describes an operator-invoked one-shot service that does not start at boot.

The CLI does not dispatch protocol effects. It uses unavailable authority and empty adapter registries. Effect dispatch remains a Rust library surface for an authorized product adapter.

OnixOS can use this profile for compatibility checks and dry-run service planning. OnixOS still owns the machine, service user, paths, package derivation, policy references, and activation decision.

A future daemon or dispatching CLI requires a new profile identity and separate review. Consumers must fail when requested behavior exceeds this profile.

## Durable order

A dispatch uses this order:

1. Plan one exact local action.
2. Persist the `Prepared` attempt and all canonical envelopes.
3. Evaluate the current Choregraph effect-authority boundary for each envelope.
4. Persist the successful authority-attempt references.
5. Preflight every recipient's adapter and exact profile before any adapter effect.
6. Persist `DispatchedUnknown` before the adapter starts.
7. Start the already-selected bounded adapters, in canonical message order.
8. Admit an exact receiver observation for every message.
9. Commit linked sender and receiver cursors only when every recipient accepted.

A crash after step 6 leaves an unknown outcome. Lattice does not resend automatically. A terminal adapter rejection can fail the attempt only when that call guarantees no receiver work started and no earlier message in the action started. Deterministic selection errors fail before step 6. `fail_before_dispatch` refuses an already-unknown attempt.

HTTP 2xx means only that transport returned a success status (`Started`), not receiver acceptance, protocol commit, or external-effect completion. All other HTTP statuses, including 500 and 502, remain `UnknownAfterStart`: a receiver or intermediary may return an error after work occurred. A generic adapter `ObservedFailure` likewise cannot establish rejection. This HTTP profile does not admit rejection receipts from response bodies; exact rejection uses the separate observation surface.

Each recipient needs its own exact observation. One accepted recipient cannot resolve another recipient's unknown outcome. Mixed accepted/rejected fanout cannot atomically commit and cannot become a terminal failure that permits a blind repetition of the accepted recipient. It remains blocked pending explicit duplicate-risk recovery. All-rejected exact observations may settle as failed.

## Envelopes and choices

Each envelope binds the protocol, complete local set, session, source and destination instances, local artifacts, step, message, attempt, label, payload tag, optional value, payload reference, schema set, adapter cohort, and authority request.

Choices use exact labels. Lattice does not convert a label set into a Boolean. A stale, unknown, duplicated, reordered, wrong-role, wrong-step, wrong-value, or cross-session envelope cannot advance a cursor.

## Ownership

An owned value moves only after the exact receiver observation commits. The prior owner cannot send that value again until a later admitted transfer returns ownership.

This local commit does not prove remote byte uniqueness or remote durable storage.

## Recovery and replay

The typed operator surface supports create, inspect, recover, retry, abandon, and replay requests. Retry requires explicit duplicate-risk acknowledgement. Abandon requires explicit unknown-outcome acknowledgement. A retry receives a new action and attempt identity.

Replay reads the persisted plan and events. It recomputes pure state only. It does not load fresh authority or call an adapter.

Agent-facing summaries and transcripts are not session records. A truthful correction must append to the original attempt history; a rewritten handoff saying `done` or a stale summary saying `failed` changes neither authority, attempt disposition, nor operation identity. Reconstruct a handoff from the persisted plan/events, not from prose. Missing or currently unavailable observations are not evidence that a receiver did nothing.

`Runtime::observe` is an embedding trust boundary: the authorized product observer must authenticate the evidence reference against its current exact receiver contract before calling it. Core admission then checks the canonical session, attempt, message, roles, instances, artifacts, step, label, payload, and adapter context. The bare evidence identity is not a signature or an external-effect verifier. Transport delivery, receiver acceptance, local protocol commit, and external-effect completion remain distinct; acceptance alone need not mean completion under another product's receiver profile.

## Deterministic uncertainty qualification

The regression target `tests/test_role_protocol_uncertainty.rs` defines a local receiver contract with an independently persisted JSON operation/effect ledger, separate from the redb session store. It records canonical message identities, a stable logical-operation key, receiver generations, delivery/effect counts, completion/rejection and compensation counts. The logical key binds the session, protocol cursor, source, recipient and payload context while excluding fresh attempt, message and authority-request identities; separate fanout recipients remain separate operations. This fixture profile produces an acceptance observation only for its exact current completed operation, a rejection only for an exact recorded no-effect rejection, and no observation while in progress. No-start rejection records have zero deliveries and effects, including the later C recipient; they pass through the same current-record observer as completion records. Stale generations, substituted records, wrong/cross-session/attempt/message/role/adapter contexts and duplicate observations are negative inputs.

The matrix varies not-applied/applied-once/in-progress receiver state independently of 202/500/502/lost-ACK transport responses. It uses unchanged history, additive truthful corrections, deliberately false rewritten `done` handoffs, and stale `failed` handoffs, both directly and after a real child-process exit/database reopen. A deterministic handoff reader consumes every selected transcript line and derives its proposed completion/retry intent. Safe reconstruction derives permitted disposition and completion claims from authoritative replay plus the independent ledger and admitted per-message observations, never from the proposed intent. Replay and handoff reconstruction never dispatch. Three-role projected choice fixtures cover missing/mismatched later profiles, partial fanout, exact per-message reconciliation, and acknowledged recovery. An independent checker aggregates effects and deliveries by logical operation across fresh attempt/message identities and rejects false completion, duplicated effects/deliveries and unjustified compensation. Broken controls consume the false summaries: `done` produces a false completion claim, and `failed` causes actual persisted duplicate receiver work, both as a same-message resend and as a fresh canonical `retry_unknown` attempt. The latter control invents duplicate-risk acknowledgment from the summary, not from an operator; it is not safe automatic recovery.

These are deterministic regression definitions, not a recorded passing run or a qualification of the immutable published runtime revision. No model behavior, remote crash durability, transport authentication, or distributed exactly-once guarantee is inferred. Run the focused target under the pinned development shell to obtain current execution evidence.

## Declared preference factors

r[impl lattice.role_protocol.preference_factors]

[Lattice #16](https://github.com/OnixResearch/lattice/issues/16) consumes the default-off annotation profile that [Choregraph #4](https://github.com/OnixResearch/choregraph/issues/4) published at the pinned revision `fcbae0aa17bc0eab1345529adb9ff104a46540ef`. The related Noble filing is https://github.com/OnixResearch/noble-lang/issues/67#issuecomment-6796434640.

The producer profile selector is `choregraph.role-choreography.profile.preference-annotations.v1`. Choregraph's own default, `choregraph.role-choreography.profile.cooperative-baseline.v1`, refuses annotated input. An annotated bundle wraps the unchanged v1 projection: `AnnotatedBundle::base()` and `projected()` feed the existing intake unchanged.

`admit_annotated_projection` takes the annotated bundle, the transported producer receipt (body and packed bytes), the annotated complete local set, and a Lattice `PreferenceProfile`. It performs these steps:

1. It admits `base()` and `projected()` through the unchanged `admit_projection`.
2. It verifies the receipt with `preference::linkage::verify_receipt` and the annotated local set with `verify_local_set`. A receipt or local set from another cohort, or one with a changed byte, fails with `protocol.identity-mismatch`.
3. It checks that the receipt names the annotated profile, the `identity-linkage-only-no-claim-promotion` claim, a `not-formalized` game mapping, and the producer's nine annotation non-claims. It also checks that the receipt binds the admitted v1 receipt, global artifact, ordered local artifacts and complete local set.
4. It returns the cohort and a `PreferenceReport`.

The session profile is explicit and off by default:

- `exact-label-baseline` (the default) returns exactly the cohort that `admit_projection` returns. The report says `factors-ignored-profile-off`. No factor data enters the session, so its plan, session and evidence identities are byte-identical to the unannotated path.
- `record-declared-factors` adds a `PreferenceRecord` to the cohort. The report says `factors-recorded`.

Every report also states `game_mapping = not-formalized`. Pact's solver and game semantics are unpublished author claims; Lattice does not use them as a session admission check.

The record keeps, for every role in normalized order, its local artifact identity and decider choices. Each choice has its global and local paths, its exact projected labels, its declared factors (id, owning role and name) and its optional world variable (id and name). The record also binds the producer profile, the annotated receipt, the v1 receipt, the complete local set, and the annotation and annotated-local-set identities. Spans are dropped.

The record is part of the cohort, so it is bound into the session-plan identity. `plan_session`, which `create_session` repeats, also rejects a record in these cases:

- it names another profile or drops a non-claim;
- it names another v1 receipt or complete local set;
- it is missing a role, or a role is reordered;
- a choice's local path does not reach that role's `InternalChoice`, or the choice does not list exactly that node's labels in order;
- a choice has no factors, or a factor is owned by another role.

Unknown record fields, such as a selected label, weight, utility or authority field, fail deserialization.

Factors are metadata only. Action planning, envelope binding, observation admission, cursor commits, ownership and the authority binding never read them. The exact-label rules above are unchanged: an undeclared, stale, reordered, wrong-step or wrong-role label, or a factor or world identifier used as a label, still cannot advance a cursor. Both labels of a recorded choice remain plannable, so factors do not pick a branch or turn a label set into a Boolean. Recorded factors are not credentials, approvals, authority or workflow-revision merge semantics.

Session inspection gains a `preference` block only when a record exists. The block holds `factors-recorded`, `not-formalized`, the producer profile, the annotated receipt and local-set identities, the per-role choices with factor and world names, and the non-claims. Names are the producer's validated lowercase kebab identifiers of at most 64 bytes. The block is absent for unannotated and `exact-label-baseline` sessions.

The standalone CLI is unchanged. Its create input accepts only the v1 choreography and rejects an annotated source before a session exists. It still dispatches no effects and has unavailable authority. Recording factors is a Rust library surface that the embedding caller selects. `tests/test_role_protocol_preference.rs` covers the recorded, ignored, comparison, crossed-cohort and forged-record cases.

## Safe evidence

Inspection reports bounded identities, status, counts, attempts, outcomes, issue codes, and the claim boundary. It omits payload bytes, commands, credentials, endpoint secrets, source text, and host paths.

The evidence reports local artifact admission, cursor decisions, durable attempt linkage, and bounded observations. It does not claim transport confidentiality, endpoint identity truth, remote durability, external-role correctness, exactly-once delivery, global progress, fault deadlock freedom, production readiness, or release eligibility.
