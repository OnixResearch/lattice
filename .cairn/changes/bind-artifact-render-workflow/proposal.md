# Bind the artifact-bound tiled render consumer

## Why

Issue #18 needs an actual Lattice consumer of Aspen's already-owned renderer,
batch executor, accepted-output observations and strict verifier. Lattice's local
component ABI is not Aspen's batch ABI, and historical producer campaigns cannot
qualify a missing Lattice binding.

## Change

- Import the canonical contract closure from anonymously acquired immutable
  public Aspen `afc8308bff9a8c50b2315cb4a17c9454252d7c45`; retain licenses,
  archive/file BLAKE3 and owned sparse-workspace metadata.
- Add the opt-in exact `process/lattice-render-v1` operation graph. Existing graph
  admission, effect authority, bounded process owner and no-retry rules remain
  authoritative. No implicit read-only grant can dispatch this mutation.
- Bind logical owner/attempt/fence/budget and actual host-issued execution context
  before possible Submit. Journal uncertainty and reconcile the exact producer
  run without resubmission. Observe may have a new host observer, not a new compute
  owner or budget.
- Retain accepted content and require canonical tile/assembly validation plus
  strict final verification before a completed graph output. Cancellation intent,
  observed stop, supersession, reserved resources and missing usage stay distinct.
- Supply focused consumer regressions and an opt-in Nix-driven retained real
  graph/component/process fixture with ACK-loss, controller/worker interruption,
  observer disconnect/cancellation and same-backend comparison.

## Evidence boundary

The anonymous archive was acquired and imported with exact identity
`a6307998de7d69f737e1ce39465c63f6742aa4612979321597a304b89771363d`.
Implementation, regression definitions and runnable fixture are authored here;
no build, test or qualification fixture was executed by the implementation worker.
The integration owner must retain actual focused/regression/fixture results. An
unexecuted fixture, producer test or historical producer receipt is not a passing
Lattice acceptance criterion. See `docs/render-workflow.md` for exact commands,
source identities, ownership and unavailable telemetry/cross-backend/two-host
claims.
