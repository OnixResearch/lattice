# Design: Run replay debugger

## Architecture

Build replay on top of unified runner events rather than introducing a second execution model.

```text
Runner events + value summaries + artifact refs
        │
        ▼
ReplayCore ─ pure timeline reconstruction and cursor movement
        │
        ▼
ReplayShell ─ load history, resolve safe artifacts, render/export/import
```

`ReplayCore` accepts immutable event records, value summaries, and optional workflow snapshots. It returns deterministic timeline frames, cursor transitions, node status maps, and diagnostics. It performs no store, clock, process, network, or UI I/O.

## Event persistence

Persist runner events with run id, sequence, event kind, node id, optional port id, parent run correlation, timestamp supplied by shell, safe summary, redaction metadata, and optional artifact reference. The event store should be append-only per run and should tolerate partial runs.

## Timeline reconstruction

Replay reconstructs node states by folding events in sequence order. The fold rejects out-of-order duplicate sequences, impossible terminal-state mutation, unknown node ids when a workflow snapshot is present, and malformed parent/child correlation. The UI can move a cursor through frames without changing persisted run data.

## Replay bundles

A replay bundle contains metadata, workflow snapshot or hash, event records, value summaries, artifact reference metadata, and redaction policy. It excludes plaintext secrets and large artifact bytes by default. Optional artifact inclusion must preserve content hashes and media metadata.

## Debugger UX

The TUI should support a timeline pane, selected-event details, node status at cursor, value summaries for selected outputs, cache/executed markers, and subworkflow drill-down. Active runs can stream into the same model as historical runs.

## Testing

Use pure tests for event folding, cursor movement, partial event streams, duplicate/out-of-order event rejection, terminal-state mutation rejection, and subworkflow nesting. Use fixture replay bundles for import/export positive and negative coverage.
