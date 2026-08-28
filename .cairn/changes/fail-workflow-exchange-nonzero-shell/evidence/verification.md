# Verification

Date: 2026-08-28

## Failure evidence

A live workflow-exchange run executed a bounded provider that exited with code `2` and emitted `provider admission failed`. Lattice persisted the shell node as completed and returned a successful terminal exchange observation. This was false completion; no provider report was published.

## Change

Workflow exchange now inspects the persisted output for the graph's typed `ExitCode` port before it projects a completed node. Exactly one canonical `exit code: 0` output remains successful. Nonzero, missing, duplicate, or malformed exit output becomes a failed node and failed terminal execution. Nodes without an exit-code port retain their existing completion semantics.

The global process shell and normal workflow executor still expose exit codes as data. The change is limited to workflow-exchange completion meaning.

## Checks

- pre-change focused workflow-exchange tests: PASS, 9 tests;
- post-change focused workflow-exchange tests: PASS, 10 tests;
- negative nonzero-exit handler fixture: PASS;
- strict package Clippy with `-D warnings`: PASS;
- `scripts/check.rs`: PASS, including the repository test and Clippy suite;
- `octet-precommit-check`: PASS with no new or stale accepted findings; inherited findings remain outside this change;
- strict Cairn validation: PASS;
- serialized `checks.x86_64-linux.lattice` Nix package rail: PASS.

Two full `nix flake check -L --option secret-key-files ''` attempts reached unrelated process tests and failed in different places: one analyzer truncation fixture and one relative-executable fixture. Each exact focused rerun passed. The changed `checks.x86_64-linux.lattice` package rail is rerun with serialized Nix jobs before publication.

## Claim boundary

These checks prove bounded local workflow-exchange classification for the checked runtime. They do not prove provider correctness, process isolation, external effect truth, distributed exactly-once execution, production availability, or release eligibility.
