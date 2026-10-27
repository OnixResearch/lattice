## Why

A valid imported JSON record with nested records or arrays panics in the native embedded evaluator when merged into another record. The pinned external Nickel CLI exports the same value. Explicit field reconstruction is not a fix: users must retain Nickel's ordinary import and merge language.

## What Changes

- Restore the evaluator's constant-or-thunk recursive-field invariant for closed imported container values at the owned source boundary, without input rewriting, assertion removal, panic interception, or an external evaluator fallback.
- Preserve the reviewed Nickel CLI 1.17.0 / nickel-lang 2.2.0 / nickel-lang-core 0.18.0 language cohort while explicitly admitting any package patch source identity.
- Add native adapter regressions for nested imported JSON merges, recursive dependencies after repeated merges, malformed/missing imports, and contract failures with safe bounded errors.
- Compare the meaningful imported merge fixture against the pinned external CLI as an independent smoke oracle.

## Impact

- **Files**: src/nickel/mod.rs, topic-owned evaluator patch/import artifacts and regression fixtures, docs/nickel-eval-source.md; dependency manifest and Nix integration remain the integration owner's responsibility.
- **Testing**: focused native adapter regressions and exact CLI/native JSON-value smoke comparison after source admission; repository validation remains integration-owned.

## Non-goals

No evaluator cohort upgrade, runtime fallback, input normalization, import-policy expansion, arbitrary equivalence claim, or remote publication. An unpublished patch cannot count as production acceptance.
