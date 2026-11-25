# Closeout Verification Evidence: `integrate-consumer-release-readiness`

## Cohort & Policy Identity
- **Pinned Cairn Producer**: `OnixResearch/cairn` rev `63e503aefc05a225650a8e842ca5bebfdc0260f1` (`cairn 0.2.1`) via `flake.nix`
- **Selected Policy**: `cairn-policy/default.ncl` (`lattice-default`, exported to `cairn-policy/generated/cairn-policy.json`)
- **Policy Hash (Cairn JSON `policy_hash`)**: `8f09d1fdd7503d1d48cbd9ed3248d4026cc7d6fe6d026c1b467b5bfd33d19afb`
- **Policy Check**: `cairn policy export --source cairn-policy/default.ncl --output cairn-policy/generated/cairn-policy.json --check` -> exit `0` (`policy fresh: cairn-policy/generated/cairn-policy.json`)
- **Traceability Markers**:
  - `r[lattice.release_integration.selection]`: `cairn-policy/default.ncl` (`# r[impl ...]`), `tests/test_release_readiness_integration.rs` (`// r[verify ...]`)
  - `r[lattice.release_integration.selection.gates]`: `tests/test_release_readiness_integration.rs`
  - `r[lattice.release_integration.inputs]`: `scripts/release-readiness-inputs.rs`, `scripts/release_readiness/core.rs`, `tests/test_release_readiness_integration.rs`
  - `r[lattice.release_integration.inputs.mcp]`: `scripts/release_readiness/core.rs`, `tests/test_release_readiness_integration.rs`
  - `r[lattice.release_integration.inputs.corpus]`: `tests/test_release_readiness_integration.rs`

## Real Native MCP Observation & Receipt Completion
Executed inside `nix develop .`:
1. `scripts/release-readiness-inputs.rs "$LATTICE_CAIRN_BIN"` -> exit `0`:
   - `passed`: `true`
   - `subject_hash`: `11f6212e4c3387cf804cdea50962ad240877ad2e56ed13f02f8a9c26a39d3e7f`
   - `observation_hash`: `f72a8158dccd938063df0a0924963276c39de32593703cfbc8a1a4d6fd7e7ea4`
   - `policy_hash`: `8f09d1fdd7503d1d48cbd9ed3248d4026cc7d6fe6d026c1b467b5bfd33d19afb`
   - `native_binary_hash`: `156586ec44299c53c25c9a64538f56f232273068f3f1116d2e65d7449a110051`
   - `stdout_hash`: `386fad1d881071d73f8b0416c0a88ebfdc95a55f9dfd74be8c98f1b6214096a0`
   - `stderr_hash`: `af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262`
   - Generated `target/release-readiness/{mcp-contract.json,mcp-results.json,observation.json,source-manifest.json,validation.json,validation.stderr}`.
2. `CAIRN_POLICY="$PWD/cairn-policy/generated/cairn-policy.json" "$LATTICE_CAIRN_BIN" agent-receipt complete --contract target/release-readiness/mcp-contract.json --results target/release-readiness/mcp-results.json --subject-hash "$SUBJECT_HASH"` -> exit `0`:
   - `valid`: `true`, `verdict`: `"PASS"`, `classification`: `"authoritative"`
   - `passed_checks`: `["lattice-native-mcp-validation"]`, `non_passing_checks`: `[]`
   - `completion_receipt_hash`: `decceb10f975bc232d454ce24a26de9e8d7adb345c4e3e1f981a8b7feb1ce2ee`
   - `receipt_hash`: `2d10a603860a0e96e86ba082c19767b60a3d56446cede7b53ecb1029ce0a10ac`
   (Re-verified on the final post-archive tree as well; see verification log.)

## Canonicalization Corpus & Determinism Coverage Audit
1. `"$LATTICE_CAIRN_BIN" preserves-corpus update --path fixtures/release-readiness/canonicalization.json` -> exit `0`:
   - `valid`: `true`, `case_count`: `4`, `stale_count`: `0`, `mutated`: `false`
   - `receipt_hash`: `9c639efff822a5aa77c734a70a9a045bb829e9fc43697ecf0cc0e6a15d848527`
