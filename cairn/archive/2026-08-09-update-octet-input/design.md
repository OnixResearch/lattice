# Design

## Dependency boundary

The stack dependency plan binds the old pin, published Octet revision, validation plane, adapter, and expected `flake.lock` path. The coordinator must make the lockfile change.

## Semantic migration core

The migration core groups findings by normalized semantic fields. It ignores `crate_name` because the old runner used `n8nclone` and the current package uses `lattice`.

Normalization remains lint-specific. Each normalizer must accept the old and new reviewed forms. It must preserve the parameter, function, unit, type, and file facts that identify the warning.

The migration compares each semantic group count. A candidate count greater than the old count fails. Stable-ID membership alone is insufficient because one identity can occur in several crate contexts.

## Imperative shell

The script shell reads JSON and JSONL files. The pure core normalizes, groups, and compares values. The shell writes a baseline only after the complete transition passes.

## Validation

Positive tests cover each reviewed wording transition and accepted occurrence reduction. Negative tests cover changed parameter facts, changed unit facts, malformed messages, candidate-only groups, and occurrence growth.

The final checks include the migration self-test, the Octet stable-ID gate, repository checks, Cairn validation, and the bounded Nix flake check.

## Claim boundary

The migration proves only the reviewed transition between two finding sets. It does not prove source correctness, Octet correctness, whole-workflow correctness, or release readiness.
