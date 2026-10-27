# Design: Profile admission core and strategy-parity rail

## Context

The repository is a planned extraction with no code. The founding package scopes the full hardened component extracted from Aspen; that work is large and reviewed elsewhere. This change stands up the first runnable, testable slice so later extraction lands on an existing identity/outcome foundation instead of inventing one.

## Decisions

### Decision: Pure core with no runtime dependency; execution lives behind the test rail

**Choice:** `src/lib.rs` exposes strategy, limits admission, and outcome normalization as pure functions over owned data with zero third-party dependencies. Wasmtime execution appears only in the test-side rail module (`#[cfg(test)]`), keeping the shipped crate dependency-free at rung 1.

**Rationale:** AGENTS requires a pure deterministic core and a thin shell. The eventual shell (restricted linking, instantiation) arrives with the Aspen extraction; committing to its shape now would be speculative. A dependency-free core also keeps the Octet deny-all surface small.

**Risk:** the rail exercises only what the corpus covers. Mitigation: the outcome model is total over the enumerated classes and fixtures pin each class.

### Decision: Cranelift versus Pulley as the first differential pair

**Choice:** Execute each corpus module twice per case — once under `Strategy::Cranelift`, once under `Strategy::Pulley64` — with fuel metering enabled and identical limits.

**Rationale:** Both strategies ship in one Wasmtime build, need no external artifacts, and differ in compilation/execution pipeline (JIT versus portable interpreter), giving an honest two-backend comparison without pulling Mantle or SpaceWasm into this repository.

**Risk:** both strategies share Wasmtime's validation and store semantics, so agreement cannot detect shared-engine bugs. Recorded as a non-claim; deeper independence is the founding package's concern.

### Decision: Comparison through pinned `differential-execution-core`

**Choice:** The rail admits cases (program label), admits observations (canonical JSON of the returned value or trap class), runs `admit_independent_backends` + `compare_pairwise`, and classifies against exact-value fixtures with `classify_with_oracle`. The dependency is pinned to immutable revision `cad5a7cb…` on the seed, the same revision Seaglass, ChaosControl, and Ambient consume.

**Rationale:** One shared comparison boundary across the stack; identical pins keep resolution reproducible across consumers.

### Decision: Strict Octet gate from first commit

**Choice:** Pin `octet` at DEH's reviewed revision through the flake, copy the reviewed deny-all configuration, expose `apps.octet-deny-all`, wire `octet.lib.mkConsumerCheck` into checks, and enforce workspace/all-target scope.

**Rationale:** The project-wide hard rule for new Rust projects; cheapest to adopt before source exists.

## Risks / Trade-offs

- Pulley must be enabled in the Wasmtime feature set; if the host toolchain lacks it the rail fails loudly rather than skipping.
- Fuel semantics can differ by strategy version; the corpus pins small modules whose fuel behavior is stable, and exhaustion is asserted explicitly rather than inferred.
- The Cairn lifecycle is bootstrapped against the pinned Cairn revision `b6759012…`; the ambient installed binary has drifted mid-session, so all lifecycle commands in this repository use the pinned flake entry.
