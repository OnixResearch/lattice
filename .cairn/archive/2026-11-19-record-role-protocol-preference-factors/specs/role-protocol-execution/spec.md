# Role Protocol Declared Preference Factors

## ADDED Requirements

### Requirement: Declared preference factors are default-off metadata

r[lattice.role_protocol.preference_factors] Lattice MUST admit declared preference factors, and an optional exogenous world variable, only from an admitted Choregraph `choregraph.role-choreography.profile.preference-annotations.v1` cohort. It MUST record them only under an explicitly selected `record-declared-factors` session profile. The default `exact-label-baseline` profile MUST ignore them, and every report MUST distinguish `factors-recorded`, `factors-ignored-profile-off` and `game_mapping = not-formalized`.

#### Scenario: Selected profile records the decider's factors

r[lattice.role_protocol.preference_factors.recorded]
- GIVEN an annotated projected cohort with factors and a world variable on a decider `InternalChoice`
- WHEN it is admitted under `record-declared-factors`
- THEN the cohort MUST record, per role, each decision's paths, exact labels, factors and world
- AND the report MUST state `factors-recorded` and `not-formalized`
- AND session inspection MUST expose the recorded factors after persistence.

#### Scenario: Annotated input without opt-in records nothing

r[lattice.role_protocol.preference_factors.default_off]
- GIVEN the same annotated cohort
- WHEN it is admitted under the default profile, or offered to the producer baseline selection or the standalone CLI
- THEN no factor data MUST enter the session
- AND the cohort, session and inspection bytes MUST equal those of the unannotated `admit_projection` path
- AND the report MUST state `factors-ignored-profile-off`.

#### Scenario: Annotated producer cohort is the exact pinned revision

r[lattice.role_protocol.preference_factors.producer]
- GIVEN Choregraph `fcbae0aa17bc0eab1345529adb9ff104a46540ef` publishes the annotation profile on its canonical Radicle seed
- WHEN source admission and the frozen role-protocol intake run
- THEN every Choregraph Cargo dependency, the Cargo lock source, the flake input and lock, the admission receipt and `FROZEN_CHOREGRAPH_REVISION` MUST name that revision
- AND the admission receipt MUST bind its source archive BLAKE3 and the lock its narHash
- AND the producer's v1 projection receipts MUST remain the ones the unchanged v1 intake admits.

### Requirement: Recorded factors are identity-bound to the cohort

r[lattice.role_protocol.preference_factors.identity] A recorded factor set MUST be verified with the producer's `verify_receipt` and `verify_local_set` against the annotated bundle. It MUST be linked to the admitted v1 receipt, global artifact, ordered local artifacts and complete local set, and it MUST be bound into the session-plan identity. A crossed, stale, reordered, incomplete or tampered receipt or annotated local set MUST be rejected.

#### Scenario: Receipt or local set comes from another cohort

r[lattice.role_protocol.preference_factors.identity.crossed]
- GIVEN two annotated cohorts over the same protocol with different factors
- WHEN one cohort's bundle is paired with the other's receipt body, receipt bytes or annotated local set
- THEN intake MUST fail with `protocol.identity-mismatch` under both profiles.

### Requirement: Factors cannot influence labels, ownership or authority

r[lattice.role_protocol.preference_factors.boundary] Recorded factors MUST NOT select or narrow a label, weigh branches, convert a label set into a Boolean, change cursor or ownership decisions, enter envelopes or authority requests, or grant authority. Exact-label admission MUST stay unchanged.

#### Scenario: Baseline and recorded sessions run the same choice

r[lattice.role_protocol.preference_factors.baseline_comparison]
- GIVEN baseline and recorded sessions over one annotated choice fixture
- WHEN the same undeclared, factor-identifier, wrong-role and declared labels are planned, and the same choice, owned transfer and denied-authority dispatch run
- THEN both sessions MUST return the same rejections and plannable labels
- AND their committed cursors, owners, attempt states and envelope semantics MUST be byte-identical
- AND no envelope or authority request MUST contain a factor or world name.

#### Scenario: Forged record tries to steer a choice

r[lattice.role_protocol.preference_factors.boundary.forged]
- GIVEN a record that narrows or reorders labels, moves a decision to another role, reassigns a factor, misaddresses a choice, drops its factors or a non-claim, names another profile or receipt, or adds a selected-label, weight, utility or authority field
- WHEN the session is planned or the cohort is decoded
- THEN planning MUST fail with a stable issue code or decoding MUST fail before a session exists.
