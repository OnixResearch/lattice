# Repository-validation traceability audit

## Result

The complete 11-row repository-validation tranche is classified:

| Requirement group | Rows | Classification | Bounded reason |
|---|---:|---|---|
| Clippy ownership | 2 | partial | Package-scoped `--no-deps` denial exists, but no maintained warning-triggering vendor fixture directly tests exclusion. |
| Host signing | 3 | partial | The check-scoped override is documented and succeeds, but no maintained hostile missing-key process test or explicit external-policy assertion exists. |
| Formatter ownership | 2 | partial | Package-scoped formatting exists, but no maintained differently formatted vendor fixture directly tests exclusion. |
| Nix test source | 2 | exact | Nix Nextest compiles a repository-owned test that embeds a tracked JSON fixture from the filtered `tests/` closure. |
| Vendored snapshot | 2 | partial | Nested `result` and `coverage` Rust paths exist, but no maintained Git-ignore plus filtered-Nix-closure assertion observes both admissions. |

Three correlated lenses were used: executable configuration, closure observation, and adversarial scope. Deterministic repository checks remain authoritative.

## Coverage

Coverage remains `788 / 473 / 315` requirements/referenced/missing, with no missing additions or removals, zero dangling markers, and `valid: false`.

- missing-set BLAKE3: `2b7906ee39473996244bcfbf795e9ee984704c8bbfe2db9017309647c4d8373c`
- input BLAKE3: `8602c4080f1b246f90ed7c7f74fb7ba2f7920659045f37e964ae76fc77a94190`
- receipt BLAKE3: `731f62c6d8b9d24836784cf590c33d64c45ee0c4401b0800853a22f3ac87c075`

Inventory is 315 total, 11 reviewed, 2 exact, 9 partial, and 304 unassessed. Review-policy BLAKE3 is `788b365acb3fe4dbe38ab55c42829dff6d3a1d0cb00059527d094147a309af15`; inventory BLAKE3 is `38e3e4b76d5198805d9771fa37191a0f97eff5bda2d660f9a73e1f10cd15191b`.

## Validation

The pre-change Nix package, Nextest, format, and strict package-scoped Clippy checks passed with `--option secret-key-files ''`. Positive and adversarial audit validation and deterministic inventory generation/checking pass.

## Non-claims

Partial evidence is not near-complete correctness. Inventory rows create no Tracey evidence and do not prove arbitrary Nix, Git, Cargo, formatter, lint, signing, release, or production behavior.
