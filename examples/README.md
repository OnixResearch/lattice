# Examples

These examples are intentionally more complex than the default starter catalog.
They are useful for demos, import/export validation, and headless regression tests.

## Templates

`examples/templates/` contains advanced Nickel node templates:

- `complex-sort-lines.ncl` — sort piped text with optional sort flags.
- `complex-unique-lines.ncl` — collapse adjacent duplicate lines with optional `uniq` flags.
- `complex-cut-field.ncl` — extract a delimited field with bounded numeric field selection.

## Workflows

`examples/workflows/` contains runnable, workflow-shaped examples:

- `daily-digest.ncl` — filters event records and renders a Markdown digest.
- `csv-cleanup-report.ncl` — filters active CSV records, sorts/deduplicates them, and renders a team report.
- `log-triage.ncl` — filters warning/error log lines and renders an incident summary.
- `complex-text-normalization.ncl` — sorts, deduplicates, filters, replaces, and counts text lines.
- `selfie-to-fine-art.ncl` — routes a selfie plus inspiration images through a generic Clankers agent node.
- `webhook-cookbook.ncl` — imports a credential-reference authenticated webhook trigger for local listener demos.

Import and run one with a temporary config/store:

```bash
n8nclone --config ./demo/config.ncl --init
n8nclone --config ./demo/config.ncl import examples/workflows/daily-digest.ncl
n8nclone --config ./demo/config.ncl run daily-digest --output json
```

## Webhook cookbook

`webhook-cookbook.ncl` demonstrates the webhook listener path without storing plaintext secret material in the workflow. The workflow references `env:N8NCLONE_WEBHOOK_TOKEN`; set that environment variable from your secret manager before dry-run or listener startup.

```bash
n8nclone --config ./demo/config.ncl --init
n8nclone --config ./demo/config.ncl import examples/workflows/webhook-cookbook.ncl
N8NCLONE_WEBHOOK_TOKEN="$WEBHOOK_TOKEN" n8nclone --config ./demo/config.ncl webhook-listen --dry-run --bind 127.0.0.1:8787
N8NCLONE_WEBHOOK_TOKEN="$WEBHOOK_TOKEN" n8nclone --config ./demo/config.ncl webhook-listen --bind 127.0.0.1:8787
```

Send an accepted request from another terminal by passing the same environment-backed token at runtime rather than writing it into the workflow:

```bash
curl -sS -X POST http://127.0.0.1:8787/hooks/cookbook-lead \
  -H 'Content-Type: application/json' \
  -H "Authorization: Bearer $WEBHOOK_TOKEN" \
  -H 'Idempotency-Key: cookbook-event-1' \
  --data '{"name":"Ada"}'
```

Rejected requests are deterministic and do not start a workflow run. For example, this missing path returns a safe not-found response:

```bash
curl -sS -X POST http://127.0.0.1:8787/hooks/missing \
  -H 'Content-Type: application/json' \
  -H "Authorization: Bearer $WEBHOOK_TOKEN" \
  -H 'Idempotency-Key: cookbook-event-2' \
  --data '{"name":"Ada"}'
```

Inspect trigger state and safe history after requests:

```bash
n8nclone --config ./demo/config.ncl triggers list
n8nclone --config ./demo/config.ncl triggers history
```
