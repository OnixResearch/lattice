# Proposal: Add preview-first resource operation plans

## Why

Lattice already owns typed operation contracts, durable approvals, effects, persistence, and recovery. It does not define a resource-specific plan that binds catalog observations, replicas, conflicts, estimates, and destructive effects.

A cross-device catalog needs one safe orchestration boundary for copy, move, materialize, derive, and delete requests. A generic command or raw path cannot provide that boundary.

## What Changes

- Add a pure resource-operation plan to `lattice-core`.
- Bind plans to exact catalog, resource, content, replica, locator, observation, policy, and operation identities.
- Include bounded estimates, conflicts, requested effects, and required capabilities.
- Require exact approval and current-state rechecks before effects.
- Record completed, denied, failed, cancelled, and uncertain observations without claiming exactly-once execution.
- Add one fake catalog adapter and one fake executor for conformance tests.

## Impact

Direct commands and workflow nodes can use one plan shape for resource operations. Lattice retains orchestration authority and does not become the resource catalog or filesystem owner.

## Dependencies

The implementation depends on a published resource-catalog contract. It reuses current Lattice operation contracts, approval, effect-authority, persistence, and recovery surfaces.

## Non-goals

- Do not implement catalog storage, search, hashing, filesystem traversal, cloud access, or peer transport.
- Do not accept raw host paths as canonical resource identity.
- Do not use sampled fingerprints as exact content identity.
- Do not claim exactly-once external effects.
- Do not add a Spacedrive source dependency.
