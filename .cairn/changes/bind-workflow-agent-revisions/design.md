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

`propose` compares two bundles of one definition and reports the new revision with its changes (compiler, plan, graph, selection, requirements, prompt, context, policy, runtime profile, worker revision, executable, tools, model route, compute, task families, protocol). The applicability decision belongs to Cairn #14 and Kiln #83; the type admits only `unknown`, so a decoded decision is refused. The proposal changes no lifecycle; it reaches new sessions only through desire, preparation, qualification and promotion.

## Decision 4: identities only in the Kiln projection

`lattice.workflow-agent-revision-slots.v1` projects desired, last usable, selected and pinned revisions, each with the compiled identities (compiler revision and artifact, plan, graph, selection, obligations, policy) and the worker (definition, name, revision, bundle identity) apart. Guidance, context, source and per-obligation identities stay out. A lifecycle of plain agent bundles has no slots, and a referenced revision that is missing or not a workflow-agent bundle is refused. Publication to Kiln #84 is later work.

## Decision 5: the attempt seam is documented, not wired

`implementation-attempt` is owned elsewhere this round. `workflow::check_attempt(bundle, profile_revision)` refuses an attempt whose worker revision is not the one the bundle binds (`revision_not_selected`). The request change for the parent is an optional field naming the workflow-agent definition and revision; `seam::admit` would then pin that revision after `check_attempt`.

## Decision 6: the package comparison is a test harness

The comparison of the agent-package draft lives in `tests/test_agent_package_comparison.rs` over inert fixtures. The fixtures carry the draft's manifest and Package Config plus the facts a materialization owner would report; the harness compares them and derives the BLAKE3 inventory, materialized, closure and evidence-subject identities, keeping `sha256:` digests as provider-native fields. It never reads an archive. The decision is "useful mapping, adoption deferred", so no production code is added for it.

## Risks

- The composite revision is spelled like a profile revision. Consumers must key on the definition as well, which they already do.
- The comparison's decompression bounds are fixture choices; the draft sets none.
