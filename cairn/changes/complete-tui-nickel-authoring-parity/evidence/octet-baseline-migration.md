# Octet baseline migration evidence

## Scope

The parity implementation moved pre-existing file/function findings in `src/engine/workflow.rs`, `src/tui/app.rs`, `src/tui/input.rs`, `src/tui/panels.rs`, and `src/workflow_format.rs`. Before migration, every genuine candidate-only semantic finding was fixed.

The guarded migration compared the checked-in baseline with `target/octet-precommit/results.jsonl`, normalizing only existing file-length, assertion-density, and function-length identity drift. It reported:

```text
old=1151 candidate=1141 matched=1141 unmatched_old=5 unmatched_candidate=0
```

No candidate-only semantic key was accepted. The ten resolved findings were bounded to:

- assertion-density debt for `handle_add_node_key`, `handle_canvas_key`, and `node_config_to_portable`, removed by adding direct invariant assertions;
- one of two pre-existing `catch_all_on_enum` findings in `src/tui/app.rs`, removed by replacing a variant-specific catch-all path;
- six of ten pre-existing `raw_arithmetic_overflow` findings in `src/tui/app.rs`, removed by using saturating node-label cardinality arithmetic.

The migration tool has positive and negative self-tests and requires those exact five normalized diagnostics and counts. It rejects every candidate-only semantic key, missing allowlisted resolution, extra resolution, or altered count.

## Verification

- Migration self-test: passed.
- Dry run: 1,141 candidate findings retained; zero candidate-only semantic keys.
- Executed migration: wrote the current candidate identities and retired only the exact reviewed findings above.
- `nix develop -c octet-precommit-check`: passed against the migrated baseline.
- `pre-commit run --all-files`: passed.

This is an identity migration plus reviewed debt removal, not acceptance of new lint debt or evidence of behavioral correctness.
