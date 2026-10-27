# Verification record — Phase 1 (2026-09-09)

## Source inventory (r[onix.wasmtime_runtime.source_inventory])

`docs/source-inventory.md` records the reviewed Aspen sources: the
`molten.wasm-component-profile.v1` Nickel contracts and generated profile
(`docs/wasm-component-runtime/`), the four negative profile fixtures, the
product-specific hyperlight runtime (inventoried, out of extraction
scope), the wasmtime 45.0.0 / wasmparser 0.240 / wit-bindgen 0.46
toolchain pins, the complete explicit feature policy, every named
resource limit, and the `molten:component-runtime@1.0.0` world `actor`
identity.

## Fixture freeze (r[onix.wasmtime_runtime.fixture_compatibility])

`fixtures/manifest.json` (`hwr.fixture-manifest.v1`) freezes the four
Aspen negative profile fixtures under `fixtures/aspen/negative/` with
expected `profile_denied` classes, and names the compile, invocation,
limit, trap, import, and pointer fixture classes Phase 3 must add. The
fixtures import the vendored `fixtures/aspen/profile-template.ncl`, so
they are self-contained in this repository.

## Source revision (r[onix.wasmtime_runtime.source_revision])

- `crates/hardened-wasmtime-runtime` remains the pure core (limits and
  outcome admission already landed with the archived profile-admission
  rung).
- New `crates/hardened-wasmtime-shell` is the thin imperative shell. Its
  immutable identity: `wasmtime =45.0.0` with `default-features = false`
  and exactly `cranelift`, `winch`, `runtime`, `component-model`, `wat`
  enabled — matching the admitted profile; async, threads, gc, cache,
  profiling, and parallel-compilation stay compiled out. The workspace
  pin and `SHELL_SCHEMA_ID`/`WASMTIME_VERSION`/
  `ENABLED_WASMTIME_FEATURES` are test-enforced.

## Checks run

- `nix develop -c cargo test --workspace --all-targets`: 9 tests pass
  (8 core, 1 shell) with the 45.0.0 pin.
- `cargo fmt --check` and workspace strict Clippy `-D warnings`: clean.
- `cargo octet check` (deny-all catalog): exit 0, no findings.
- `cairn validate --root .`: valid; proposal/design/tasks gates pass.

## Remaining

Phase 2 (artifact shape, resource limits, epoch deadline, restricted
imports, normalized outcomes; feature-policy admission is complete),
Phase 3 (shell implementation and boundary fixtures), Phase 4 (consumer
pilots and release). Commit is local; push awaits explicit user request
per the repository's agent notes.

## Feature-policy admission (r[onix.wasmtime_runtime.feature_policy], 2026-09-09)

`crates/hardened-wasmtime-runtime/src/feature_policy.rs` admits a
complete explicit feature policy: all 17 reviewed Wasmtime feature facts
are required struct fields, the 12 denied proposals fail admission by
name when enabled, and the five determinism facts (fuel interruption,
NaN canonicalization, relaxed-SIMD determinism, fixed memory growth,
fixed table growth) are required. Admitted policies carry a BLAKE3
identity over the exact facts. Four tests cover the molten profile
admission, stable identity, every denied feature, and every required
determinism fact. The JSON cross-boundary exhaustiveness check lands
with artifact-shape admission, which consumes the profile document.

Checks: 12 core tests pass; fmt, strict Clippy `-D warnings` (all
targets), and the Octet deny-all check are clean.

## Artifact shape and profile document (r[onix.wasmtime_runtime.artifact_shape], 2026-09-09)

`crates/hardened-wasmtime-runtime/src/profile_document.rs` admits the
exact Molten profile document across the JSON boundary: all 17 feature
facts and all determinism facts are required (deny_unknown_fields plus
exhaustive structs), the schema/profile/strategy/evidence-source
identities and every toolchain pin are checked, ambient imports and
WASI surfaces are denied, and zero resource limits fail admission.
`crates/hardened-wasmtime-runtime/src/artifact_shape.rs` compares
inspected artifact identity, class, world, import, and export facts
against declared materialization facts and names the drifted field.
The frozen `fixtures/aspen/molten-profile.json` is the test document.

Checks: 22 core tests pass; fmt, strict Clippy, Octet deny-all, and
lifecycle validation are clean.

## Cumulative resource limits (r[onix.wasmtime_runtime.resource_limits], 2026-09-09)

