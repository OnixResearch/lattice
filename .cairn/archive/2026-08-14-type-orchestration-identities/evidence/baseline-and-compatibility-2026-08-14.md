# Baseline and compatibility evidence

## Source baseline

The clean baseline worktree used Lattice revision `cbb5db0e4844b466dcd1877218eb1f00a2a7caf2`.

Pueue task `25938` ran the focused orchestration, replay, inspection, scheduler, and webhook suites before migration. All outer tests passed. The restart child also produced its expected inner negative result before the parent test passed.

The baseline included these surfaces:

- orchestration CLI, core, format, shell, and redb store
- replay and run inspection
- run-store recovery
- scheduler and scheduler occurrences
- webhook claims and operator webhook execution

## Frozen pre-migration record

`tests/fixtures/orchestration-identities/pre-migration-redb-v1.json` contains exact JSON bytes produced by the old redb model. It came from the clean baseline revision above.

The fixture records these existing tables and keys:

- `orchestration_definitions`
- `orchestration_instances`
- `orchestration_events`

Its BLAKE3 identity is `51302c6b18109162868c01205c74a6d3cb378c2551c0b1ff99e78fcbdb89f905`.

`test_orchestration_identity_compatibility` inserts those bytes into redb. It then loads them through the typed store adapters. The test requires exact JSON projection equality and pure event replay equality.

The negative fixture test rejects a malformed stored key before core loading. Its diagnostic does not include the supplied value.

## Nickel compatibility

The checked `examples/orchestrations/llm-review-loop.ncl` file has this BLAKE3 identity on both revisions:

`bc051b8538fd8c5355ddc0fd9888a25dfec3384a2132a184e131d2ad3a003dea`

The format tests require definition and proposal round trips through explicit string DTOs. They also reject invalid identity text before core construction.

## JSON, event, replay, and inspection compatibility

Identity wrappers serialize as the same JSON strings that the old model used. The frozen record test compares complete definition, instance, and event JSON strings.

The event fixture retains the old event identity, prior link, state digest, fact identity, dispatch identity, workflow revision, and sequence. Replay reconstructs the stored state without running effects.

Run inspection tests retain their existing JSON field names and safe text policy. They reject malformed, foreign, and contradictory owners without exposing private content.

## Crash and effect boundary

The baseline recovery suites cover pending dispatch recovery, cancellation, abandonment, observed failure retry, foreign run records, and duplicate trigger admission.

The migrated suites run the same cases with nominal instance, dispatch, reservation, fact, trigger, revision, and event roles. A mismatch fails before a workflow, provider, credential, scheduler, or webhook effect starts.

## Reviewed Octet policy

Octet revision `bac0f3220ada38110c03498ccf2beae5ee83e642` owns the reviewed Lattice orchestration domains and the focused wire-record scope. Lattice pins that immutable revision in `flake.nix` and `flake.lock`.

The Octet metadata tests distinguish core, wire, and outside-scope paths. The reviewed catalog includes `src/orchestration/format/wire.rs` after the DTO split.

The top-level Octet baseline migration compared 864 old findings with 859 current findings. It matched all 859 current findings, admitted only three exact resolved `ambiguous_params` findings, and admitted no new findings. The migrated baseline identity is `b3:06a05f687a1e64bf1e28ddffecc77aa49bc09e901a15e60663701aaf6cc967e1`.

The independent `lattice-core` migration compared 24 old findings with 24 current findings. It matched all findings without an exception. Its migrated baseline identity is `b3:7e0b9fe7f99811d14db2e1f91e3c3090e336d18c148234071298ec9f4ab3f236`.

The historical pilot policy hash remains `76b82ab58db137a1b3d0172faea971ff37564435fa8b6cefa95b483f9fff834c`. Tests require that historical value to remain valid and differ from the expanded current catalog.

## Claim boundary

This evidence proves supported local format and state-machine compatibility for the listed fixtures and tests. It does not prove external effect deduplication, provider truth, remote cancellation, complete history, or production readiness.
