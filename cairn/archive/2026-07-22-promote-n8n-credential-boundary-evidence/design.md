# Design: promote exact n8n credential boundary evidence

## Goal and completion evidence

The promotion is complete only when deterministic checks prove all of the following:

- the baseline receipt has exactly 340 missing IDs;
- the current receipt has exactly 336 missing IDs;
- baseline minus current is exactly the four-ID reviewed allowlist;
- current minus baseline is empty;
- both receipts have zero dangling markers and remain `valid: false`;
- every marker is attached directly to its exact supporting test; and
- current debt policy contains none of the promoted IDs.

## Selected mechanism

Bind the reference marker to the configured-provider execution test, the export marker to the save/raw-store/CLI-export test, and the plaintext marker to the rejected CLI-import/no-store test. Bind the parity marker to both the rejected CLI-output test and the bounded fixture/report/output test because the parent requirement spans supported and unsupported surfaces.

Reuse the pure promotion validator for deterministic missing-set comparison and local marker binding. The test shell owns only checked-in receipt and source inclusion.

## Portfolio audit

- **Implementation-name inference:** rejected because source shape does not establish execution, persistence, export, or operator-output observations.
- **Completion-test binding:** rejected because lifecycle classification tests are indirect evidence.
- **One marker per file:** rejected because file scope cannot establish exact assertion ownership.
- **Exact boundary-test binding:** selected because each marker remains adjacent to maintained assertions for every accepted clause.

## Fail-closed validation

Negative mutations introduce duplicate baseline/current IDs, an unexpected removal, a re-added promoted ID, an unaccepted addition, a dangling marker, duplicate allowlist entry, missing local test binding, stale reviewed policy rows, and successful coverage. Deterministic checks must reject every mutation.

## Claim boundaries

The promotion proves only an exact maintained linkage transition. It does not prove arbitrary n8n semantic parity, provider truth, arbitrary secret confidentiality, complete credential traceability, release readiness, or production readiness.
