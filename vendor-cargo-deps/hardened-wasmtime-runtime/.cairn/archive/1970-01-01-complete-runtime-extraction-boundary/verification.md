# Runtime boundary correction evidence — 2026-10-26

## Scope and ownership

`complete-runtime-extraction-boundary` corrects published baseline
`8d88e00504c112d81b3b6e11e88f6d7cad6c7e79`. Historical extraction archives are
preserved; they do not establish execution of this correction. The runtime
maintainers own engine/admission/limit mechanisms. Molten and Site maintainers
own their independent product worlds, trusted materialization inputs, policy,
output meaning and canonical receipts.

The immediate outcome is an enforced, executable component boundary rather than
an unvalidated Config facade or a core-module substitute. The durable capability
is repeatable exact-artifact admission and bounded execution with consumer-owned
imports and epoch driving. This remains experimental, not stable publication,
production adoption, authority, sandboxing proof or release eligibility.

## Failing-before reproduction

A detached worktree at the exact baseline received a throwaway integration probe
`crates/hardened-wasmtime-shell/tests/artifactbaseline.rs`. It used the baseline
`BoundedRuntime::build(Config, Strategy)` and `PrepareOptions`, the frozen Molten
profile, `Config::consume_fuel(true)`, no descriptors, and `(module)` (8 bytes),
with `limits.max_component_bytes = 1` and `epoch_deadline_ticks = Some(10)`.
The assertion required `BoundedInvocation::prepare` to return an error.

Command, from that baseline worktree:

```sh
nix develop --builders '' -c cargo test -p hardened-wasmtime-shell --test artifactbaseline -- --nocapture
```

Observed exit **101**, one failed test, with:

```text
baseline admitted 8 artifact bytes under a 1-byte limit
```

The permanent corrected regression is
`tests/execution.rs::oversized_artifact_is_denied_before_compilation`; it rejects
before compilation under the replacement admitted API. The baseline probe is
throwaway instrumentation, not a modification of the published source.

## Executed corrected checks

From the producer checkout, inside its Nix shell:

```sh
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
```

The integrated run passed **59 tests**: 9 core unit/parity tests, 29 admission
regressions and 21 actual Wasmtime execution regressions. The count changed from
54 when existing feature/resource tables were split into meaningful semantic
groups; no consumer behavior is inferred from a count. Strict Clippy passed.
Execution covered actual modules and components, admitted consumer imports,
unknown-error failure, fuel, aggregate memory, pointer edges, injected epoch
interruption, shared concurrency, output refusal and ownership before cleanup.

The initial full deny-all and Nix checks failed on source policy. They exposed
incorrect `core_source_scopes = ["src/"]` classification of imperative shell
code, plus real shape/ownership findings in newly discoverable fixtures and shell
code. The corrected scope selects only
`crates/hardened-wasmtime-runtime/src/`; no lint is disabled. Those initial failed
checks are not passing evidence.

Strict native Cairn validation passed with no findings after complete delta
requirements and canonical scenario references were recorded. This establishes
lifecycle structure, not implementation correctness or review authority.

Final deny-all, hermetic check and immutable consumer results are recorded below
only after those commands complete.

### Final local producer checks

The final `cargo test --workspace --all-targets` passed **60 tests**: 9 core,
29 admission and 22 executable shell cases. The epoch cases now separately
exercise service identity and finite-budget admission; the covered inputs are
unchanged. Strict workspace Clippy passed. Rustfmt ran before these checks.

The deny-all command passed with `Status: clean`, **0 findings, 0 warnings and
0 errors**, using the pinned Octet producer and every configured deny flag:

```sh
nix run .#octet-deny-all --builders '' -- --workspace --artifact-dir target/octet-boundary -- --all-targets
```

Proposal, design and tasks gates passed, and strict Cairn validation reported no
findings. Their default policy selection is recorded by the command output; this
is structural lifecycle evidence, not independent review or release authority.

An initial hermetic source fetch failed because the pinned differential harness
has a historical GCC store path in `evidence/rust-gpu-offload-live/README.md`.
The same `cad5a7cb6c6502b0ec5dc76c51e03352260d8efd` source is now a hash-locked
flake input supplied through Crane's existing `overrideVendorGitCheckout` hook.
No dependency revision or source bytes changed; no reference check was disabled.
Later hermetic failures identified fixture-policy findings, which are fixed by
the final clean local deny-all run. A successful hermetic run is recorded only
once it completes.

The immutable implementation candidate is published on its change branch for
consumer pinning. Producer mainline completion remains contingent on the
independent consumer runs and final hermetic results below.

### Final hermetic producer checks

`nix flake check -L --builders ''` completed with **all checks passed** on
`x86_64-linux`. Its build and test rails executed the 60-test suite in release
mode; its separate Octet rail passed. Other declared systems were omitted as
incompatible, not verified. The remaining app-metadata warning is not a test
failure. This run used the final source snapshot subsequently committed as
`ce9b222be993c43e9960ce7c12aed2a5130ecf76`.

The candidate branch was pushed to the repository's signed Radicle publisher
namespace. The unqualified Radicle adapter advertises only canonical main, so
consumer manifests explicitly select publisher
`z6MksnXbFoE8zkCkGWhHc8zuxpnEUhrJHv2KECRV4GSv9gkx` and the immutable revision.
The push reported no seeds; local Radicle publication is observed, network
replication or availability is not claimed.

### Independent consumer execution

Both consumers pinned this exact producer candidate, selected the explicit
publisher namespace, resolved their own pilot lockfiles and ran through actual
Wasmtime Component Model execution without path patches:

- Molten: `molten:component-runtime@1.0.0/actor`, `actor-turn` returned exact
  `070000002a000000`; remaining fuel was **9,973 / 10,000**. Malformed input,
  oversized input and absent epoch service produced respectively `GuestDenied`,
  `InputLimitExceeded` and `DeadlineDenied`.
- Site: `site:content-transform@1.0.0/renderer`, `render-fragment` passed three
  rounds of exact escaped paragraphs, empty input, maximum 1,287-byte output,
  changing input, publication refusal, invalid input and valid calls after denial.
  The four successful cases retained **99,156, 99,951, 92,271 and 99,654** fuel
  from 100,000 in each round.

Repeatable consumer commands, from their respective repository roots:

```sh
CARGO_NET_GIT_FETCH_WITH_CLI=true nix develop -c cargo run --locked --manifest-path pilots/hwr-consumer/Cargo.toml
```

Molten owns the `pilot-hwr-consumer` verification receipt; Site owns
`.cairn/archive/1970-01-01-correct-hwr-component-renderer-pilot/verification.md`. Their
maintainers own product semantics and repeatability. Successful pilot output
does not establish production adoption, materialization trust or authority.

### Integration-owner review

Review checked explicit feature re-admission and resource validation before
engine construction; byte identity, class, world/name and descriptor admission
before consumer effects; aggregate store memory and cloned-engine concurrency;
typed host/trap precedence and unknown-error failure; bounded owned component
results before automatic cleanup; and consumer ownership of epoch operation.
The transfer-limit diagnostic remains an explicit Wasmtime 45.0.0 compatibility
risk, covered by executable oversized-output and cleanup regressions, not
wording tests. No production adoption, trust or isolation claim was added.
This is integration-owner review, not an independent approval receipt.
