## ADDED Requirements

### Requirement: Webhook listener process
r[n8nclone.scheduling.webhook_listener.process] The system MUST provide an operator-started webhook listener that receives HTTP requests for persisted webhook triggers.

#### Scenario: Listener starts from store configuration
r[n8nclone.scheduling.webhook_listener.process.start]
- GIVEN persisted webhook triggers exist in the configured store
- WHEN the operator starts the webhook listener command
- THEN the listener loads enabled trigger definitions and begins accepting HTTP requests on the configured bind address

#### Scenario: Listener rejects absent webhook triggers
r[n8nclone.scheduling.webhook_listener.process.missing]
- GIVEN no enabled webhook trigger matches an inbound request path
- WHEN the listener receives the request
- THEN the listener returns a deterministic not-found response and records no workflow run

### Requirement: Webhook request planning
r[n8nclone.scheduling.webhook_listener.planning] The system MUST translate HTTP requests into trigger events through a pure deterministic planning core before network or workflow side effects occur.

#### Scenario: Request facts are normalized
r[n8nclone.scheduling.webhook_listener.planning.normalize]
- GIVEN an inbound HTTP request has method, path, query, headers, content type, and body bytes
- WHEN webhook request planning runs
- THEN the planner returns trigger event facts with normalized headers and parsed JSON only when the content type declares JSON

#### Scenario: Body parsing failure is deterministic
r[n8nclone.scheduling.webhook_listener.planning.bad_body]
- GIVEN an inbound webhook request declares JSON content but contains malformed JSON
- WHEN webhook request planning runs
- THEN the planner returns a deterministic request diagnostic without attempting trigger admission

### Requirement: Webhook listener admission shell
r[n8nclone.scheduling.webhook_listener.admission] The system MUST admit listener requests through persisted trigger definitions, credential providers, idempotency records, and overlap state before starting workflow execution.

#### Scenario: Valid listener request starts workflow
r[n8nclone.scheduling.webhook_listener.admission.valid]
- GIVEN an enabled webhook trigger matches the request and authentication succeeds
- WHEN the listener admits the request
- THEN the system starts the configured workflow through the headless trigger plan adapter and records an accepted trigger history entry

#### Scenario: Duplicate listener request is rejected
r[n8nclone.scheduling.webhook_listener.admission.duplicate]
- GIVEN a webhook trigger has already accepted an idempotency key
- WHEN the listener receives another request with the same key
- THEN the listener rejects the request before workflow execution and records a rejected trigger history entry

#### Scenario: Listener authentication remains redacted
r[n8nclone.scheduling.webhook_listener.admission.redaction]
- GIVEN a webhook request fails bearer or signature authentication
- WHEN the listener reports the rejection
- THEN response bodies, logs, diagnostics, and trigger history omit plaintext secret material and raw sensitive payload data

### Requirement: Webhook response mapping
r[n8nclone.scheduling.webhook_listener.responses] The system MUST map listener outcomes to stable HTTP responses that reveal no secret material.

#### Scenario: Accepted request response
r[n8nclone.scheduling.webhook_listener.responses.accepted]
- GIVEN a listener request is admitted and a workflow run is started
- WHEN the listener sends the response
- THEN the response identifies the trigger and run outcome without including raw payload or credential material

#### Scenario: Rejected request response
r[n8nclone.scheduling.webhook_listener.responses.rejected]
- GIVEN a listener request is rejected by planning, matching, authentication, idempotency, or payload binding
- WHEN the listener sends the response
- THEN the response contains a stable rejection code and safe diagnostic message without starting a workflow