`crates/hardened-wasmtime-runtime/src/resource_limits.rs` admits the
complete named limit set: fuel, stack, memory pages, tables, table
elements, instances, memories, input bytes, output bytes, component
bytes, WIT bytes, imports, exports, and concurrency. Every zero limit
fails naming the field, and the outcome classes for input, output,
memory, and deadline exhaustion are stable constants. Profile documents
now return an `AdmittedProfile` binding the admitted feature policy to
the derived limits (16 MiB memory = 256 pages, fuel 100000, and so on),
rejecting non-page-aligned memory bounds.

Checks: 25 core tests pass; fmt, strict Clippy `-D warnings` (all
targets), and Octet deny-all are clean.

## Phase 2 core complete (2026-09-09)

- `epoch_deadline.rs`: epoch budgets admit only when positive and an
  invocation without a present host epoch service is denied.
- `restricted_imports.rs`: exact import descriptors bind name, world,
  capability, schema, and adapter identities; WASI name prefixes,
  duplicates, unbound descriptors, and cross-world bindings fail closed;
  an empty admitted set links nothing.
- `outcomes.rs`: thirteen stable outcome classes cover completed,
  profile, artifact, compile, instantiate, import, fuel, deadline,
  memory, guest, host, input, output, and pointer failures, each with a
  distinct lowercase name; limit classes match the named constants.
- Functional-core boundary: the core's runtime dependencies are blake3,
  serde, and serde_json only. Wasmtime appears exclusively in the
  cfg(test) strategy-parity rail (documented in the crate root) and in
  the shell crate.

Checks: 35 core tests pass; fmt, strict Clippy `-D warnings` (all
targets), and Octet deny-all are clean.

## Phase 3 shell and boundary fixtures (2026-09-09)

`crates/hardened-wasmtime-shell/src/engine.rs` builds an engine strictly
from admitted feature facts (explicit feature setters only; the five
always-on proposals need no setter) with fuel metering and epoch
interruption enabled. `invoke.rs` keeps the linker empty so undeclared
and WASI imports fail instantiation, attaches store limits for memory,
memories, tables, table elements, and instances, meters fuel, applies
explicit epoch deadlines (unset deadlines default to the maximum budget
because epoch interruption is always compiled in), and validates guest
pointer arithmetic before every memory exchange.

Six boundary guests (`fixtures/boundary/*.wat`) run the whole path and
produce exactly one stable class each: bounded completion with output,
WASI import denial, undeclared import denial, fuel exhaustion, guest
trap, and oversized output rejection, plus input-bound rejection,
cross-world descriptor denial at prepare, pointer overflow and range
checks, and epoch-deadline normalization. Eleven shell tests pass.

Checks: full workspace tests, fmt, strict Clippy `-D warnings` (all
targets), Octet deny-all, and lifecycle validation are clean.

## Toolchain floor lowered for consumer pilots (2026-09-09)

The second-consumer pilot (Site) builds under its pinned rustc 1.93, so
the workspace `rust-version` floor moves from 1.96 to 1.93. No source
change was required; the extraction uses no post-1.93 language or
library features.

## Remaining for the full rail (2026-09-09)

`nix flake check -L` reaches the Octet consumer check, which enforces
naming rules on the new admission modules: item names must not repeat
ancestor path segments (`feature::policy::FeaturePolicyError`,
`epoch::deadline::EpochDeadline`, `admit_artifact_shape`, and similar
all need shortening), `admit_runtime_resource_limits` and
`policy_inputs` need assertion-density top-ups after refactors, and
`limits`-style identifiers need unit suffixes. All functional
admission, boundary, and shell behavior is implemented and locally
tested; the remaining work is a mechanical API renaming pass plus a
green `nix flake check -L`.

## Octet compliance pass complete (2026-09-09)

The full Octet consumer rail is now green. `nix flake check -L` passes
with exit code 0, including the Octet consumer check, all workspace
tests, strict Clippy -D warnings, fmt, and lifecycle validation.

The naming and architecture compliance pass addressed:
- Module restructuring into directory form (no underscores)
- Path-repeating type names shortened against ancestor segments
- Assertion density raised to 2+ in all functions over 20 lines
- Concrete Hasher construction removed from the core (pure identity
  computation uses blake3::hash on a byte buffer)
- Sentinel fallbacks replaced with explicit match + failure returns
- Checked arithmetic for page-byte calculations
- Quantity-like identifiers renamed with unit-suffix context
- Const-compatible helpers marked const fn

Both consumer pilots pass: Molten (source product) and Site (second
consumer, different world). The extraction is ready for sync and
archive.
