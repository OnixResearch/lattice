## Why

Workflows need credentials for shell commands, HTTP requests, and future integrations, but plaintext secrets must not become part of workflow files, logs, run history, or cache keys. A provider interface lets the product support multiple secret backends without coupling the workflow core to any one storage mechanism.

## What Changes

- Add opaque secret references to workflow/node configuration.
- Resolve secret references through a credential-provider interface at execution boundaries.
- Redact secret material from TUI, CLI output, logs, history, and summaries.
- Add positive and negative tests for resolution, missing secrets, and redaction.