2. `CAIRN_POLICY="$PWD/cairn-policy/generated/cairn-policy.json" "$LATTICE_CAIRN_BIN" determinism coverage-audit --root "$PWD"` -> exit `0`:
   - `valid`: `true`, `verdict`: `"pass"`, `surface_count`: `25`, `missing_surfaces`: `[]`
   - `current_replay_policy_hash`: `8f09d1fdd7503d1d48cbd9ed3248d4026cc7d6fe6d026c1b467b5bfd33d19afb`
   - `receipt_hash`: `1c8d6b1422c67451ff0c108420feec5081418c19d94fcce1055cd561e990d3f2`

## Release-Readiness & Replay Assessment (Truthful Boundary Results)
1. `"$LATTICE_CAIRN_BIN" release-readiness --root "$PWD" --policy "$PWD/cairn-policy/generated/cairn-policy.json"` -> exit `0`:
   - `receipt_hash`: `ec7fc346fbb33ff02c59c2eb4d8d0ef9f936a7ab3a9c2908b6dce4abbe35f0cc`
   - Passing checks:
     - `cairn_validate`: `passed: true` (`b66c7cbffded995bd1c64ce34af101beaca360b114a04e503129caae93faa8c5`)
     - `determinism_coverage_audit`: `passed: true` (`1c8d6b1422c67451ff0c108420feec5081418c19d94fcce1055cd561e990d3f2`)
     - `repository_gate_policy_listed`: `passed: true`, `source: "consumer-selected policy"`, `policy: "consumer-selected-gates"`, `required_commands: ["scripts/check.rs", "octet-precommit-check", "nix flake check -L --option secret-key-files ''"]`
   - Optional / skipped / disabled checks (truthfully retained as `skipped: true`):
     - `release_evidence_index_freshness`, `semantic_review_ledger`, `stack_provenance` (`absent`), `function_address` (`absent`), `stack-provenance_authentication`, `function-address_authentication`, `workspace_contract_matrix` (`disabled`)
   - `onix_release_ready`: `false`, `valid`: `false`, `verdict`: `"fail"` due to:
     - `traceability_coverage` (`passed: false`): pre-existing baseline debt across `.cairn/specs/` + unarchived externally-blocked changes.
     - `mcp_agent_smoke` (`passed: false`, `No such file or directory (os error 2)`): pinned Cairn `0.2.1` `smoke-mcp` hardcodes Cairn self-host fixture paths (`fixtures/agent-completion/*`, `fixtures/agent-receipts/positive.json`) rather than reading `release_readiness_policy.mcp_completion_{contract,results}`.
     - `preserves_corpus_freshness` (`passed: false`): pinned Cairn `0.2.1` hardcodes `fixtures/preserves-canonicalization/corpus.json` whereas Lattice maintains `fixtures/release-readiness/canonicalization.json`.
2. `CAIRN_POLICY="$PWD/cairn-policy/generated/cairn-policy.json" "$LATTICE_CAIRN_BIN" determinism replay --root "$PWD"`:
   - Lattice-owned cases all exited `0` with `valid: true, verdict: "pass"`, `mismatch_count: 0`, `nondeterminism_failure_count: 0`:
     - `status`: `receipt_hash = 8cfd5c5e291a1634f78a7df116b4cad1dba47ad71cb50250e0fad123386ffeb0`
     - `validate`: `receipt_hash = 1ddfb974e4632e05880c97536b7f175151e585fc40309ec8320c99c56f4d8b52`
     - `mcp_describe`: `receipt_hash = 47f5553190fc323db39d08bcb4ff2b6eba1ab7b104ca614001116bed07cb2ad0`
     - `mcp_status`: `receipt_hash = 1a20e91f2b1ee28e589a0b1fe4dc8c61c44530f29f42f68238999288052320f6`
     - `release_readiness`: `receipt_hash = 61955ca657bdfd40b36fd6022be98da5992090a53e4bef068b55252670d7993b`
   - `--group release` exited `1` with `error: No such file or directory (os error 2)` because producer `upstream-default.ncl` replay cases `agent_completion_validate` and `agent_receipt_validate` target Cairn self-host fixtures (`fixtures/agent-completion/*`, `fixtures/agent-receipts/positive.json`), which Lattice intentionally does not fabricate.

## Required External Gate Results
Full repository verification (`./scripts/check.rs`, post-archive `scripts/release-readiness-inputs.rs` + `agent-receipt complete`, `octet-precommit-check`, `cairn validate --strict`, and `nix flake check -L --option secret-key-files ''`) is executed on the post-sync / post-archive tree before pushing `main` and confirmed via GitHub Actions CI.
