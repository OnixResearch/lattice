## ADDED Requirements

### Requirement: Webhook cookbook operator flow
r[n8nclone.scheduling.webhook_cookbook_operator_flow] The system MUST document and test an end-to-end operator flow for webhook trigger dry-run, request admission, deterministic rejection, and run-history inspection.

#### Scenario: Cookbook valid request starts a run
r[n8nclone.scheduling.webhook_cookbook_operator_flow.accepted]
- GIVEN the webhook cookbook workflow has been imported and the listener has loaded its enabled trigger
- WHEN the cookbook sends a valid webhook request
- THEN exactly one workflow run is started and safe accepted trigger history is recorded

#### Scenario: Cookbook invalid request is rejected
r[n8nclone.scheduling.webhook_cookbook_operator_flow.rejected]
- GIVEN the webhook cookbook workflow has been imported and the listener has loaded its enabled trigger
- WHEN the cookbook sends a request with an invalid path, invalid authentication, malformed body, or duplicate idempotency key
- THEN the listener returns a deterministic rejection and records no new workflow run for that request

#### Scenario: Cookbook dry-run proves readiness
r[n8nclone.scheduling.webhook_cookbook_operator_flow.dry_run]
- GIVEN the webhook cookbook workflow has been imported into the configured store
- WHEN the operator runs listener dry-run for the cookbook
- THEN the command reports the loaded trigger count and safe readiness diagnostics without binding a network socket
