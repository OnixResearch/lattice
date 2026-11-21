# Design: workflow-agent revisions and the OCI package comparison

## Context

Baseline: Lattice `db64cd654e52cf6579643e6e96ad92bda2b3d61a`.

- `lattice_core::agent_revision` keeps one lifecycle per definition, keyed by an opaque bare-hex revision, with a `lattice.agent-bundle.v1` bundle per candidate. A qualification attachment answers for an exact definition, revision and task family.
- `workflow::intent::compile` produces `lattice.workflow-intent-plan.v1` with a `b3:` plan identity, a Choregraph machine identity and the host-normalized Cairn cohort.
- The issue's acceptance for workflow pinning depends on Lattice #26, Cairn #14 and Kiln #83/#84; none supplies a producer cohort yet, so this change works at fixture level.

## Decision 1: one bundle, one lifecycle, no second controller

A workflow-agent bundle is an ordinary lifecycle `Bundle` with an optional `workflow` binding, not a second lifecycle or registry. Its definition (`lattice-workflow-agent-definition-v1` over the pairing name and the worker definition) never equals a worker definition, so the two never share a lifecycle row. Its revision (`lattice-workflow-agent-revision-v1` over the canonical binding) changes with any plan or worker change, so every existing rule applies unchanged: a failed or late preparation keeps the last usable revision and the selection, promotion is explicit and generation-checked, pins never move, revocation blocks effects and rollback serves only new sessions.

The worker's own fields stay in the bundle, and validation rebuilds the worker bundle and compares its identity with the binding's. A bundle whose name, schema, plan or worker fields disagree with its definition, revision or recorded worker identity is `malformed`.

Rejected: a separate `lattice.workflow-agent-bundle.v1` type stored beside the lifecycle. Inspection and the Kiln projection would then need bundles supplied from outside and could disagree with the lifecycle.

Rejected: framing the plan into the worker profile revision. That would change every existing revision and let a worker qualification cover a plan.

## Decision 2: qualification answers for the pair

The qualification query names the bundle's own definition and revision. An attachment for another plan with the same worker, or for the worker alone, is `not_qualified` / `subject_mismatch`, and reusing another revision's reference is `qualification_mismatch`. A positive prior evaluation therefore never approves a different intent.

## Decision 3: adaptations are data with an unknown applicability

`propose` compares two bundles of one definition and reports the new revision with its changes (compiler, plan, graph, selection, requirements, prompt, context, policy, runtime profile, worker revision, worker bundle, worker name, executable, tools, model route, compute, task families, protocol). `worker_bundle` is set whenever the recorded worker bundle identity changes, so every difference that changes the revision is classified and a valid adaptation is never `malformed`; `worker_name` names a worker definition-name change that keeps the worker definition and revision. The applicability decision belongs to Cairn #14 and Kiln #83; the type admits only `unknown`, so a decoded decision is refused. The proposal changes no lifecycle; it reaches new sessions only through desire, preparation, qualification and promotion.

## Decision 4: identities only in the Kiln projection

`lattice.workflow-agent-revision-slots.v1` projects desired, last usable, selected and pinned revisions, each with the compiled identities (compiler revision and artifact, plan, graph, selection, obligations, policy) and the worker (definition, name, revision, bundle identity) apart. Guidance, context, source and per-obligation identities stay out. A lifecycle of plain agent bundles has no slots, and a referenced revision that is missing or not a workflow-agent bundle is refused. Publication to Kiln #84 is later work.

## Decision 5: an attempt names its workflow-agent revision; one function selects the subject

`workflow::check_attempt(bundle, profile_revision)` refuses an attempt whose worker revision is not the one the bundle binds (`revision_not_selected`) and a plain agent bundle (`malformed`).

The implementation-attempt request gains an optional `workflow_agent` `{definition, revision}`. It is per-attempt, bound by the exact request bytes like `lineage`, and not framed into the profile revision: the bundle already binds that revision, so framing the pair into it would be circular. Absent, it is not serialized, so requests keep their bytes, identity and profile revision.

`seam::subject(request)` is the only subject selection: the named pair, else the worker's definition and profile revision. `seam::admit`, `effect`, `retained` and `close` each call it, so the implementation-attempt shell (`revision::{admit, permit, recheck, close, exportable}`), recovery and the per-step repair recheck all key on the same subject without passing it around. `admit-session --request` uses it too, after the same `Request::validate` that `run` applies, so it never pins a request that fails that validation. With a named pair, `seam::admit` refuses a malformed pair as `malformed`, a definition without a lifecycle as `unknown_revision` (a named pair is never `unmanaged`), and, inside the lifecycle transaction that pins the session, a missing candidate as `unknown_revision` and a failing `check_attempt` with its refusal. A named pair without `--revision-store` is an error before any write.

`run` admits the session under the per-work fence lock, after reading the intent store and recording the fence and before the binding and intent (`fence-intent-creation-atomically` Phase 3). The request file's fields outside the envelope (`workflow_agent`, `lineage`) let two valid requests share one attempt-fence and session; with the intent store read first under the lock that creates intents, only one of them can pass admission, and the other is `substituted_request` (`dispatched: true`) without pinning. A `revision_not_admitted` refusal closes the attempt-fence before it is printed with `dispatched: false`, so no request for it can create an intent afterwards.

`effect`, `retained` and `close` compare the full subject: a pin or recorded permit of another revision than the request's is blocked or refused as `session_conflict`, and `close` then closes nothing, so a request naming one revision never acts on a session pinned to another revision of the same definition. They do not repeat `check_attempt`: the bundle is immutable per revision and the request's profile revision is fixed by its bytes, so closure does not depend on the candidate still being stored. `inspect` keeps the worker profile facts on the worker definition's lifecycle, which the projection requires, and reads the attempt's pin from the session subject's lifecycle.

Rejected: putting the subject choice in `cli/session/revision.rs`. The operator command `admit-session --request` also admits a request's session, and a second selection there could disagree; the seam is the one place both callers share.

`admit-session` for an attempt's own session (its envelope identity) is attempt-bound (final review at `ba6ed91d`): it needs `--state` and calls `cli::execution::admission::admit_session`, which runs the same `Context::admit_first` that `run` runs (preflight with the workspace boundary and bounded-step timeout, lock, orphaned-pin reconciliation, intent store, gates with the standalone CLI's ports, fence, write-ahead record and session) and stops after admission, so its refusals are `run`'s and it pins nothing on any of them. Any other session id is no attempt's and keeps `run`'s request validation before `seam::admit`. Rejected: a separate operator path with its own fence checks, which could drift from `run`'s.

## Decision 6: the package comparison is a test harness

The comparison of the agent-package draft lives in `tests/test_agent_package_comparison.rs` over inert fixtures. The fixtures carry the draft's manifest and Package Config plus the facts a materialization owner would report; the harness compares them and derives the BLAKE3 inventory, materialized, closure and evidence-subject identities, keeping `sha256:` digests as provider-native fields. It never reads an archive. The decision is "useful mapping, adoption deferred", so no production code is added for it. The harness recomputes per-file SHA-256 with the hash provider of the *ring* suite `TLS13_AES_128_GCM_SHA256` in the existing `rustls` dev-dependency, checked against the standard vectors. A new dev-dependency would change Octet's root configuration and profile hashes and re-key every root finding, which the baseline migration cannot follow, so the manifest stays unchanged.

## Risks

- The composite revision is spelled like a profile revision. Consumers must key on the definition as well, which they already do.
- The comparison's decompression bounds are fixture choices; the draft sets none.
