# Resumable component invocation verification — 2026-10-26

## Outcome and ownership

The producer implements `hwr.shell.v3`: one retained host-polled invocation of a synchronous component export, typed failure phases, cumulative resource limits and explicit cancellation/concurrency ownership. Hardened-runtime maintainers own the mechanism, cohort and regressions. Molten retains its scheduler, continuation/generation identities, materialization trust, authority, product effects and canonical receipts. This is a reusable upstream prerequisite, not Molten production adoption.

Baseline was `0455e0958e6f8948d0df5f62950be39519d717a0`. Wasmtime remains exactly `45.0.0`; host `async` support was enabled without admitting guest component async or threads. The resolved lock retains the existing dependency versions. Source-gate and fixture failures are preserved in [initial observations](initial-observations.md), including the failed-run memory/fuel difference that independent consumers must evaluate.

## Executed workspace checks

From the producer repository with task-owned executable scratch directories, `CARGO_PROFILE_DEV_DEBUG=0`, `CARGO_INCREMENTAL=0` and two build jobs:

```sh
nix develop --builders '' -c cargo fmt --all --check
nix develop --builders '' -c cargo test --locked --workspace --all-targets
nix develop --builders '' -c cargo clippy --locked --workspace --all-targets -- -D warnings
```

The fixture-owner refactor passed formatting, 8 core unit tests, 29 admission tests, 36 actual execution tests and strict Clippy. The final WAT-data extraction subsequently passed formatting and the same complete workspace test selection. The execution cases cover retained state, action/cleanup suspension, exact successful-run fuel/output parity, terminal replay refusal, unpolled/suspended cancellation, concurrency retention/release, cumulative fuel exhaustion, memory/input/output limits, cleanup traps, injected epoch expiry and typed module/component failure phases.

## Executed public API smoke

A disposable external Cargo program used the public API, the frozen admitted profile and actual component bytes. It invoked each component both synchronously and through `start_sliced`/`run_slice`, asserted exact whole-outcome equality for these three cases, checked every yielded ordinal/quantum and refused post-completion polling. The program installed no product adapter, imports, scheduler or authority. Its path dependencies were confined to the disposable smoke, not product configuration.

```text
schema=hwr.shell.v3 case=continued yields=20 class=Completed phase=Invocation output=[79, 75] fuel_remaining=98584
schema=hwr.shell.v3 case=start-trap yields=0 class=GuestTrapped phase=Instantiation output=[] fuel_remaining=99999
schema=hwr.shell.v3 case=export-trap yields=0 class=GuestTrapped phase=Invocation output=[] fuel_remaining=99997
scope=upstream-api-smoke product-adoption=false authority=false
```

The successful output is `OK`. Configured quantum was 64 and total fuel 100000. The 20 pending polls are observations, not a claim that each consumed exactly 64 fuel. No output was returned before terminal completion.

## Executed lifecycle checks

Using the repository's explicit `cairn-policy/generated/cairn-policy.json`:

```sh
cairn validate --root . --policy cairn-policy/generated/cairn-policy.json --strict
cairn gate proposal add-resumable-component-invocation --root . --policy cairn-policy/generated/cairn-policy.json
cairn gate design add-resumable-component-invocation --root . --policy cairn-policy/generated/cairn-policy.json
cairn gate tasks add-resumable-component-invocation --root . --policy cairn-policy/generated/cairn-policy.json
```

Strict validation returned `valid: true` with empty findings/issues; all three gates returned `PASS`. These structural checks do not establish runtime correctness, review quality, consumer parity or release eligibility.

## Bounds and repeatability

The pinned runtime and actual-component corpus provide the repeatable capability. Consumers can reproduce the workspace checks and drive the public carrier with their own admitted facts and epoch service. Completion/drop destroys invocation state and releases its permit; cancellation does not undo host imports or promise guest cleanup. Synchronous host callbacks must return. Fuel scheduling and injected epochs do not establish a hard wall-clock bound.

Passing evidence does not prove Wasmtime or guest correctness, host isolation, arbitrary behavioral equivalence, source trust, effect authority, whole-system safety or release eligibility. In particular, Molten's frozen local/shared corpus, materialization bridge, actor/extension composition and product receipts remain separate adoption obligations.

## Pinned source policy and hermetic checks

After the final fixture extraction:

```sh
nix run .#octet-deny-all --builders '' -- --workspace --artifact-dir /dev/shm/hwr-resumable-20261026-octet -- --all-targets
nix flake check -L --builders ''
```

Both exited successfully. The pinned Octet producer (`nightly-2026-03-21`) reported `Status: clean`, `Findings: 0`, `Errors: 0`. The flake completed its host `build`, `tests` and `octet` checks and reported `all checks passed!`; the hermetic debug and release execution runs each passed the 8/29/36 case selections. The command explicitly omitted incompatible `aarch64-darwin`, `aarch64-linux` and `x86_64-darwin` systems; none is claimed checked. Cache index I/O warnings disabled compiler caching without disabling compilation or checks.

Strict workspace Clippy also passed after the final WAT extraction. The review in `../review.md` is an explicit self-review of the producer mechanism and limits, not an independent approval or product-adoption receipt.

## Immutable implementation publication

After source gates, self-review, document-link checking and native requirement synchronization, implementation commit `b84bb2970bedda50ef63366e120e06d796df92f3` was committed and pushed to `rad://z3hRCegTsS8jpJVgxYfb9psEzxHpG/z6MksnXbFoE8zkCkGWhHc8zuxpnEUhrJHv2KECRV4GSv9gkx`, `refs/heads/main`. The push reported the canonical reference updated from `0455e09` to `b84bb29` and exited successfully. It also reported `No seeds found`; this proves the observed canonical Radicle update, not network replication or GitHub availability. The archive-only completion revision will be a descendant of this exact exercised implementation.

Native `cairn sync --execute` applied the three reviewed requirements without conflicts. Strict validation after synchronization returned `valid: true` and empty findings. A local document check covered nine affected Markdown documents and five relative links, with no broken target. These checks do not add behavioral or release authority.
