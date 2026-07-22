# Agent-runtime traceability audit

## Result

All 14 agent-runtime rows are reviewed: 7 exact and 7 partial.

Exact evidence covers existing route preservation, tool denial without a model effect, fixed-loop completion, turn exhaustion, runner execution, typed text/JSON output with safe metadata, and unchanged plain OpenAI-compatible content.

Partial rows retain explicit gaps for missing-route state preservation, complete credential omission, redacted no-fallback provider failure, identity drift before transport, inert authority-bearing output, incomplete-status mapping, and unsupported-input zero dispatch.

## Coverage

Coverage remains `806 / 502 / 304` requirements/referenced/missing, with zero dangling markers and `valid: false`.

- missing-set BLAKE3: `dcff4383f0a161102646ca402e01deb4dd684e68e21821c4131638eaa3180fc7`
- input BLAKE3: `0f2ff59f6a9a97b1ec1cad435789e3c30fb52930c75516df1f35bd84bab3bd5a`
- receipt BLAKE3: `3b0b1050225b000ad1c1ba98e6ac180edfb9dfe7a67ccce065ad72c09b68b9c9`

Inventory is 304 total, 14 reviewed, 7 exact, 7 partial, and 290 unassessed. Review-policy BLAKE3 is `5aff825a7d77c6e379c7e1304303ac8e2591c004daf1193d88ced6918546f2eb`; inventory BLAKE3 is `95544f2893b8276963542cbfc38e2de8f6bc90792f83d8490ee46ef601c6d523`.

## Validation

Focused Animus/OpenAI tests passed before classification. Positive and adversarial audit validation, inventory generation/checking, and Cairn gates pass.

## Non-claims

Partial evidence is not near-complete correctness. This audit does not prove arbitrary model, provider, tool, orchestration, release, or production behavior.
