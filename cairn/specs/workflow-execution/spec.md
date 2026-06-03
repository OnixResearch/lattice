# Workflow Execution Specification

## Purpose

Defines the `workflow-execution` capability.

## Requirements

### Requirement: Shell pipeline stdin propagation
r[n8nclone.workflow_execution.pipeline_stdin.stdout_to_stdin] The system MUST pass a connected shell node output value into a downstream shell node's stdin input.

#### Scenario: Stdout reaches downstream stdin
r[n8nclone.workflow_execution.pipeline_stdin.stdout_to_stdin.connected]
- GIVEN a workflow connects one shell node's stdout port to another shell node's stdin port
- WHEN the workflow runs
- THEN the downstream shell command receives the upstream stdout bytes on standard input

### Requirement: Real graph output port values
r[n8nclone.workflow_execution.pipeline_stdin.real_port_outputs] The system MUST store completed shell outputs under the shell node's real graph output port IDs.

#### Scenario: Completed shell outputs use real ports
r[n8nclone.workflow_execution.pipeline_stdin.real_port_outputs.completed]
- GIVEN a shell node has stdout, stderr, and exit output ports
- WHEN that shell node completes
- THEN the completed output map uses those real port IDs for stdout, stderr, and exit values

### Requirement: Cache output port identity
r[n8nclone.workflow_execution.pipeline_stdin.cache_real_ports] The system SHOULD treat cached shell outputs as values keyed by real graph output port IDs.

#### Scenario: Cached outputs remain routable
r[n8nclone.workflow_execution.pipeline_stdin.cache_real_ports.routable]
- GIVEN a cached shell output is used during a workflow run
- WHEN a downstream edge reads the cached upstream output
- THEN the downstream input lookup uses the same real output port ID as non-cached execution

### Requirement: Shell cache keys include executable configuration
r[n8nclone.workflow_execution.cache_key.shell_config] The system MUST include shell node executable configuration in the cache key used to reuse completed shell outputs.

#### Scenario: Command edits do not reuse stale output
r[n8nclone.workflow_execution.cache_key.shell_config.command_edit]
- GIVEN a shell node previously cached output for one command
- WHEN the same node inputs are run with a different command
- THEN the executor runs the new command instead of reusing the previous command's cached output

### Requirement: Headless workflow execution
r[n8nclone.workflow_execution.headless.run] The system MUST let operators run a saved workflow without starting the TUI.

#### Scenario: Run saved workflow
r[n8nclone.workflow_execution.headless.run.saved]
- GIVEN a workflow exists in the configured store
- WHEN the operator runs it through the headless command
- THEN the system loads the workflow, executes its nodes, and reports the completed run

#### Scenario: Missing workflow
r[n8nclone.workflow_execution.headless.run.missing]
- GIVEN no saved workflow matches the requested name
- WHEN the operator runs it through the headless command
- THEN the system reports a clear load error without starting the TUI

### Requirement: Headless exit status
r[n8nclone.workflow_execution.headless.exit_status] The system MUST return a deterministic process result for headless workflow runs.

#### Scenario: Successful workflow exits successfully
r[n8nclone.workflow_execution.headless.exit_status.success]
- GIVEN every node in the workflow completes successfully
- WHEN the headless run finishes
- THEN the command exits successfully

#### Scenario: Failed workflow exits with failure
r[n8nclone.workflow_execution.headless.exit_status.failure]
- GIVEN at least one node in the workflow fails
- WHEN the headless run finishes
- THEN the command exits with failure and identifies the failed node

### Requirement: Headless run summary
r[n8nclone.workflow_execution.headless.summary] The system SHOULD expose headless run summaries in human-readable and machine-readable forms.

#### Scenario: Text summary
r[n8nclone.workflow_execution.headless.summary.text]
- GIVEN a headless run reaches a terminal state
- WHEN the operator requests default output
- THEN the system prints a concise text summary with run status, node count, failures, and cache hits

#### Scenario: JSON summary
r[n8nclone.workflow_execution.headless.summary.json]
- GIVEN a headless run reaches a terminal state
- WHEN the operator requests JSON output
- THEN the system prints a structured summary that automation can parse deterministically

### Requirement: Headless run history
r[n8nclone.workflow_execution.headless.history] The system SHOULD persist headless run records using the same history model as TUI runs.

#### Scenario: Persist headless run
r[n8nclone.workflow_execution.headless.history.persist]
- GIVEN a headless run reaches a terminal state
- WHEN persistence is available
- THEN the system stores a run record that appears in recent run history

### Requirement: Headless error handling
r[n8nclone.workflow_execution.headless.errors] The system MUST handle headless load and execution errors without corrupting persisted workflow state.

#### Scenario: Load failure preserves store
r[n8nclone.workflow_execution.headless.errors.load_failure]
- GIVEN the requested workflow cannot be loaded
- WHEN the headless command fails
- THEN the system leaves saved workflows unchanged and reports the error
