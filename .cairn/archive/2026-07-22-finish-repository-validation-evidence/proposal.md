# Finish repository-validation evidence

## Summary

Add maintained adversarial fixtures and tests that directly establish the nine partial repository-check rows while preserving the two already exact test-source rows and all Tracey linkage.

## Scope

- Add a compiled path-dependency fixture with nested `coverage/result.rs`, upstream-style formatting, and a dependency-local Clippy denial trigger.
- Observe root-anchored Git-ignore admission, filtered Nix test compilation, package-scoped formatting, and `--no-deps` strict Clippy configuration.
- Assert the documented check-only signing override and absence of a flake-level signing override.
- Reclassify exactly the selected 11 rows as exact without promoting markers.

## Non-goals

This change does not weaken host signing policy, lint vendored dependencies, reformat imported snapshots, claim arbitrary source-closure completeness, or promote Tracey markers.
