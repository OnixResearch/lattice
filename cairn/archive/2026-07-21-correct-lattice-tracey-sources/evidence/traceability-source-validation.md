# Lattice Tracey source validation

## Scope and claim boundary

This evidence proves that Cairn's maintained traceability command reads Lattice-owned requirement and marker roots from a typed Nickel policy, rejects an unsafe source root, and continues to fail on real missing or dangling linkage.

It does not prove that every accepted requirement is implemented, that every marker is semantically accurate, that tests are adequate, that behavioral correctness holds, or that Lattice is release-ready.

## Baseline observation

Before the local policy source existed, the generated `cairn-default` profile scanned `crates/` and `tools/`. Lattice has no owned evidence in those roots. The baseline `cairn tracey coverage --root .` receipt reported:

- accepted requirements: 692
- referenced requirements: 0
- missing requirements: 692
- evidence files: 0
- `valid: false`
- receipt BLAKE3: `960ae80e0a14ae95e86bf28a4914728c0d461b097b65a63b5c5cbab0162ed4cd`

The earlier retry archive incorrectly attributed these roots to `.config/tracey/config.styx`; Lattice had no such file. The roots came from `cairn-policy/generated/cairn-policy.json`.

## Bounded approach registry

| Family | Mechanism | Result | Evidence or blocker |
|---|---|---|---|
| External Tracey config | Add `.config/tracey/config.styx` | Falsified | Cairn's maintained `tracey coverage` alias reads the generated Cairn policy profile, not the external Tracey config. |
| Generated-policy overlay | Import generated JSON from Nickel and overwrite that same JSON | Rejected | Cairn's Nickel export boundary returned `destination must not overwrite a dependency`; accepting this route would create circular policy generation. |
| Repository-owned Nickel policy | Check in typed policy contracts/defaults, customize only Lattice roots, and regenerate JSON | Validated | Export, freshness, positive/negative tests, source discovery, and isolated Nix checks completed. |

The search budget was three materially distinct mechanisms and one adversarial pass. It stopped when the non-circular Nickel source passed deterministic checks and the alternatives were falsified or blocked.

## Policy and fixture validation

The repository owns `cairn-policy/default.ncl` and `cairn-policy/contracts.ncl`, imported from Cairn revision `19fc6793720d5fea0185d4d3cab1626ddbc67a40`. The default profile retains the exact `impl`, `verify`, `depends`, and `related` marker verbs and scans:

- accepted requirements: `cairn/specs` Markdown
- Rust evidence: `src`, `tests`, and `examples`
- Nickel evidence: `examples` and `cairn-policy`
- Rust tool evidence: `scripts`

`vendor-cargo-deps`, `target`, generated JSON, and the stale `crates`/`tools` fallback roots are not evidence sources. Cairn's collector reads only files with the configured extensions; listing `cairn-policy` does not execute Nickel imports or ingest generated JSON.

The invalid Nickel fixture uses `../src`. `cairn policy export` rejected that fixture through Cairn's safe-root policy while accepting and regenerating the positive policy. A second `policy export --check` reported the generated JSON fresh.

## Focused and repository checks

- `cargo test --test test_traceability_policy`: 2 passed, covering exact positive roots and negative fallback/unsafe-root behavior.
- `scripts/check.rs`: passed formatting, the complete repository test suite, strict project Clippy, and accepted Cairn spec validation.
- Cairn proposal, design, and tasks gates: passed.
- `nix flake check -L --option secret-key-files ''`: all five bounded `x86_64-linux` checks passed after `cairn-policy` was retained in the source closure. The first isolated attempt failed because the compile-time policy and fixture were absent; the source filter correction was then validated by the passing rerun.
- Nix omitted incompatible `aarch64-darwin`, `aarch64-linux`, and `x86_64-darwin` systems.

## Corrected discovery before requirement synchronization

The corrected command scanned 12 requirement files and 123 evidence files and reported:

- accepted requirements: 692
- referenced requirements: 323
- missing requirements: 369
- dangling markers: 12, including the four not-yet-synchronized IDs in this change
- `valid: false`
- policy BLAKE3: `f5188ad02b20fe2c5b86fb6a890b8d90402eed165b9f075c0df96044a3bad177`
- receipt BLAKE3: `2c11e0a9b194534cc899a847eed845c0fb9e8fccc58ce3fbee2cb61e35933092`

This is the intended fail-closed transition from hidden debt to measured debt. Final post-sync counts and identities are recorded below after synchronization.

## Post-sync result

After synchronizing the new requirement and three scenarios, the standard command reported:

- accepted requirements: 696
- referenced requirements: 327
- missing requirements: 369
- dangling markers: 8
- requirement files: 12
- evidence files: 123
- `valid: false`
- policy BLAKE3: `f5188ad02b20fe2c5b86fb6a890b8d90402eed165b9f075c0df96044a3bad177`
- input BLAKE3: `9421b315927253e0f868e9cbdaeafc59bf6dfd5241188becf668dbd12f9d96f7`
- receipt BLAKE3: `2160e982d7019220178a37bdf389db31d7ac2dfff3e9f029babd97e7cf2ff0cf`

All four IDs introduced by this change are referenced and absent from both the missing and dangling sets. The remaining 369 missing requirements and eight dangling legacy markers are now exact, visible follow-up debt. The unsuccessful exit is required evidence that source correction did not manufacture a false-green coverage result.
