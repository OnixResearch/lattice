# Webhook cookbook credential promotion evidence

## Result

Exactly `n8nclone.credentials.webhook_cookbook_secret_safety` moved from reviewed debt into maintained Tracey evidence. No runtime behavior changed.

The marker is attached directly to both exact supporting tests in `tests/webhooks.rs`:

- `webhook_cookbook_valid_request_starts_one_run_and_records_safe_history`
- `webhook_cookbook_rejected_requests_do_not_start_runs_or_expose_secrets`

Together those tests observe accepted safe history, rejected responses and no-run history, credential-reference workflow/docs artifacts, and omission of provider values, authorization values, and sensitive payload fields.

## Portfolio and adversarial audit

Child-marker inference, file-scope binding, generated inventory binding, and a single rejected-test binding were rejected. Dual exact-test binding was selected because the accepted and rejected smoke surfaces are both required.

The first full repository check correctly rejected the historical 14-row audit after the new marker changed `tests/webhooks.rs`. The historical audit now reconstructs the pre-promotion bytes by removing exactly two complete allowlisted marker lines, verifies their count, checks that reconstructed bytes retain the audited BLAKE3 identity, and continues rejecting markers for every unrelated partial row. This preserves historical byte evidence without accepting arbitrary source drift.

## Exact transition

- baseline requirements/referenced/missing: `722 / 381 / 341`
- current requirements/referenced/missing: `726 / 386 / 340`
- baseline minus current: exactly `n8nclone.credentials.webhook_cookbook_secret_safety`
- current minus baseline: empty
- baseline dangling: 0
- current dangling: 0
- current `valid: false`

Current coverage identity:

- policy BLAKE3: `f5188ad02b20fe2c5b86fb6a890b8d90402eed165b9f075c0df96044a3bad177`
- input BLAKE3: `e5b7f725f24d0dedc06209f65a6e0cfc04b6b960b436ed59ff930140a8cf31d3`
- receipt BLAKE3: `7a9ef4ae04d59511bcb285922138c827017dd04e75fe7b4dbd8421a87134059a`

## Current inventory

- total: 340
- reviewed: 13
- partial: 13
- exact: 0
- unassessed: 327
- missing-set BLAKE3: `e0ce72fe7f22dfe1fb007d33d93d6bc0b61b9efbc389339e9a0ed8656bc6f13b`
- review-policy BLAKE3: `28f6d4190587ec345d019f53b851f1f02b0598acee4b3c01e187a01ecd440c37`
- inventory BLAKE3: `29fd52ee16d35b5503ee31045e2ad75aa73c972df78cbec32228729392bf15f9`
- inventory `valid: true`

## Positive and negative checks

The maintained promotion tests prove exact attachment and set transition. Negative mutations reject duplicate baseline/current/allowlist IDs, a missing expected removal, an unrelated removal, an unexpected addition, a dangling marker, and successful coverage with deterministic issue codes.

Validation completed:

- focused webhook, promotion, historical audit, and inventory tests;
- `scripts/check.rs`;
- deterministic inventory generation/checking;
- Cairn validation and proposal/design/tasks gates;
- Octet with 912 accepted/current findings, zero new, and zero stale under baseline `b3:1420c9a0b1e03612f4cec47bee348cbab937873c4b03fcae75d6d0d7a22257a2`;
- `nix flake check -L --option secret-key-files ''` for configured `x86_64-linux` checks.

Octet's underlying Cargo subprocess remains classified as exit code 101 while the reviewed wrapper accepts the zero-delta result. Nix omits incompatible `aarch64-darwin`, `aarch64-linux`, and `x86_64-darwin` systems.

## Non-claims

This promotion establishes only maintained evidence linkage and its one-ID set transition. It does not prove arbitrary webhook confidentiality, external provider correctness, complete traceability, release readiness, or production readiness.
