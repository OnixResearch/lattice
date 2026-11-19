## Why

https://github.com/OnixResearch/lattice/issues/16 asks Lattice to record declared
preference factors, and an optional exogenous world variable, on projected
role-protocol choices. The factors must not change exact-label admission. Until
now a session could advance a labelled choice with no record of the factors
that arrived with the upstream choreography.

The producer prerequisite is now published. Choregraph #4
(https://github.com/OnixResearch/choregraph/issues/4) added the default-off
`choregraph.role-choreography.preference-annotations.v1` profile at Choregraph
`fcbae0aa17bc0eab1345529adb9ff104a46540ef` on the canonical Radicle seed. The
related Noble filing is
https://github.com/OnixResearch/noble-lang/issues/67#issuecomment-6796434640.

## What Changes

- Re-pin every Choregraph crate and the `choregraph` flake input from
  `c3577352ee1e39391f5204ee234d991c7dbeb151` to
  `fcbae0aa17bc0eab1345529adb9ff104a46540ef`. Update the source-admission
  receipt, the flake assertions, the source-admission test constants (archive
  BLAKE3 and narHash), `FROZEN_CHOREGRAPH_REVISION`, and the documents that name
  the revision. The producer's v1 receipts are byte-identical to `c3577352`, and
  its new compile-time schemas live under the already restored
  `schemas/preserves`.
- Add `admit_annotated_projection`. It admits the annotated bundle's unchanged v1
  projection through the existing intake. It verifies the producer receipt and
  the annotated complete local set with `preference::linkage::verify_receipt` and
  `verify_local_set`, and checks their linkage to the admitted cohort.
- Add the Lattice `PreferenceProfile`. It defaults to `exact-label-baseline`,
  which ignores factors and leaves the cohort byte-identical to the unannotated
  intake. Only `record-declared-factors` adds an identity-bound
  `PreferenceRecord` to the cohort.
- Reports separate `factors-recorded`, `factors-ignored-profile-off` and
  `game_mapping = not-formalized`. Session inspection shows recorded factors
  only when a record exists.
- `plan_session` rejects a record that names another profile or cohort, drops a
  non-claim, is incomplete or reordered, does not address the decider's exact
  `InternalChoice` labels, or carries a factor owned by another role.

## Consumer, producers and ownership

- Consumer: embedding callers of the projected role-protocol library that
  receive an annotated Choregraph cohort and want an inspectable record of its
  declared factors.
- Producer: Choregraph `fcbae0aa17bc0eab1345529adb9ff104a46540ef`
  (`choregraph_choreography::preference`, Choregraph #4).
- Maintenance owner: Lattice maintainers of `src/engine/protocol/`.
- Durable capability: Lattice can admit a published annotated cohort beside the
  exact-label baseline without a local annotation, projection or solver.
- Repeatability evidence: `tests/test_role_protocol_preference.rs` and the
  unchanged role-protocol targets.

## Non-claims

Factors are metadata. They do not grant authority, select a branch, weigh
labels or convert a label set into a Boolean. They are not credentials,
approvals or workflow-revision merge semantics. Lattice does not compile or
project protocols, and it does not implement a decision-policy solver, mechanism
design or an incentive-compatibility check. Pact's solver and game semantics are
unpublished author claims, and the game mapping is not formalized. The standalone
CLI, its unavailable authority and its v1 create input are unchanged.
