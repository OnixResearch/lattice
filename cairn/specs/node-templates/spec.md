# Node Templates Specification

## Purpose

Defines the `node-templates` capability.

## Requirements

### Requirement: Template parameter schema
r[n8nclone.node_templates.parameters.schema] The system MUST let Nickel node templates declare typed parameters with defaults and validation constraints.

#### Scenario: Load parameterized template
r[n8nclone.node_templates.parameters.schema.load]
- GIVEN a Nickel node template declares parameters
- WHEN the template loader evaluates the template
- THEN the system loads parameter names, types, descriptions, defaults, and required markers

#### Scenario: Reject invalid parameter schema
r[n8nclone.node_templates.parameters.schema.invalid]
- GIVEN a Nickel node template declares an unsupported parameter type
- WHEN the template loader evaluates the template
- THEN the system rejects the template and reports a template diagnostic

### Requirement: Template parameter rendering
r[n8nclone.node_templates.parameters.render] The system MUST render parameterized templates only from validated parameter values.

#### Scenario: Render command from parameters
r[n8nclone.node_templates.parameters.render.command]
- GIVEN a template command references declared parameters
- WHEN the user provides valid parameter values
- THEN the system creates a node whose command and configuration contain the rendered values

#### Scenario: Missing required parameter
r[n8nclone.node_templates.parameters.render.missing]
- GIVEN a required template parameter has no value and no default
- WHEN the user confirms node creation
- THEN the system rejects creation and reports the missing parameter

### Requirement: Template parameter prompts
r[n8nclone.node_templates.parameters.prompt] The system SHOULD collect template parameter values during the add-node flow.

#### Scenario: Prompt for required parameter
r[n8nclone.node_templates.parameters.prompt.required]
- GIVEN the user selects a parameterized template
- WHEN the add-node flow opens the parameter step
- THEN the system prompts for each required parameter before creating the node

#### Scenario: Cancel parameter entry
r[n8nclone.node_templates.parameters.prompt.cancel]
- GIVEN parameter entry is in progress
- WHEN the user cancels the add-node flow
- THEN the system creates no node and preserves the existing workflow

### Requirement: Template parameter invalid cases
r[n8nclone.node_templates.parameters.invalid] The system MUST verify invalid parameter inputs fail without creating partial nodes.

#### Scenario: Invalid typed parameter
r[n8nclone.node_templates.parameters.invalid.type]
- GIVEN a parameter expects a number or boolean value
- WHEN the user enters a value that cannot be parsed as that type
- THEN the system rejects the value and keeps the add-node flow recoverable
