## Context

Lattice uses `nickel-lang-core` directly in `src/config/mod.rs` and `src/main.rs`. Workflow and orchestration loaders call the configuration helper. Seaglass's extraction proposal starts from a different Nickel package surface, so a shared cohort must be proven before dependency replacement.

## Success Contract

Completion requires accepted producer evidence, one supported dependency cohort, complete call-site dual-run parity, source alignment, removal of duplicate mechanics, and tested rollback. Wrapping current code, using an ambient sibling repository, or accepting changed diagnostics without review is false completion.

## Decisions

### Inventory product behavior before adapter work

The baseline records every source form, import path, contract, export target, diagnostic class, safe-message boundary, missing-file behavior, and reviewed fallback. Tests run before and after core dependency changes.

### Keep evaluation effects in a thin shell

The shared repository may own file/string evaluation mechanics and normalized diagnostics. Lattice owns config path resolution, template directory reads, product contract material, workflow/orchestration decoding, warnings, defaults, CLI exit behavior, and store-opening order.

### Require explicit evaluator compatibility

The producer receipt names exact Nickel packages, versions, and features. Lattice compiles a disposable adapter and runs its fixtures. If supporting Lattice requires a second maintained evaluator implementation inside the shared repository, adoption is rejected or separately designed.

### Preserve safe diagnostics

Shared diagnostics are mapped to existing safe Lattice classes. Raw Nickel debug output, host paths, source content, credentials, and imported secret values do not enter persisted or operator-safe output. Detailed local developer errors remain explicit shell policy.

### Pin Radicle source exactly

Cargo and Nix use one `https://git.onix.computer/<rid>.git` source and exact `rev`. GitHub, branches, ambient sibling paths, and automatic fallback are prohibited. Cargo and Nix generate lock changes; `flake.lock` is never manually edited.

### Use dual-run evidence and VCS rollback

Temporary tests run legacy and shared paths over identical fixtures. Production keeps one authoritative path after acceptance. Rollback reverts source, generated locks, adapters, and duplicate-mechanic removal together; it is not a runtime fallback.

## Validation

Positive fixtures cover application config, templates, workflows, orchestration definitions and proposals, and doctor checks. Negative fixtures cover malformed source, missing imports, contract violations, invalid exports, unsupported versions, diagnostic redaction, source mismatch, and fallback attempts.

## Claim Boundary

Passing tests prove only selected evaluator observations and Lattice adapter compatibility. They do not prove Nickel soundness, evaluator equivalence, import closure, contract correctness, configuration safety, or release readiness.

## Risks

- Nickel dependency differences can block adoption.
- Shared diagnostics can change user-visible behavior or expose source context.
- Product fallback behavior can be accidentally centralized. Architecture tests must keep it local.
- Concurrent config edits in the worktree require this Cairn implementation to baseline and preserve unrelated changes.
