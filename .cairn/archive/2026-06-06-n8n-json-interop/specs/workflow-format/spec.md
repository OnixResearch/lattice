## ADDED Requirements

### Requirement: n8n JSON import
r[n8nclone.workflow_format.n8n_json.import] The system MUST import a documented subset of n8n workflow JSON into typed n8nclone workflows with deterministic mapping reports.

#### Scenario: Import supported n8n workflow
r[n8nclone.workflow_format.n8n_json.import.supported]
- GIVEN an n8n JSON workflow uses only supported node kinds and connection shapes
- WHEN the operator imports it
- THEN the system creates an equivalent n8nclone workflow and writes a mapping report for every source node

#### Scenario: Import malformed n8n JSON fails
r[n8nclone.workflow_format.n8n_json.import.malformed]
- GIVEN an n8n JSON file is syntactically invalid or missing required workflow fields
- WHEN import runs
- THEN the system rejects the file with deterministic diagnostics and preserves existing saved workflows

### Requirement: n8n JSON import CLI
r[n8nclone.workflow_format.n8n_json.import_cli] The system SHOULD expose n8n JSON import through an operator-facing command that separates workflow creation from mapping evidence.

#### Scenario: Import writes mapping report
r[n8nclone.workflow_format.n8n_json.import_cli.report]
- GIVEN an n8n JSON import succeeds or fails with supported diagnostics
- WHEN the operator requests a mapping report
- THEN the system writes stable report data containing source ids, target ids when created, statuses, diagnostics, and unsupported fields

### Requirement: n8n JSON fixtures
r[n8nclone.workflow_format.n8n_json.fixtures] The system MUST keep positive and negative n8n JSON interoperability fixtures in the repository.

#### Scenario: Supported fixtures import
r[n8nclone.workflow_format.n8n_json.fixtures.supported]
- GIVEN supported n8n JSON fixtures exist for shell and HTTP workflows
- WHEN the fixture tests run
- THEN each fixture imports into an equivalent typed n8nclone workflow

#### Scenario: Unsupported fixtures fail predictably
r[n8nclone.workflow_format.n8n_json.fixtures.unsupported]
- GIVEN negative n8n JSON fixtures exist for unsupported nodes, expressions, malformed connections, and credential data
- WHEN the fixture tests run
- THEN each fixture reports the expected deterministic diagnostic

### Requirement: n8n JSON export
r[n8nclone.workflow_format.n8n_json.export] The system MAY export supported n8nclone workflow subsets as deterministic n8n JSON.

#### Scenario: Export supported subset
r[n8nclone.workflow_format.n8n_json.export.supported]
- GIVEN an n8nclone workflow contains only node kinds and settings with known n8n mappings
- WHEN the operator exports it as n8n JSON
- THEN the system writes deterministic n8n JSON and a mapping report describing exported semantics

#### Scenario: Export unsupported subset is blocked
r[n8nclone.workflow_format.n8n_json.export.unsupported]
- GIVEN an n8nclone workflow contains nodes or settings without known n8n mappings
- WHEN the operator exports it as n8n JSON
- THEN the system reports unsupported features and does not write misleading workflow JSON unless explicit best-effort export is requested
