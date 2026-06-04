# Workflow Format Specification

## Purpose

Defines the `workflow-format` capability.

## Requirements

### Requirement: Nickel workflow export
r[n8nclone.workflow_format.nickel.export] The system MUST export saved workflows as typed Nickel workflow files.

#### Scenario: Export saved workflow
r[n8nclone.workflow_format.nickel.export.saved]
- GIVEN a workflow exists in the configured store
- WHEN the operator exports it to Nickel
- THEN the system writes a Nickel file containing workflow metadata, graph topology, ports, and node configuration

#### Scenario: Export missing workflow
r[n8nclone.workflow_format.nickel.export.missing]
- GIVEN no saved workflow matches the requested name
- WHEN the operator requests export
- THEN the system reports a clear error and writes no workflow file

### Requirement: Nickel workflow import
r[n8nclone.workflow_format.nickel.import] The system MUST import typed Nickel workflow files only after successful validation.

#### Scenario: Import valid workflow
r[n8nclone.workflow_format.nickel.import.valid]
- GIVEN a Nickel workflow file satisfies the workflow contract
- WHEN the operator imports it
- THEN the system saves the resulting workflow and makes it available to load or run

#### Scenario: Reject invalid workflow
r[n8nclone.workflow_format.nickel.import.invalid]
- GIVEN a Nickel workflow file is malformed or violates the workflow contract
- WHEN the operator imports it
- THEN the system rejects the import and preserves existing saved workflows

### Requirement: Nickel workflow round trip
r[n8nclone.workflow_format.nickel.round_trip] The system SHOULD preserve workflow meaning across Nickel export and import.

#### Scenario: Export then import preserves workflow
r[n8nclone.workflow_format.nickel.round_trip.preserve]
- GIVEN a saved workflow contains nodes, edges, ports, metadata, and shell configuration
- WHEN the workflow is exported to Nickel and imported again
- THEN the imported workflow has equivalent runnable topology and configuration

### Requirement: Complex workflow examples
r[n8nclone.workflow_examples.complex] The system MUST keep complex Nickel workflow examples importable and runnable.

#### Scenario: Run complex workflow examples headlessly
r[n8nclone.workflow_examples.headless]
- GIVEN complex Nickel workflow examples exist in the repository
- WHEN each example is imported and run headlessly
- THEN the system records successful runs with every node executed

### Requirement: Invalid workflow fixtures
r[n8nclone.workflow_format.nickel.invalid] The system MUST cover invalid Nickel workflow inputs with deterministic diagnostics.

#### Scenario: Missing required field
r[n8nclone.workflow_format.nickel.invalid.missing_field]
- GIVEN a Nickel workflow file omits a required workflow field
- WHEN validation runs
- THEN the system reports the missing field and refuses to save the workflow
