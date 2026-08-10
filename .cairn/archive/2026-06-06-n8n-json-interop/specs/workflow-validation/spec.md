## ADDED Requirements

### Requirement: n8n JSON interoperability diagnostics
r[n8nclone.workflow_validation.n8n_json.unsupported] The system MUST report unsupported n8n import features with stable diagnostics before saving imported workflows.

#### Scenario: Unsupported n8n node kind
r[n8nclone.workflow_validation.n8n_json.unsupported.node_kind]
- GIVEN an n8n JSON workflow contains a node kind with no supported n8nclone mapping
- WHEN import validation runs
- THEN the system reports the unsupported node kind with the source node identity

#### Scenario: Unsupported n8n expression
r[n8nclone.workflow_validation.n8n_json.unsupported.expression]
- GIVEN an n8n JSON workflow contains an expression that cannot be translated to n8nclone expression syntax
- WHEN import validation runs
- THEN the system reports the unsupported expression and does not execute it

#### Scenario: Malformed n8n connection
r[n8nclone.workflow_validation.n8n_json.unsupported.connection]
- GIVEN an n8n JSON workflow references a missing source or target node in a connection
- WHEN import validation runs
- THEN the system reports a stable connection diagnostic and preserves existing saved workflows

### Requirement: n8n JSON export limits
r[n8nclone.workflow_validation.n8n_json.export_limits] The system MUST report unsupported export features before writing n8n JSON.

#### Scenario: Unsupported export node
r[n8nclone.workflow_validation.n8n_json.export_limits.node]
- GIVEN a n8nclone workflow contains a node with no known n8n export mapping
- WHEN export validation runs
- THEN the system reports the unsupported node and refuses default export
