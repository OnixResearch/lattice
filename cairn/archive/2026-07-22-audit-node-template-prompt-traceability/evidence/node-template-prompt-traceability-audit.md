# Node-template prompt traceability audit

## Result

The complete three-row node-template prompt tranche is explicitly classified:

| Requirement | Classification | Rationale |
|---|---|---|
| `n8nclone.node_templates.parameters.prompt` | `exact_maintained_evidence` | State-machine assertions observe template selection, parameter-value collection, confirmation, and rendered node command creation. |
| `n8nclone.node_templates.parameters.prompt.cancel` | `exact_maintained_evidence` | The test enters parameter state, cancels, and observes zero nodes, closed flow, and unchanged dirty state. |
| `n8nclone.node_templates.parameters.prompt.required` | `partial_evidence_only` | App state and panel implementation expose the required parameter step, but no maintained assertion inspects the user-visible required prompt. |

Evidence bindings:

- `src/tui/app.rs`: `edc890f88bd0e8d5f76e66fc6ae2459b0330c94eb0116d0d85682a0a027c3984`
- `src/tui/panels.rs`: `1a66d29221790dbd366e0f517f9b0cb119b2061765567c0a290b65e7f2e83c09`

## Portfolio registry

| Family | Mechanism | State | Evidence / rejection |
|---|---|---|---|
| clause-match | Match each accepted clause to direct assertions | validated for parent and cancel; partial for required | Collection and cancellation are directly observed. User-visible required prompt text is not asserted. |
| state-machine | Trace template selection, parameter entry, confirmation, cancellation, and mutation | validated | Focused app tests prove the state transitions and workflow mutation boundaries. |
| provenance-exclusion | Exclude test names, implementation-only rendering, archives, generated reports, and broad cancellation tests | validated | None of those artifacts can strengthen the unasserted required-prompt surface. |

These were correlated serial repository passes, not independent agents.

## Adversarial audit

The test name `prompts_and_renders` was rejected as evidence by itself. `render_add_node_parameter_lines` demonstrates implementation shape but cannot establish rendered behavior without an assertion. Generic add-node cancellation was rejected in favor of the dedicated parameter-entry cancellation test. Archived tasks and generated inventory were excluded as authorities.

The full repository check also exposed a cross-tranche test defect: the earlier workflow-example audit pinned global reviewed totals. That coupling was removed; both audits now validate only their selected rows while the inventory suite owns global totals.

## Coverage and inventory

Coverage remains unchanged and unsuccessful:

- baseline/current missing: `327 / 327`
- current requirements/referenced: `761 / 434`
- additions/removals: none
- dangling: 0
- `valid: false`
- policy BLAKE3: `f5188ad02b20fe2c5b86fb6a890b8d90402eed165b9f075c0df96044a3bad177`
- input BLAKE3: `c7e2b28da8de4014735786d0d0c9eb46bfc215618ce21b60849215dac4ba7129`
- receipt BLAKE3: `52e712051091abfcf416456ba419e83de0f8e92f0c2fe7f8c5ead15f534f59dd`

Inventory:

- total: 327
- reviewed: 5
- exact: 3
- partial: 1
- no exact identified: 1
- unassessed: 322
- missing-set BLAKE3: `1c8d9ebc7d34a24365f332e8d386cc1d905601d6541faaa254e3f6d2cd4b8aa4`
- review-policy BLAKE3: `2297fe3766afcc4a27222f620783f09c3b6d6e66bb0975e8742c429ceb19b503`
- inventory BLAKE3: `ed8f58b6bae4e02cd7930935402aa3a58715674568dbc0faddb47804411b75ce`

## Validation

Focused template-parameter and inventory tests passed before policy changes. Positive and negative audit tests, prior workflow-example audit tests, `scripts/check.rs`, Cairn validation and gates, deterministic inventory generation/checking, Octet zero-delta validation, and `nix flake check -L --option secret-key-files ''` pass.

Octet remains at 912 accepted/current findings with zero new or stale IDs and baseline `b3:d5e10724e6dad828c224621686ac378f98dd2a592b35dc3091bd26ba6cd7ebcd`. Its underlying Cargo subprocess remains classified as exit code 101 while the wrapper accepts the reviewed zero-delta result.

## Budget and stop condition

Declared budget: five repository-owned root/file groups, three mechanism families, one adversarial pass, one focused validation pass, and one full validation pass; no web retrieval or advisory-model authority. Used: all three families and both validation passes. Search stopped at the validated complete three-row partition.

## Non-claims

Partial evidence is not near-complete correctness. Inventory classification creates no Tracey evidence. This audit does not prove arbitrary TUI behavior, complete traceability, release readiness, or production readiness.
