# Workflow-example traceability audit

## Result

The complete two-row workflow-example tranche is explicitly classified:

| Requirement | Classification | Rationale |
|---|---|---|
| `n8nclone.workflow_examples.image.invalid` | `no_exact_evidence_identified` | No maintained intentionally invalid workflow-example fixture or fixture-level no-partial-state test exists in reviewed roots. |
| `n8nclone.workflow_examples.image.selfie_to_fine_art` | `exact_maintained_evidence` | The checked-in Nickel example and direct integration test observe import, three image inputs, fake Clankers execution, all-node completion, and generated image output. |

The exact row binds:

- `examples/workflows/selfie-to-fine-art.ncl`: `2efd38cfd5d615233f307983ce81baf1e0c61563185526e7b4ad00880854dd0e`
- `tests/test_image_workflow_example.rs`: `657bb2f91f7f1a2471d8bd7d9556c1cb87fd41916ea255ce645e757227f099c0`

The no-exact row remains evidence-free.

## Portfolio registry

| Family | Mechanism | State | Evidence / rejection |
|---|---|---|---|
| clause-match | Match each GIVEN/WHEN/THEN clause to direct maintained assertions | validated for selfie; falsified for invalid | The selfie test covers every clause. General unsupported-image tests do not instantiate the required example fixture or observe no partial workflow state. |
| fixture-discovery | Search maintained examples and tests for the named positive and negative artifacts | validated | The positive example exists; no invalid image-example fixture exists. |
| provenance-exclusion | Exclude parent markers, archives, generated reports, and implementation shape | validated | Only accepted specs and historical/generated receipts name the invalid ID; none can establish maintained behavioral evidence. |

These were correlated serial repository passes, not independent agents.

## Adversarial audit

The parent `n8nclone.workflow_examples.image` marker was rejected because parent linkage cannot establish either child scenario. Unsupported-image unit tests were rejected because they do not import or run an intentionally invalid example and do not check partial workflow-state creation. Archived source and generated inventory were rejected as evidence authorities.

The surviving classification is deliberately asymmetric: one exact row and one bounded no-exact row.

## Coverage and inventory

Coverage remains unchanged and unsuccessful:

- baseline/current missing: `327 / 327`
- current requirements/referenced: `757 / 430`
- additions/removals: none
- dangling: 0
- `valid: false`
- policy BLAKE3: `f5188ad02b20fe2c5b86fb6a890b8d90402eed165b9f075c0df96044a3bad177`
- input BLAKE3: `591b0385a73114ebd3bfdc298986fb35fe94488d4ac2677680f02dc46315649e`
- receipt BLAKE3: `5e7d8eb8be8dbe6b21112ea3a105942b70b4726e2fef3d65478d9e4d86bd8469`

Inventory:

- total: 327
- reviewed: 2
- exact: 1
- no exact identified: 1
- unassessed: 325
- missing-set BLAKE3: `1c8d9ebc7d34a24365f332e8d386cc1d905601d6541faaa254e3f6d2cd4b8aa4`
- review-policy BLAKE3: `85dc8e286af21299136d92c713229d33059cf341c30dc26e389da8606293ddb4`
- inventory BLAKE3: `6929946be7a3a84ddfa653d78c44e3f51907d68da005029748ed8a634c60cee1`

## Validation

Baseline image-example and inventory tests passed before changes. Focused positive and negative audit tests, `scripts/check.rs`, Cairn validation and gates, deterministic inventory generation/checking, Octet zero-delta validation, and `nix flake check -L --option secret-key-files ''` pass.

Octet remains at 912 accepted/current findings with zero new or stale IDs and baseline `b3:d5e10724e6dad828c224621686ac378f98dd2a592b35dc3091bd26ba6cd7ebcd`. Its underlying Cargo subprocess remains classified as exit code 101 while the wrapper accepts the reviewed zero-delta result.

## Budget and stop condition

Declared budget: four repository-owned root groups, three mechanism families, one adversarial pass, one focused validation pass, and one full validation pass; no web retrieval or advisory model authority. Used: all three mechanism families and both validation passes. Search stopped at the validated complete two-row partition.

## Non-claims

`no_exact_evidence_identified` is bounded to the reviewed maintained roots and is not proof of global absence. Inventory classification creates no Tracey evidence. This audit does not prove arbitrary image workflow correctness, complete traceability, release readiness, or production readiness.
