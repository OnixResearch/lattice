# Maintained project-identity audit

## Scope and non-claims

This evidence proves that current bounded public surfaces contain canonical Lattice identity except for explicitly classified compatibility, protocol, and marker identities; that the accepted `n8nclone.*` requirement-ID manifest is pinned; and that the original rename archive's local paths and bytes are pinned.

It does not prove arbitrary Git history, remote repository identity, publication state, semantic correctness of historical requirements, complete traceability coverage, or release readiness.

## Baseline

Before maintained tests, `cairn tracey coverage --root .` exited unsuccessfully with:

- accepted requirements: 698
- referenced requirements: 329
- missing requirements: 369
- dangling markers: 0
- requirement files: 12
- evidence files: 124
- `valid: false`
- input BLAKE3: `c42291640f88856ca44d066ca68f2b602d0410f51e1e9a0c258455c62b02448e`
- receipt BLAKE3: `9ce2c5cc15a12242fe1f5393440b8c75c83d70fce7ddf49acb0d1ad0ddd3f967`

The target missing IDs were:

- `lattice.project_identity.canonical.scenario.stale-branding`
- `lattice.project_identity.stable-identities.scenario.cairn-history`

Their previous markers existed only in the excluded original rename archive.

## Archived-check audit

The archived checker was not copied unchanged:

- It exempted all lines in `tests/test_identity.rs`, `src/config/identity.rs`, and the project-identity spec, allowing unrelated stale branding to hide in those files.
- Its Cairn-history marker was attached to canonical package/Nix/README substring checks and did not inspect accepted requirement identity or archive immutability.
- Its self-test covered one stable domain and two stale strings but did not mutate accepted IDs or archive bytes.

The maintained audit instead scans tests and compatibility source lines, classifies complete lines, and supplies separate BLAKE3 history manifests.

## Functional core and shell

`tests/identity_audit/core.rs` is the pure core. It accepts snapshots and deterministically provides:

- stale-branding findings;
- missing canonical-surface text;
- the accepted legacy-ID manifest hash; and
- the rename-archive manifest hash.

`tests/identity_audit/shell.rs` is the imperative shell. It reads sorted, non-symlink snapshots from bounded public roots, `cairn/specs`, and only `cairn/archive/2026-07-20-rename-n8nclone-to-lattice/`.

The public scanner admits a case-insensitive `n8nclone` occurrence only when the complete line is a closed marker-only line, or every occurrence is an exact path-owned compatibility or stable-protocol token. Negative tests reject stale README text, stale test text, stale runtime text, an incomplete marker, and a marker followed by stale prose. One previously malformed wrapped marker was replaced by non-claiming follow-up prose; this did not change the missing set.

## Pinned history manifests

The pure manifest core uses BLAKE3 over sorted, length-delimited path and byte fields with distinct domains.

- accepted `path + n8nclone.* requirement ID` manifest: `18df6fa6c52ecf7e21b154020a71c9194f17936fa6958cf830acafdcd8772c5e`
- original rename archive path-and-byte manifest: `79d1fa98e6f3e134950f542e54e899fa54c3202926251677ff2555279f5050f3`

Negative tests require changed accepted IDs, archive paths, and archive bytes to produce different hashes. The original rename archive was not edited.

## Final Tracey result

After scenario synchronization and final formatting, the standard command still exited unsuccessfully with:

- accepted requirements: 698
- referenced requirements: 331
- missing requirements: 367
- dangling markers: 0
- requirement files: 12
- evidence files: 126
- `valid: false`
- requirement-manifest BLAKE3: `af272fb658334367c5632fc68bf56efd67ff51003636d5e7d75c2df5b86c32e9`
- evidence-manifest BLAKE3: `005804c45adce314b7e0c1707f2ad1419ab28f82531661270e2ff96d40849a55`
- policy BLAKE3: `f5188ad02b20fe2c5b86fb6a890b8d90402eed165b9f075c0df96044a3bad177`
- input BLAKE3: `47521b8d4847f26415fc46e1affb0c5da5f43f0293e3470ee6edaf2813358dc8`
- receipt BLAKE3: `ef9bc9e0807bb1370ee3ac6a639f793aede8999398a4dbb30ec7bab3e954e0c3`

A Steel comparison removed only the two target IDs from the baseline and returned `non_target_missing_exactly_equal: true`, `targets_absent_from_final: true`, and `final_dangling: 0`.

## Validation

- `cargo test --test test_identity`: 7 passed, including live positive checks and negative classifier/manifest mutations.
- `cargo test --test test_traceability_marker_alignment`: 2 passed after its legacy guard constants were made self-auditing fragments.
- `cairn policy export --check`: generated policy remained fresh.
- `scripts/check.rs`: passed formatting, complete tests, strict project-only Clippy, and accepted Cairn spec validation.
- Cairn proposal, design, tasks, synchronization, and repository validation passed.
- Final `nix flake check -L --option secret-key-files ''`: all configured `x86_64-linux` checks passed; incompatible `aarch64-darwin`, `aarch64-linux`, and `x86_64-darwin` systems were omitted.

The first isolated attempt omitted untracked new test modules from the Git flake snapshot. After staging, the next attempt correctly exposed missing `README.md`, `flake.nix`, and `scripts/` runtime audit inputs. The final bounded filter includes those public surfaces and only the exact rename archive, and the complete isolated rail passes.

A direct dependency-traversing Clippy invocation encountered pre-existing vendored Rats wildcard-import findings. The repository-owned strict `--no-deps` Clippy rail and the Nix Clippy check both passed.
