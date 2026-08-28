## Context

Lattice shell nodes intentionally expose exit codes as typed output values. The general executor can therefore complete a node while preserving a nonzero exit code. Workflow exchange currently reads only the persisted node status and promotes every completed node to `DurableNodeResult::Completed`.

The exchange boundary has stronger semantics: its terminal success is consumed as an external provider result. It must inspect the typed exit-code port before it creates that result.

## Decisions

### Decision: Classify exit status only at the workflow-exchange boundary

**Choice:** Keep the process shell and general workflow executor unchanged. In the workflow-exchange worker, locate the node's typed `ExitCode` output port. Accept completion only when exactly one persisted output for that port has the canonical `exit code: 0` text. Classify nonzero, missing, duplicate, or malformed exit output as failure. Nodes without an exit-code port retain their normal completed classification.

**Rationale:** This preserves existing workflows that use exit codes as data while preventing workflow exchange from promoting a failed provider process. Using the graph's type tag avoids relying on a user-editable port label.

### Decision: Test success and failure through the real handler

**Choice:** Keep the existing successful shell fixture and add a handler-level negative test with a nonzero shell exit.

**Rationale:** The test covers dispatch, the headless runtime, persisted run details, completion projection, and final observation. A helper-only test would not catch integration drift.

## Risks / Trade-offs

- Persisted run details currently store rendered exit text, not the typed scalar. The check therefore admits only the canonical success rendering and fails closed for every other value.
- This change proves local workflow-exchange classification. It does not prove provider correctness, process isolation, external effect truth, or release eligibility.
