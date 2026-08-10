# n8n credential boundary promotion evidence

## Result

Exactly four reviewed n8n credential rows were promoted into maintained Tracey evidence:

- `n8nclone.credentials.n8n_json.placeholders.export`
- `n8nclone.credentials.n8n_json.placeholders.plaintext`
- `n8nclone.credentials.n8n_json.placeholders.reference`
- `n8nclone.credentials.n8n_parity_secret_safety`

No runtime behavior changed.

## Local assertion binding

Markers attach directly to maintained integration tests in `tests/test_n8n_credential_boundaries.rs`:

- reference import and configured-provider execution;
- opaque save, raw-store, and CLI export;
- rejected plaintext CLI import with no store creation; and
- both rejected-output safety and bounded parity fixture/report/output safety.

The parity parent marker binds to both required tests because its accepted clauses span supported and unsupported surfaces. No marker relies on implementation names, file scope, lifecycle classification tests, generated inventory, or archived evidence.

## Exact coverage delta

- baseline requirements/referenced/missing: `731 / 391 / 340`
- current requirements/referenced/missing: `735 / 399 / 336`
- baseline minus current missing: exactly the four-ID allowlist
- current minus baseline missing: empty
- baseline dangling: 0
- current dangling: 0
- baseline `valid: false`
- current `valid: false`
- current policy BLAKE3: `f5188ad02b20fe2c5b86fb6a890b8d90402eed165b9f075c0df96044a3bad177`
- current input BLAKE3: `6a64f687c3f8248cfa667c9080f2174549a70af88c68f4e21faa89e03847a415`
- current receipt BLAKE3: `87869188ce937a58e769a5673b4f8463a63567264280c70382f8eecee2e7357c`

The pure validator rejects duplicate or unsorted receipt IDs, duplicate allowlist IDs, missing expected removals, unrelated removals, additions, dangling markers, successful coverage, missing local bindings, and stale promoted policy rows with deterministic issue codes.

## Inventory

- total: 336
- reviewed partial: 9
- unassessed: 327
- missing-set BLAKE3: `9013bbe478784e6cec07f744410b35c27dad79cdf6c0451b496fbdc1a3e3ce7d`
- review-policy BLAKE3: `5c65366e8948058bc0db5dcb469d2ab925178cf28b95ef38f2aa7342db04119c`
- inventory BLAKE3: `afc25cd14e9f1cdb7343488863fa5b2b921b898c08885196dcccf5fb1088a523`
- issues: none
- inventory `valid: true`

The historical four-row exact review is retained in `tests/fixtures/traceability-n8n-credential-completion-reviewed.json`. Its test reconstructs the reviewed boundary-test bytes by removing only the five expected promoted marker lines before checking the pinned BLAKE3 identity.

## Validation

- `scripts/check.rs` passes formatting, all tests, strict project-only Clippy, identity checks, and accepted Cairn spec checks.
- Three positive/negative promotion tests pass.
- Historical completion-review and inventory tests pass.
- Deterministic inventory generation/checking passes.
- Cairn validation and proposal/design/tasks gates pass.
- Octet retains 912 accepted/current findings with zero new and zero stale under baseline `b3:1420c9a0b1e03612f4cec47bee348cbab937873c4b03fcae75d6d0d7a22257a2`.
- `nix flake check -L --option secret-key-files ''` passes configured `x86_64-linux` checks.

Octet's underlying Cargo subprocess remains classified as exit code 101 while the reviewed wrapper accepts the zero-delta result. Nix omits incompatible `aarch64-darwin`, `aarch64-linux`, and `x86_64-darwin` systems.

## Non-claims

This promotion proves only exact maintained linkage for the four selected requirements. It does not prove arbitrary n8n semantic parity, provider truth, arbitrary secret confidentiality, complete traceability, release readiness, or production readiness.
