## ADDED Requirements

### Requirement: Webhook cookbook example
r[n8nclone.workflow_format.webhook_cookbook_example] The system MUST provide a runnable Nickel cookbook workflow that demonstrates a webhook-triggered workflow using the persisted workflow format.

#### Scenario: Import cookbook workflow
r[n8nclone.workflow_format.webhook_cookbook_example.import]
- GIVEN the webhook cookbook Nickel workflow from the examples tree
- WHEN the operator imports the workflow into an empty configured store
- THEN the workflow imports successfully, validates successfully, and exposes its webhook trigger definition for listener dry-run

#### Scenario: Cookbook drift is rejected
r[n8nclone.workflow_format.webhook_cookbook_example.drift]
- GIVEN the webhook cookbook Nickel workflow is changed into malformed or unsupported workflow input
- WHEN automated cookbook validation runs
- THEN the validation fails with a deterministic diagnostic instead of silently accepting the drift
