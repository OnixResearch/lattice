## Context

The scheduler reserves a run identity, executes the workflow, persists terminal run evidence, and only then advances the schedule. The webhook listener plans from trigger history, executes the admitted workflow, and persists the accepted trigger record after the headless run returns. A process killed after a shell/file/HTTP effect but before those writes can therefore leave an observable effect with no terminal local record.

Existing process tests kill before the recovery effect and prove safe restart from that earlier boundary. Existing duplicate webhook coverage sends a second request only after the first request has returned and its trigger history is durable. Neither test reaches the post-effect/pre-persistence interval.

### Completion contract

The change is complete only when real scheduler and webhook processes cross a deterministic effect checkpoint, die before terminal persistence, and are inspected from an untouched crash snapshot before restart. The checks must then restart or redeliver, assert exact effect-attempt and durable-record observations against a version-controlled profile, and preserve explicit non-claims. Test count, child exit status, marker existence without durable-state inspection, or a test-only production hook is not completion evidence.

False completion includes killing before the effect, allowing the first process to persist terminal state, opening the original store before the restart shell repairs it, counting only successful terminal runs, accepting multiple unclassified outcomes, or describing at-least-once observations as exactly-once guarantees.

## Decisions

### Decision: Inject death through real workflow effects

**Choice:** Use deterministic shell workflows that append a uniquely identified effect checkpoint, flush the local file operation, signal the owning Lattice process, and terminate. Keep the mechanism in test fixtures and reuse the existing child-process guards and bounded waits.

**Rationale:** This crosses the real runtime and process boundary without adding a production-only fault branch. Unit-level executor errors cannot prove ordering relative to process death and durable store writes.

### Decision: Inspect copied crash state before restart

**Choice:** Copy the crashed redb file before any mutating reopen, inspect the copy for run, trigger, and schedule facts, and leave the original untouched for the restarted process.

**Rationale:** Redb may repair storage metadata when reopened after forced death. Inspecting the original first could accidentally perform part of recovery and invalidate the restart claim.

### Decision: Bind observations to a reviewed profile

**Choice:** Represent the expected ingress, checkpoint, pre-restart state, restart/redelivery action, effect-attempt count, and terminal evidence as deterministic facts evaluated by a pure classifier. The initial profile records the current duplicate-effect risk. A later semantic hardening change must update the webhook profile and retain the previous baseline in its archived evidence.

**Rationale:** The requirement is continuous characterization of a dangerous boundary, not preservation of accidental behavior forever. A checked profile makes changes visible while allowing later atomic admission to improve webhook behavior deliberately.

### Decision: Reject exactly-once language

**Choice:** Describe repeated execution as at-least-once risk and describe non-repetition only as a bounded Lattice admission observation. Do not claim external effect identity, provider deduplication, remote cancellation, or exactly-once execution.

**Rationale:** A marker and local database can establish local ordering and observed attempts only. They cannot prove the semantics of arbitrary external systems.

## Functional Core / Imperative Shell

The pure test core classifies explicit ingress, effect-count, response, schedule, run, and trigger-history facts against the reviewed profile. The imperative test shell creates temporary state, starts real binaries, waits for checkpoints, copies crash storage, sends signals or requests, restarts processes, and reads observations. Product runtime code receives no fault-injection hooks.

## Risks / Trade-offs

- Shell signal and PTY/TCP behavior bounds this evidence to supported Unix/Linux process semantics.
- An effect checkpoint proves the selected local write occurred; it does not prove arbitrary remote systems committed equivalent work.
- The initial webhook profile intentionally exposes a weakness that the dependent atomic-idempotency change is expected to improve.
- Forced process death can leave child processes; every fixture must retain an owned teardown guard and bounded wait.
