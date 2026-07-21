# Node Templates Specification Delta

## MODIFIED Requirements

### Requirement: Template parameter invalid cases
r[n8nclone.node_templates.parameters.invalid] The system MUST verify invalid parameter inputs fail without creating partial nodes.

#### Scenario: Invalid typed parameter
r[n8nclone.node_templates.parameters.invalid.type]
- GIVEN a parameter expects a number or boolean value
- WHEN the user enters a value that cannot be parsed as that type
- THEN the system rejects the value and keeps the add-node flow recoverable

#### Scenario: Numeric parameter exceeds its maximum
r[n8nclone.node_templates.parameters.invalid.bounds]
- GIVEN a numeric template parameter declares a maximum value
- WHEN the supplied value exceeds that maximum
- THEN the system MUST reject rendering with a maximum-specific error before producing a command
