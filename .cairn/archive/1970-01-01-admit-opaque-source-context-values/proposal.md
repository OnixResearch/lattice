# Admit opaque source context values

## Why

Workflow request version 2 used lowercase `Label` values for source repository and revision facts. Real Radicle RIDs contain uppercase characters, so valid source-bound requests fail before dispatch.

## What Changes

- Add a bounded opaque source value type for repository and revision facts.
- Keep provider names as canonical labels.
- Reject empty, oversized, control-bearing, and delete-bearing source values.
- Preserve request version 2 and its identity framing.

## Impact

The JSON shape does not change. Consumers must pin the corrected immutable Lattice revision.
