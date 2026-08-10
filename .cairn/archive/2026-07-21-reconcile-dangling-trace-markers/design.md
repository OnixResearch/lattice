# Design: Reconcile dangling Tracey markers

## Context

The corrected `cairn-default` profile scans 12 accepted requirement files and 123 Lattice-owned evidence files. Its baseline receipt contains 696 accepted requirements, 327 referenced requirements, 369 missing requirements, and eight dangling marker identifiers. Coverage is correctly invalid.

The eight identifiers are not one uniform defect. One names behavior that is tested but absent from accepted requirements, one is an obsolete persistence alias whose test can prove the accepted save scenario after strengthening, two support only an accepted parent requirement, and four would overclaim narrower scenarios if renamed mechanically.

## Goals and non-goals

### Goals

- Reduce the exact dangling set from eight to zero.
- Preserve the 369 unrelated missing requirements and fail-closed exit.
- Add only accepted behavior that existing evidence establishes exactly.
- Make stale-marker absence and new-marker linkage regression-testable.

### Non-goals

- Do not bulk-add markers for missing requirements.
- Do not claim complete scenario coverage from partial tests.
- Do not change runtime trigger, template, credential, or workflow behavior.
- Do not claim semantic correctness, complete evidence, or release readiness.

## Decisions

### Decision: classify markers before changing them

Each marker is reconciled through one of three explicit classes:

1. **Exact accepted behavior**: add `n8nclone.node_templates.parameters.invalid.bounds` as a narrow above-maximum scenario because the test checks the exact `NumberAboveMaximum` outcome.
2. **Supported accepted identity**: replace `n8nclone.scheduling.triggers.model.persist` with `n8nclone.scheduling.triggers.model.save` only after the test compares the complete saved and loaded trigger definition. Replace the two webhook-auth aliases with the accepted parent `n8nclone.credentials.webhook_auth`, which is the strongest identity their credential-reference and secret-omission assertions support. Bind the headless trigger-plan implementation to the already referenced `n8nclone.scheduling.webhook_listener.admission.valid` scenario because it starts the configured workflow and records accepted trigger history.
3. **Unsupported stale alias**: remove `admission.enabled`, `webhook.method`, and `webhook.content_type`. The surrounding tests remain, but their stale comments do not become evidence for broader accepted scenarios that the individual assertions do not completely establish.

**Rationale:** Traceability is stronger when unsupported evidence is absent than when approximate names create a false coverage gain.

### Decision: preserve the missing set as an invariant

The accepted bounds scenario and marker-reconciliation scenario each arrive with exact evidence, so requirement and referenced counts increase together. All other target IDs are already referenced, or stale comments are removed. Therefore the 369-item missing set MUST remain byte-for-byte unchanged.

**Rationale:** The bounded change addresses dangling identity only. A lower missing count would indicate accidental evidence promotion and fail review.

### Decision: add a focused source/spec alignment guard

A repository test uses checked `include_str!` inputs to prove the two new accepted IDs have matching test markers and that all seven retired dangling strings are absent from the affected source files.

**Rationale:** The maintained coverage command must continue exiting unsuccessfully while real missing debt exists, so its nonzero status cannot serve directly as a green CI gate for dangling-only invariants.

## Bounded search registry

| Family | Mechanism | Result | Reason |
|---|---|---|---|
| Mechanical rename | Map every stale suffix to the nearest accepted scenario | Rejected | Several tests establish only part of the target scenario and would manufacture coverage. |
| Delete every dangling comment | Remove all eight identifiers without accepted-spec review | Rejected | It would discard exact bounds evidence and avoid strengthening the persistence claim. |
| Classified reconciliation | Add exact behavior, strengthen one test, bind four markers to supported accepted identities, and retire three unsupported aliases | Selected | It removes identity errors while preserving the missing set and fail-closed status. |

An adversarial review specifically flagged the persistence test's partial field assertions and the risk of obscuring webhook/admission semantics. The selected design responds by comparing the complete trigger and refusing approximate scenario mappings.

## Validation

- Baseline and post-change Tracey receipts with exact missing-set comparison.
- Focused positive and negative marker-alignment tests.
- Existing trigger and template-parameter tests.
- `scripts/check.rs` and bounded Nix flake checks.
- Cairn proposal, design, tasks, validation, sync, and archive gates.
