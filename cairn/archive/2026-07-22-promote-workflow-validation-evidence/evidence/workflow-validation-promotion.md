# Workflow-validation promotion

## Result

All 29 exact workflow-validation IDs are assertion-locally bound across maintained credential, core validation, n8n, replay, control-flow, provider, runner-policy, subworkflow, and TUI tests. Review policy contains no promoted row.

## Exact transition

- baseline missing: 290
- current missing: 261
- removed: exactly the 29-ID allowlist
- added: none
- dangling: zero
- valid: false

Coverage is `827 / 566 / 261` requirements/referenced/missing.

- missing-set BLAKE3: `de87e38b8b3298301d5872c62c6dc887eba0a3bc2ce9f1556996a470fc3ac902`
- input BLAKE3: `47b5e4b2936d7ab2a3f4836e5bca411d73986b9669ae031eb4d99a00500287b8`
- receipt BLAKE3: `c6b644ecae093d7d7b0c1836dff53eb5125db97e9a0ec6c81a2cfb6dddc179fb`
- review-policy BLAKE3: `ac8e7691cf6b85671be68912fc83d78258a1bc973355eaad6fae35f268b8f239`
- inventory BLAKE3: `aa31434b03bdcdf4b95943aced7b318ba5f0acc5971f66f292c3627cccf29399`

Inventory is 261 total, zero reviewed, and 261 unassessed.

## Validation

Focused behavior, historical audit/completion, promotion, inventory, and identity tests pass. Adversarial promotion validation rejects duplicate, missing-binding, stale-policy, removal, addition, dangling, and successful-coverage drift.

## Non-claims

Coverage remains unsuccessful. Promotion proves bounded linkage only, not arbitrary n8n parity, host sandboxing, complete traceability, release readiness, or production readiness.
