## Context

Lattice consumes Cairn in three places: the native binary (`LATTICE_CAIRN_BIN` in `scripts/check.rs`, the dev shell and the `nextest` check), the upstream policy snapshots that `cairn-policy/default.ncl` overlays, and the generated policy that every Cairn command loads. The previous pin read consumer-selected gates but lived only on a release branch.

## Decisions

### Decision: Pin Cairn `main` and refresh the snapshots from the same revision

**Choice:** Pin `317d3ff648f01a5ea1572e6126f4dd71bc7afa99` and copy `cairn-policy/contracts.ncl` and `cairn-policy/default.ncl` from it verbatim. Their imports, `lifecycle-store-contracts.ncl` and `nominal-evidence-identities.ncl`, are already identical at that revision.

**Rationale:** The new binary refuses the policy exported from the `5e1fe08d` snapshots (`policy has invalid field generated_agent_workflow_policy`). Re-snapshotting keeps one producer revision for the binary and the policy structure. A Lattice-local patch would fork the upstream snapshot. The regenerated policy differs from the old one only at the JSON paths that also changed in Cairn's own generated policy between those revisions. Lattice's name, traceability profiles, release selection and refresh command are unchanged.

### Decision: Keep Lattice's release-selection contract as the first refusal

**Choice:** Keep `release-selection-contracts.ncl` unchanged. It already refuses an absent, empty, non-array, blank, duplicated or non-string gate list and a non-object `release_readiness_policy` at export, so a malformed list never reaches Cairn's new `invalid-consumer-selected-gates` path.

**Rationale:** Lattice's predicate matches Cairn's rule (non-empty array of unique strings that are not blank after trimming) and also requires the list to be present.

### Decision: Declare each Cairn subcommand's options

**Choice:** The test fixture keeps a table of the options each Cairn subcommand accepts, taken from the pinned `cairn --help`, and refuses an undeclared subcommand. The dev-shell wrapper injects `--policy` only into subcommands that accept it.

**Rationale:** The pinned binary refuses unknown options. The old binary ignored them, so `init`, `status` and `preserves-corpus` never used the appended policy. Dropping those options keeps today's behavior, and a new subcommand forces an explicit choice.

## Risks / Trade-offs

- The upstream snapshots now carry Cairn's newer generated-workflow guidance (`command_catalog`, `entry`, the `readiness` class and `drain-plan-readiness`). These are upstream policy text, not Lattice policy, and no Lattice rule is loosened.
- A policy that Cairn cannot load still yields a `maintained-local-gates` summary in the release-readiness receipt, alongside failed validation checks. That is Cairn behavior, outside this change.
