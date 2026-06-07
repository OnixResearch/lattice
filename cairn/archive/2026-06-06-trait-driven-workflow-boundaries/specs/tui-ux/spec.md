## ADDED Requirements

### Requirement: Node-kind provider property metadata
r[n8nclone.tui_ux.node_kind_provider] The system SHOULD render and edit node-specific TUI metadata through node-kind providers instead of scattered presentation-only `NodeConfig` matches.

#### Scenario: Provider-backed property panel
r[n8nclone.tui_ux.node_kind_provider.property_metadata]
- GIVEN a selected node has a registered node-kind provider
- WHEN the TUI renders the properties panel or opens a supported property editor
- THEN the system SHOULD use provider-supplied labels, editable fields, display values, and validation hints while preserving current built-in node behavior

#### Scenario: Unsupported field edit remains recoverable
r[n8nclone.tui_ux.node_kind_provider.unsupported_edit]
- GIVEN a selected node does not support a requested editable field through its provider
- WHEN the user attempts to edit that field
- THEN the system MUST reject the edit without mutating workflow configuration and keep the user in a recoverable TUI state

#### Scenario: Provider fallback preserves display
r[n8nclone.tui_ux.node_kind_provider.fallback]
- GIVEN a built-in node kind has not yet migrated every display path to provider metadata
- WHEN the TUI renders that node during the migration
- THEN the system MUST keep the existing display branch active as a guarded fallback and MUST NOT hide existing node details
