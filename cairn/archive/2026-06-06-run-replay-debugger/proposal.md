# Proposal: Run replay debugger

## Summary

Add a replayable workflow debugger built from persisted runner events and typed value summaries. Operators should be able to inspect a historical or active run as a timeline, step through node transitions, compare cached and executed steps, and export a redaction-safe replay bundle.

## Problem

Run history currently summarizes completed runs and node details, but it is not a deterministic replay/debugging surface. Once workflows gain unified runner events, artifacts, triggers, and subworkflows, operators need a way to understand why a run took a path, which inputs produced each output, and how failures propagated without rerunning side effects or exposing secrets.

## Goals

- Persist ordered runner events and safe value summaries for replay.
- Render active and historical runs as a deterministic timeline.
- Support step-through debugging over recorded events without executing side effects.
- Export and import redaction-safe replay bundles for support and tests.
- Preserve secret redaction and artifact reference boundaries.

## Non-Goals

- Time-travel mutate persisted workflow state.
- Re-execute side effects during replay mode.
- Store full plaintext values or secrets in replay bundles.
- Replace normal runner execution or run history storage.

## Success Criteria

- Historical runs can be replayed from persisted events even when the original workflow has changed.
- Replay surfaces show node readiness, cache hits, starts, completions, failures, cancellations, trigger context, and subworkflow nesting.
- Missing artifact payloads or unavailable value stores degrade to clear placeholders instead of panics.
- Replay bundle import rejects malformed or unsafe data deterministically.
