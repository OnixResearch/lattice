## Context

`RunHistoryStore` already exposes exact `RunRecord`, optional `RunDetailRecord`, and optional replay events. Orchestration-owned records carry an immutable `RunOwner::Orchestration { instance_id, dispatch_id }`, while the instance event chain contains dispatch admission, acknowledgement, observed completion, budget charge, cancellation, and abandonment facts. The TUI can render full persisted details, but there is no bounded headless inspection surface.

The operator command must not turn persistence into a new content-exfiltration interface. Even values that are not credential-tagged may legitimately contain prompt echoes or provider output. Therefore the safe CLI report cannot serialize durable detail or event records directly.

## Portfolio

### Family: Serialize existing store records

**State:** Rejected.

This is the shortest implementation, but `RunDetailRecord` contains output and error text, replay events contain summaries, and orchestration instances contain objectives, trigger keys, actors, and terminal summaries. Redacting only known credentials would not exclude prompt content or unclassified private text.

### Family: Reuse the TUI run-history renderer

**State:** Rejected.

The TUI intentionally renders output and error detail for an interactive local operator and is coupled to selection state and replay frames. Reusing it would invert the shell boundary and preserve the wrong disclosure contract.

### Family: Join existing orchestration status and replay commands

**State:** Rejected.

This leaves run details fragmented across commands, does not verify the exact run-owner tuple in one report, and still exposes the broader orchestration instance JSON shape.

### Family: Pure safe projection plus thin store shell

**State:** Selected.

A pure core accepts already-loaded run, detail, replay, orchestration-instance, and orchestration-event records. It emits only bounded identifiers, numeric lifecycle facts, enum states, counts, budget dimensions, and diagnostic codes. A thin shell performs exact store reads. The CLI renders text or JSON from the same safe report.

## Decisions

### Decision: Omit arbitrary content instead of attempting semantic redaction

**Choice:** Exclude node labels, port labels, output text, error text, replay summaries, orchestration summaries, objectives, actors, and trigger idempotency keys. Report node identity, terminal status, cache state, output count, and error presence only.

**Rationale:** Prompt content is semantic and cannot be reliably recognized by substring filtering. Structural omission gives the command a testable disclosure boundary independent of provider behavior.

### Decision: Validate exact orchestration ownership

**Choice:** An orchestration-owned run is valid only when exactly one `StartRound` fact matches the record's instance ID, dispatch ID, and run ID, the complete orchestration chain replays to the persisted instance, and subsequent dispatch facts remain consistent with that tuple.

**Rationale:** Instance-only or run-only matching could attach foreign budget and lifecycle evidence to the inspected run.

### Decision: Distinguish invalid evidence from historical absence

**Choice:** Missing detail or replay records are reported as `not_recorded` and do not by themselves invalidate a legacy run. A present record with mismatched identity, foreign owner binding, wrong run IDs, fatal replay diagnostics, or an invalid orchestration chain makes the report invalid and the CLI exit unsuccessfully after rendering it.

**Rationale:** Operators need truthful visibility into old stores without treating missing evidence as verified. Present contradictory evidence must fail closed.

### Decision: Keep unobserved dispatches on the orchestration surface

**Choice:** `runs show` requires a persisted `RunRecord`. An abandoned or pending-unobserved dispatch with no run record remains inspectable through `orchestration status|recover|replay` using its instance identity.

**Rationale:** Synthesizing a run record for an unknown remote outcome would falsely claim a locally observed run. A global event scan by run ID would be unbounded and ambiguous without a dedicated index.

## Risks / Trade-offs

- The command intentionally cannot recover arbitrary output or error text; the local TUI remains the detailed inspection surface.
- Historical runs may report details or replay as `not_recorded`.
- Per-run budget evidence is limited to the exact dispatch facts: planned upper bound, observed charge when recorded, and settlement state. It does not claim provider billing or token truth.
- Replay validity proves local record consistency only, not external anchoring or event completeness.
- Unobserved abandoned dispatches remain outside `runs show` because they are not persisted runs.
