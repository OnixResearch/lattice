# Design: promote webhook cookbook credential evidence

## Goal and completion evidence

The promotion is complete only when deterministic checks prove all of the following:

- the baseline receipt has exactly 341 missing IDs;
- the current receipt has exactly 340 missing IDs;
- baseline minus current is exactly `n8nclone.credentials.webhook_cookbook_secret_safety`;
- current minus baseline is empty;
- both receipts have zero dangling markers and remain `valid: false`;
- the marker is attached directly to both exact supporting cookbook smoke tests; and
- the current debt policy no longer contains the promoted ID.

## Selected mechanism

Attach the same requirement marker to the accepted-request test and the rejected-request/artifact test. Together they directly observe safe accepted history, rejected responses and no-run history, credential-reference cookbook artifacts, and omission of provider values, authorization values, and sensitive payload fields.

A pure promotion validator compares decoded in-memory receipts and reports deterministic issues for duplicate, unsorted, missing-removal, unrelated-removal, addition, dangling, or successful-coverage transitions. The test shell owns only checked-in fixture inclusion.

## Portfolio audit

- **Child-marker inference:** rejected because child markers do not establish the reviewed parent row.
- **Single rejected-test binding:** rejected because it does not directly bind accepted smoke history.
- **Generated inventory binding:** rejected because inventory rows do not create Tracey evidence.
- **Dual exact-test binding:** selected because it spans the accepted and rejected smoke surfaces without file-scope or archive inference.

## Fail-closed validation

Negative mutations introduce duplicate baseline/current IDs, an unexpected removal, a re-added promoted ID, an unaccepted addition, a dangling marker, duplicate allowlist entry, and successful coverage. The validator must reject every mutation with deterministic issue codes.

## Claim boundaries

The promotion proves only an exact maintained linkage transition. It does not prove arbitrary webhook confidentiality, external provider correctness, complete credential traceability, release readiness, or production readiness.
