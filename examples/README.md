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

Import and run one with a temporary config/store:

```bash
n8nclone --config ./demo/config.ncl --init
n8nclone --config ./demo/config.ncl import examples/workflows/daily-digest.ncl
n8nclone --config ./demo/config.ncl run daily-digest --output json
```
