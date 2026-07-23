## Context

`App` already models failed node status, diagnostics, active-run cancellation, run-history inspection, and explicit rerun. `input.rs` maps real key events to those actions, and panels use bounded rectangles for most overlays. Unit and rendered-buffer tests cover many individual transitions. The current operator PTY test proves only the successful path at one terminal size.

A full process test must avoid brittle byte-for-byte screen snapshots because alternate screen rendering includes cursor motion and ANSI control sequences. It must also distinguish an explicit fresh rerun from accidental automatic retry and must verify child-process cancellation rather than accepting a changed UI label alone.

### Completion contract

The change is complete only when a real Lattice TUI process in a PTY executes deterministic failure, validation-rejection, cancellation, rerun, and resize scenarios; observes semantic rendered surfaces after each action; exits through keyboard input; and verifies durable run/detail state plus absence of forbidden effects. Unit transitions, a rendered word without persisted state, killing the whole TUI instead of canceling the run, or resize without post-resize input is not completion evidence.

False completion includes automatic retry, mutation of the historical failed record, diagnostics that still execute the invalid workflow, cancellation that leaves the bounded child alive or permits a pending effect, checking only the final terminal size, matching raw ANSI frames exactly, or adding production-only test controls.

## Decisions

### Decision: Use one fail-once workflow for failure and rerun

**Choice:** Seed a shell workflow whose first run records a deterministic attempt and fails, while a later explicit run-history rerun observes the marker and succeeds. Drive open, run, history, inspect, rerun, and inspect through PTY key input.

**Rationale:** Distinct first and second outcomes prove a fresh explicit run occurred. The final store can show two immutable run identities and preserve failed details without relying only on transient text.

### Decision: Test validation rejection separately

**Choice:** Seed an invalid workflow through a controlled store fixture, invoke run from the PTY, open the diagnostics overlay, and assert that no effect marker or run record was created before continuing navigation.

**Rationale:** Runtime failure and pre-effect validation rejection have different safety obligations. Combining them could hide workflow execution behind a generic error word.

### Decision: Observe cancellation at both UI and process boundaries

**Choice:** Run a bounded long-lived shell node that records readiness and has a later forbidden effect. Send the real cancel shortcut only after readiness, assert the terminal cancellation surface, verify the forbidden effect never appears, and use an owned child/process-group observation when available.

**Rationale:** A status transition alone does not prove the child stopped. The check must show pending external work did not continue after TUI cancellation.

### Decision: Resize through the PTY master

**Choice:** Use `portable-pty` resize operations to shrink below preferred overlay dimensions and restore the terminal while a workspace or modal is active. After each resize, wait for semantic surfaces and send navigation input.

**Rationale:** Directly calling `Event::Resize` tests only app state. Resizing the PTY exercises terminal event delivery, layout, rendering, and continued input handling together.

### Decision: Assert semantic surfaces and durable facts

**Choice:** Search the accumulated transcript for unique phase tokens and panel titles, then verify run records, node details, statuses, and effect files after the TUI exits. Do not compare exact escape sequences, coordinates, or complete frames.

**Rationale:** Semantic assertions are stable across harmless terminal rendering changes while durable facts prevent stale transcript content from producing false success.

## Functional Core / Imperative Shell

Pure helpers evaluate expected phase surfaces, run-history outcomes, attempt sequences, cancellation facts, and resize invariants from captured values. The PTY shell creates temporary configuration and workflows, resizes the terminal, writes key bytes, owns reader and child teardown, waits with explicit bounds, and opens the store only after the TUI exits. Any production fixes preserve the existing App/core separation from terminal I/O.

## Risks / Trade-offs

- PTY resize and Unix process-group observations bound the strongest evidence to supported Unix/Linux hosts.
- Accumulated transcripts can contain stale text; each phase must include at least one token unique to the new action and pair it with durable or filesystem facts.
- Timing-sensitive cancellation can flake unless the fixture exposes a deterministic readiness checkpoint before input is sent.
- A failed process test may reveal that current cancellation stops app bookkeeping but not the underlying bounded process; that is a product defect to fix rather than weaken in the oracle.
