## Design

The qualification adds no second compiler, scheduler, authority policy or
provenance system. Every regression uses an existing production route:

| Concern | Route exercised |
|---|---|
| Construction | `authoring::shell::compile`, store source preview/admission, `lattice import --construction` |
| Runtime admission | `authoring::construction::admit_runtime` and `admit_materialized` |
| Authority | `standalone::inspect_workflow` and `OperatorRunContextSource` exact grants |
| Execution | `headless::run_saved_workflow*`, `run_workflow_snapshot_with_authority_provider_and_cancellation`, `orchestration::start_with_execution`, `lattice run` |
| Navigation | `authoring::construction::lowering::origin_for`, TUI `source.origins`, `lattice authoring origin` |

Functional core / imperative shell is unchanged: the regressions observe the
existing pure admission and the existing shells. Each new validation behavior has
a positive and negative control in the same test, for example a host registry
that compiles beside copy/drop-restricted registries that do not, an exact grant
that runs beside absent and foreign grants that do not, and a successful first
effect that releases its dependent beside a failed or cancelled one that does
not.

## Type vocabulary

The first target registry declares `Unit`, `string` and `exitcode`, all copyable
and droppable, and two operations: `shell-source` and `shell-stdin`. Admitted
construction projections require exact type-name equality within that registry.
`any` is not a registry type, so the legacy materialized equality/`any`
compatibility rule does not widen an admitted construction edge; a port with an
undeclared or different type rejects. Copy/drop rules are host registry facts,
enforced by native elaboration and again by projection admission.

## Host-run qualification

The v3 runtime admits only root-owned, non-writable Nix-store executables and binds
`/proc/self/exe`. Nix sandboxes do not map host root, so store entries appear as
the overflow user and every v3 positive fails closed inside any flake check. The
sandboxed `construction-runtime` check therefore could never qualify and is
removed rather than kept failing. `constructionRuntimeTests` stays as the
store-installed release test package (`packages.<system>.construction-runtime-tests`),
and `apps.<system>.construction-runtime-qualification` runs
`scripts/construction-runtime-qualification.sh` on the host with the exact
packaged `lattice`, companion worker and reviewed Bash backend. The script
refuses an existing output path and retains source, expanded graph,
`commands.tsv`, per-step logs and `summary.json`. It is not wired into
`scripts/check.rs`.

The immutable-executable policy is unchanged. The final observation is a host
`nix run .#construction-runtime-qualification` over flake-built release
artifacts. Earlier focused runs used dev-shell executables that were stripped and
added with `nix store add-path`; they satisfy the same policy and are recorded as
such, not as app observations.

## Test partition

Plain Cargo and the sandboxed `nextest` check cannot select the worker, so every
captured-worker or v3 test is `#[ignore]`d with a reason and runs in the host app.
`nextest` no longer sets `LATTICE_CLOSED_EVAL_WORKER`. A non-ignored regression
re-runs its own executable with no worker selected and proves captured
evaluation fails closed, leaving a bounded draft diagnostic and no admission.

## Producer cohort and vendoring

The parent re-pinned nickel-eval to `f0ea0d2a31d3482ddf4e575482f6ddf7a7e7d4e9`
because crane rejected the reserved `Cargo.toml.orig` in `2d1fb2b5`, and
`lattice-cargo-vendor` restores Choregraph's above-root
`library/nickel/graph-construction-v1.ncl` and
`contracts/nickel/graph-construction-contracts-v1.ncl` for
`choregraph-nickel`'s construction profile. This change applies the same edits
so the app builds; their ownership and documentation stay with the parent.
