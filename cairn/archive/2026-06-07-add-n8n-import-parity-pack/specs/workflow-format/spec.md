## ADDED Requirements

### Requirement: n8n parity fixture pack
r[n8nclone.workflow_format.n8n_parity_fixture_pack] The system MUST maintain a curated n8n JSON fixture pack that proves supported imports and predictable unsupported-feature rejection for common n8n workflow shapes.

#### Scenario: Supported parity fixtures import
r[n8nclone.workflow_format.n8n_parity_fixture_pack.supported]
- GIVEN a supported n8n JSON fixture from the parity pack
- WHEN the operator imports the fixture
- THEN the import succeeds, the resulting workflow validates, and the mapping report records the supported node and connection mappings

#### Scenario: Unsupported parity fixtures fail predictably
r[n8nclone.workflow_format.n8n_parity_fixture_pack.unsupported]
- GIVEN an unsupported n8n JSON fixture from the parity pack
- WHEN the operator imports the fixture
- THEN the import fails with a deterministic diagnostic and no partially accepted workflow is persisted

#### Scenario: Parity fixture documentation stays aligned
r[n8nclone.workflow_format.n8n_parity_fixture_pack.docs]
- GIVEN the parity fixture pack is checked by automation
- WHEN supported or unsupported fixture coverage changes
- THEN the documented compatibility subset and fixture metadata remain aligned with the tested outcomes

### Requirement: n8n parity mapping reports
r[n8nclone.workflow_format.n8n_parity_mapping_reports] The system MUST produce deterministic mapping reports for parity fixtures that identify supported mappings, warnings, and unsupported feature classes without leaking secret material.

#### Scenario: Supported mapping report is stable
r[n8nclone.workflow_format.n8n_parity_mapping_reports.supported]
- GIVEN a supported n8n JSON fixture imports successfully
- WHEN the mapping report is generated
- THEN the report deterministically records imported node ids, node-kind mappings, connection mappings, credential placeholders, and warnings

#### Scenario: Unsupported mapping report is stable
r[n8nclone.workflow_format.n8n_parity_mapping_reports.unsupported]
- GIVEN an unsupported n8n JSON fixture is rejected
- WHEN the mapping report or diagnostic summary is generated
- THEN the report deterministically records the unsupported feature class and safe remediation hint
