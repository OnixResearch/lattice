# Design: n8n JSON interoperability

## Architecture

Keep n8n JSON as an external format with a pure mapper and a thin shell.

```text
n8n JSON
  │
  ▼
N8nImportCore ─ parse, normalize, map, diagnose
  │
  ▼
WorkflowFormatShell ─ file I/O, save workflow, render report
```

The core consumes JSON values and an explicit support table. It returns either a `PortableWorkflow` plus mapping report or deterministic diagnostics. The shell owns reading files, writing reports, saving workflows, and CLI output.

## Supported subset

Initial mappings should target primitives already present in n8nclone: shell/execute-command nodes, HTTP request nodes, file read/write nodes, simple set/transform values expressible as templates, schedule/manual triggers once trigger support exists, and webhook triggers once webhook support exists. Unsupported nodes produce diagnostics and can optionally become disabled placeholder nodes if an import policy requests best-effort import.

## Expressions

n8n expressions are not executed. The importer classifies expressions as literal-compatible, n8nclone-expression-compatible, or unsupported. Unsupported expressions remain diagnostics unless an explicit placeholder policy stores the original expression as non-executable metadata.

## Credentials

n8n credential references import as opaque credential reference placeholders. The importer rejects plaintext credential data and encrypted credential blobs unless a future explicit migration tool can prove safe provider mapping. Export writes references only.

## Mapping report

Every import/export produces a report with source node id, target node id when created, mapping status, diagnostics, unsupported fields, credential placeholders, and behavior changes. Reports are stable JSON so tests and operators can diff them.

## Testing

Use fixture-driven tests for positive supported workflows and negative unsupported nodes, malformed JSON, expression mismatches, duplicate ids, unknown ports, and plaintext credential attempts. Export tests compare deterministic JSON with sorted keys where practical.
