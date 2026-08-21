# Explicit workflow dependency validation

<!-- r[verify lattice.workflow_validation.explicit_dependencies.validation] -->

## Baseline

A detached worktree at `71628cd` passed the focused core, workflow-validation, runner, and portable-format tests.

The baseline fixture produced `expression-ok` through incidental serial order. Its runner and Choregraph graphs contained no dependency pair.

## Implementation result

The pure core now extracts ordered consumer, field-path, producer, and output-port facts. The runtime and portable adapters use one exhaustive field inventory.

Runner admission and Choregraph lowering receive the union of graph edges and expression dependencies. Choregraph retains cycle, order, readiness, lifecycle, and replay authority.

Runtime contexts contain only admitted references. Missing values fail before credential resolution, attempt creation, or effects.

Expression-aware cache keys bind exact producer values. Cache keys without expression values retain their prior framing.

Portable source bytes remain unchanged. The checked fixture records its source length, BLAKE3 identity, runner-plan identities, and Choregraph machine identities.

## Positive and negative evidence

The focused rail covers these positive cases:

- direct and duplicate references
- an existing graph edge
- derived execution order
- exact runtime context
- cache hits
- portable reopening and byte preservation
- runner-plan and machine identity changes

The focused rail covers these negative cases:

- malformed delimiters
- missing nodes and ports
- self-references
- combined cycles
- unsupported prompt-file path expressions
- missing runtime values
- stale runner-plan identity
- invalid dependencies before shell effects

## Repository validation

These checks passed:

- `cargo test -p lattice-core`
- `cargo test --test test_expression_dependencies`
- focused workflow-validation, runner, workflow-format, Choregraph-adoption, and migration tests
- `cargo clippy --package lattice --all-targets --no-deps -- -D warnings`
- `scripts/check.rs`, including operator checks and the complete Cargo suite
- `octet-precommit-check`
- Cairn validation and proposal, design, and task gates
- bounded `nix flake check -L`, with 1,038 nextest cases passed and one skipped

The broad workspace Clippy command still reports pre-existing findings in vendored Rats, Animus, and verified-logic sources. The repository-owned scoped Clippy gate passes.

The Octet migration accepted no semantic growth. It reduced the root accepted count from 855 to 847 and the pure-core count from 24 to 23.

The prior backend-orchestration sync lacked accepted-spec scenario markers. This change repaired those markers before repository validation.

The Choregraph source check now captures its expected Nickel rejection explicitly. It also checks the resolved-prompt lowering path.

## Claim boundary

This evidence establishes supported expression extraction, dependency projection, runtime context confinement, identity binding, compatibility, and fail-closed cases.

It does not establish effect commutativity, provider correctness, runtime progress, arbitrary expression meaning, workflow correctness, or release eligibility.
