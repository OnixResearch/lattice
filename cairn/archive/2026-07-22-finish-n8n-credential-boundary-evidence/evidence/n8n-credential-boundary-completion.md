# n8n credential boundary completion evidence

## Result

Four reviewed n8n credential rows now have exact maintained evidence without receiving implementation or verification markers:

- `n8nclone.credentials.n8n_json.placeholders.export`
- `n8nclone.credentials.n8n_json.placeholders.plaintext`
- `n8nclone.credentials.n8n_json.placeholders.reference`
- `n8nclone.credentials.n8n_parity_secret_safety`

No production Rust behavior changed. The completion uses direct integration assertions plus a contract clarification for the designated plaintext rejection fixture.

## Boundary observations

`tests/test_n8n_credential_boundaries.rs` proves:

- an imported opaque credential identity becomes the expected `n8n` provider reference;
- a configured provider supplies a non-empty execution-only shell binding before the side effect runs;
- a missing provider value fails the node and leaves the side-effect path absent;
- a provider-resolved workflow can be saved and exported through the n8n CLI while output JSON contains only the opaque reference identity;
- export output, report, command output, and raw redb bytes omit the resolved plaintext sentinel;
- a plaintext CLI import exits unsuccessfully, writes a safe report with opaque-reference guidance, omits the sentinel from output/report, and does not create the configured redb store; and
- supported fixtures, checked-in reports, rendered diagnostics, and CLI output omit plaintext while the designated negative input contains exactly one bounded sentinel.

Positive and negative cases are paired at the execution, import, persistence, export, and parity boundaries.

## Contract correction

The parity parent requirement previously prohibited plaintext in every fixture while its child required a plaintext rejection fixture. The accepted contract now permits one designated unsupported fixture to contain one bounded sentinel solely as import-validation input. Supported fixtures, checked-in reports, rendered diagnostics, and test output remain plaintext-free.

This is not a broad n8n parity claim.

## Portfolio and adversarial audit

- Implementation-shape inference was rejected because it could not prove save, raw-store, CLI, and execution boundaries.
- Core-only tests were rejected because they could not prove store non-creation or operator output safety.
- Removing the plaintext fixture was rejected because it would remove the required negative path.
- A bounded negative-input exception plus multi-boundary integration assertions was selected.

The adversarial pass checks missing-provider side effects, resolved-value raw-store leakage, opaque export identity, report guidance, sentinel multiplicity, selected marker absence, unrelated classification preservation, and unchanged failing coverage.

## Inventory

- total: 340
- reviewed: 13
- exact: 4
- partial: 9
- unassessed: 327
- missing-set BLAKE3: `e0ce72fe7f22dfe1fb007d33d93d6bc0b61b9efbc389339e9a0ed8656bc6f13b`
- review-policy BLAKE3: `5d4ba383454de50a58ce41d3ba1503985b53aba70b3cfbc2bad3f629b92a3709`
- inventory BLAKE3: `029ce36ef9f30cb78672b64bb0204e851468af539c1e4b2f2ada017b1209f55e`
- issues: none
- inventory `valid: true`

The four exact decisions bind current BLAKE3 identities for `src/main.rs`, `src/n8n_json.rs`, `src/engine/executor.rs`, both maintained n8n test suites, and every parity fixture/report named by the corrected contract.

## Coverage invariant

The completion contract adds five accepted and five referenced lifecycle IDs while the selected credential IDs remain unpromoted:

- baseline requirements/referenced/missing: `726 / 386 / 340`
- current requirements/referenced/missing: `731 / 391 / 340`
- baseline minus current missing: empty
- current minus baseline missing: empty
- dangling: 0
- current `valid: false`
- policy BLAKE3: `f5188ad02b20fe2c5b86fb6a890b8d90402eed165b9f075c0df96044a3bad177`
- input BLAKE3: `a5f2db4132bac5c731b080329db2ca855cedde7c54c26ddfef5a088552ec6523`
- receipt BLAKE3: `8391cb39ece3a8d814207114df934aa46c4107a98bcf430d52fb747a6cae2403`

The pure completion validator rejects row omission, unexpected rows, classification/rationale/evidence drift, selected markers, duplicate selected IDs, removed/added/duplicate/unsorted missing IDs, dangling markers, and successful global coverage with deterministic issue codes.

## Validation

- Baseline repository tests passed before the completion work.
- Four n8n boundary tests pass.
- Two completion-policy tests pass.
- Historical remaining-credential audit and inventory tests pass.
- `scripts/check.rs` passes formatting, all tests, strict project-only Clippy, identity checks, and accepted Cairn spec checks.
- Deterministic inventory generation/checking passes.
- Cairn validation and proposal/design/tasks gates pass.
- Octet retains 912 accepted/current findings with zero new and zero stale under baseline `b3:1420c9a0b1e03612f4cec47bee348cbab937873c4b03fcae75d6d0d7a22257a2`.
- `nix flake check -L --option secret-key-files ''` passes configured `x86_64-linux` checks.

Octet's underlying Cargo subprocess remains classified as exit code 101 while the reviewed wrapper accepts the zero-delta result. Nix omits incompatible `aarch64-darwin`, `aarch64-linux`, and `x86_64-darwin` systems.

## Non-claims

Exact classification is not marker promotion. This evidence does not prove arbitrary n8n semantic parity, provider truth, remote exactly-once effects, arbitrary secret confidentiality, complete traceability, release readiness, or production readiness.
