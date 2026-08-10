# Agent-runtime promotion

## Result

All 14 exact agent-runtime IDs are locally bound across Animus integration tests, internal shell tests, and OpenAI plain-content tests. The review policy contains no promoted rows.

## Exact transition

- baseline missing: 304
- current missing: 290
- removed: exactly the 14-ID allowlist
- added: none
- dangling: zero
- valid: false

Coverage is `815 / 525 / 290` requirements/referenced/missing.

- missing-set BLAKE3: `ba797fa2f3f60697a1941f1b61927af134df835f8f6454bb1febda85ebcfcc7d`
- input BLAKE3: `1748effdc89c6870ec594a5fa1cea97e86594453854e5ba345612513baef3630`
- receipt BLAKE3: `176cd7c6929ec281e53f99f27cefb5c1653c1174fab78adfeaad3126c71f8a75`

Inventory is 290 total, zero reviewed, and 290 unassessed. Review-policy BLAKE3 is `809558e543a0e5e8be4e4a31b7b4c0b3370dc2d26cbcb97eeafcd34c58854757`; inventory BLAKE3 is `60086021e3024bfaafc1827a7d8559151984243aece790986622e48d89ad3e88`.

## Validation

Animus integration/internal-shell, OpenAI, historical completion, promotion, inventory, and Cairn checks pass. Adversarial promotion validation rejects duplicate, missing-binding, stale-policy, removal, addition, dangling, and successful-coverage drift.

## Non-claims

Coverage remains unsuccessful. This proves bounded linkage only, not arbitrary agent correctness, release readiness, or production readiness.
