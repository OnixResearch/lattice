# n8n parity fixtures

This pack documents the tested n8n JSON compatibility subset.

## Supported fixtures

- `supported/command-pipeline.json` maps Execute Command nodes and a main connection into a n8nclone shell pipeline.
- `supported/http-credential-chain.json` maps an HTTP Request node with an opaque credential placeholder and a downstream Execute Command node.
- `supported/default-http-method.json` maps an HTTP Request node that relies on the default GET method.

Supported fixtures must import successfully, validate as n8nclone workflows, and produce deterministic mapping reports.

## Unsupported fixtures

- `unsupported/webhook-trigger.json` documents that n8n webhook trigger nodes are not imported as workflow nodes.
- `unsupported/schedule-trigger.json` documents that n8n schedule trigger nodes are not imported as workflow nodes.
- `unsupported/expression-url.json` documents deterministic rejection of unsupported n8n expression syntax.
- `unsupported/plaintext-node-credential.json` documents safe rejection of non-opaque credential material.
- `unsupported/malformed-connection.json` documents deterministic malformed connection diagnostics.

Unsupported fixtures must fail predictably without persisting partial workflows. Reports and diagnostics must not include credential payload values.
