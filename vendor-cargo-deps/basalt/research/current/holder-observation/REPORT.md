# Non-consuming holder observation evidence

## Source and scope

Qualified Basalt source: `c67a380b1fabc6d34a220c3e7d4c6c7fd9760e2c`.
Base: `3f1fd1ed63f2b55f07a27f662e5a720518662b7e`.
Published UCAN source: `3a2aa043068a82663cf72ec05ed5d56a7d65ecd5`.

The importer retrieved the published Git object, not a sibling checkout.
Exact source parity passed for 697 files. Snapshot BLAKE3:
`82b5e1c414fadcbdc90ef951624c9ccd4e8eb6bdce0d57daf2830fd79cd592fb`.

`LiveUcanEnforcement::observe_holder` delegates verification to UCAN with the retained private holder authorization.
The call accepts explicit token, verification services, time, and recheck policies.
It accepts no replay port. It returns UCAN's borrowed observation, not new Basalt enforcement or a new receipt.

The selected source inventory includes the upstream recheck document and both recheck source files.
Cohort revisions and generated projections match the imported source.
The source-bound receipt and Nostr implementation identities changed.
Receipt algorithms, Nostr credential contract, policy identity, and producer schema did not change.

## Observed checks

Checks ran on 2026-09-08. Complete private logs and receipts are retained outside the worktree under
`neural-stream/.pi/drains/materialization-complete/` in the research workspace.
This report records the bounded results without publishing host paths or fixture transport contents.

| Evidence | Observed result |
| --- | --- |
| `basalt-recheck-baseline-v1.log` | 28 enforcement tests passed before changes |
| `basalt-recheck-import-v1.log` | Published source import and 697-file parity passed |
| `basalt-recheck-focused-v2.log` | Six importer controls passed; subsequent API compilation failed |
| `basalt-recheck-focused-v4.log` | 109 library, 33 enforcement, five final-profile, and three Nostr tests passed |
| Same focused log | One compile-fail type-separation control, formatting, and minimal all-target strict Clippy passed |
| `basalt-recheck-release-v1.log` | Direct owner release verifier passed in 7m42s |
| `basalt-recheck-release-v1/summary.json` | Release verdict PASS; 245 minimal nextest and 258 CLI nextest tests passed, with no skipped tests in those runs |
| Same release evidence | Contract fixtures, doctests, dependency checks, library/CLI Clippy, package inventory, and documentation passed |
| `basalt-recheck-flake-v1.log` | Complete `nix flake check --no-update-lock-file` passed in 2m05s |
| `basalt-recheck-rust195-v1.log` | Pinned Rust 1.95 profile passed 109 library and 33 enforcement tests |

The release verifier marked the unavailable Clankers consumer fixture as deferred with success status.
That entry is **not execution evidence**. No Clankers compatibility claim is made.
The direct verifier ran Cargo steps and did not use its prevalidated-freshness flags.
The Nix release derivation does not replace the direct Cargo qualification.

## Boundary controls

The five new integration tests cover:

- repeated direct and delegated observations without another replay commit;
- unchanged original receipt and bounds, followed by duplicate invocation denial;
- token substitution and observation time before initial admission;
- revoked leaf, revoked delegated parent, expiry, and revocation service failure;
- changed profile and caveat policy denial.

The compile-fail example rejects an observation where live Basalt enforcement is required.
Upstream UCAN owns observation construction, lifetime, and non-cloning guarantees.
Basalt does not copy that verifier or issue another holder authorization.

## Preserved failures

The first focused run caught a test declaration typo.
The next run reused artifacts that did not expose the new UCAN symbols.
A fresh target directory compiled the imported API and passed the final checks.

A trial Nickel export omitted Nostr producer metadata and was rejected during diff review.
The existing Rust `nostr_identities` example generated the complete projection.

The third focused run rejected the old receipt golden hash after the source revision changed.
The exact expected hash was updated for the unchanged receipt algorithm.
Redaction, replay, and substitution controls remain in place.
No failed run is counted as complete qualification.

The first evidence-validation command used a development shell without `CAIRN_POLICY_PATH`.
It stopped before validation with an unbound-variable error.
The corrected command used the same pinned Cairn tool and explicit policy as the baseline lifecycle check.

## Reproduction

Use the pinned development shell. Run the snapshot checker and importer tests through the owner release verifier:

```sh
nix develop -c sh -eu -c 'rustc --edition=2024 tools/release-verify.rs -o /tmp/basalt-release-verify; /tmp/basalt-release-verify'
nix flake check --no-update-lock-file
nix develop .#enforcement-compat -c cargo test --locked --no-default-features --lib --test ucan_enforcement
```

Use an empty owned Cargo target when checking a changed vendored snapshot.
For the focused API check, run `cargo test --locked --no-default-features --doc observe_holder` in the development shell.

## Non-claims and lifecycle

An observation does not refresh Basalt policy, establish backend authenticity or freshness, or prove continuing holder-key custody.
It does not authorize a later effect or atomic resource release.
Consumers still need current policy and effect-time checks, monotonic observation history, and any stricter deadline policy.

No real issuer was accessed. No grant, training, activation, resource release, or promotion occurred.
Animus and Neural Stream consumer pins remain unchanged at this checkpoint.

All four implementation and evidence tasks are complete.
The final task records evidence for the lifecycle handoff rather than making archive its own precondition.

The corrected strict validation and task receipts passed with no issues or findings:
`basalt-recheck-validation-v3.json` and `basalt-recheck-tasks-v3.json`.
The sync receipt records all three requirements applied to `.cairn/specs/holder-observation/spec.md`.
The archive receipt records the move to `.cairn/archive/1970-01-01-adopt-holder-observation/`.
The epoch prefix is the CLI's deterministic label, not the execution date.
Post-archive strict validation reports zero active changes, no findings, and `valid: true`.

These observed operations are retained in `basalt-recheck-sync-v1.json`,
`basalt-recheck-archive-v1.json`, and `basalt-recheck-archived-validation-v1.json`.
The evidence/archive commit changes lifecycle and research evidence only; product source remains the qualified source above.
