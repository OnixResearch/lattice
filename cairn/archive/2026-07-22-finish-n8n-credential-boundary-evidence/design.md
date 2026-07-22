# Design: finish n8n credential boundary evidence

## Goal and completion evidence

The change is complete when direct maintained assertions establish every clause of these four rows while current coverage remains exactly 340 missing with zero dangling markers:

- `n8nclone.credentials.n8n_json.placeholders.export`
- `n8nclone.credentials.n8n_json.placeholders.plaintext`
- `n8nclone.credentials.n8n_json.placeholders.reference`
- `n8nclone.credentials.n8n_parity_secret_safety`

The inventory must contain exactly four new exact decisions, retain nine partial decisions, and add no selected credential marker.

## Functional core and imperative shell

Pure fixture construction and in-memory import/export observations remain separate from command/store orchestration. CLI tests create bounded temporary directories, invoke one command, then inspect only explicit output, report, and configured store paths.

The imported execution fixture uses an `executeCommand` node with opaque credential metadata and a command that tests only whether the imported environment binding is non-empty. A configured map provider supplies the execution-only value. A second execution with no provider value must fail before the shell command runs.

The export observation executes a reference-bearing shell node through a provider, saves the unchanged workflow, invokes the n8n CLI exporter, and checks the output identity plus plaintext omission from output, report, and raw store bytes.

## Parity contract correction

The parent parity requirement currently says all fixtures are plaintext-free while its child scenario requires a plaintext rejection fixture. The parent is clarified so supported fixtures, checked-in reports, rendered diagnostics, and test output remain plaintext-free while one designated unsupported fixture may contain one bounded sentinel solely as validation input.

## Portfolio audit

- **Implementation-shape inference:** rejected because save/export and execution boundaries require observations.
- **Core-only import tests:** rejected because they cannot prove CLI store non-creation or command output safety.
- **Removing the plaintext negative fixture:** rejected because it would delete the required negative case.
- **Bounded exception plus multi-boundary tests:** selected because it preserves rejection coverage while keeping every durable/operator-visible output secret-free.

## Fail-closed checks

Positive tests cover provider success, opaque export identity, store non-creation, report guidance, and parity omission. Negative tests cover missing provider values, plaintext import rejection, bounded sentinel multiplicity, and completion-policy drift. A dedicated completion validator rejects row omission, reclassification, evidence drift, selected markers, missing-set drift, dangling markers, and successful global coverage.

## Claim boundaries

Exact classification records maintained evidence for the accepted clauses only. It does not establish arbitrary n8n semantic parity, provider truth, remote effects, complete traceability, release readiness, or production readiness.
