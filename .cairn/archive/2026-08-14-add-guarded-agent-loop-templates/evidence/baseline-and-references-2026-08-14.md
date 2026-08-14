# Guarded agent template baseline and reference review

Date: 2026-08-14

## Dependency

The archived `add-ui-workflow-template-catalog` change is present at `origin/main` revision `41ccc736944adcbf1740d3183b4bc7b9b0a002aa`. It supplies typed local bundles, effect-free preview, exact save, explicit activation, compatibility checks, and identity-bound assets.

## Pre-change baseline

The baseline used the clean guarded-change worktree and `target/guarded-loop-baseline`. The focused tests passed before guarded implementation for:

- agent nodes and their no-fallback routes;
- bounded local process execution;
- orchestration core, format, shell, store, approval, recovery, and replay;
- workflow templates and examples;
- the full library suite.

No baseline failure was assigned to this change.

The accepted Octet receipts before implementation were:

- `lattice`: `b3:ea0aec4872a3d77db6bdc0cdb39fec49cbc9e4fce2e5fdcb6c3abeef396984bf`
- `lattice-core`: `b3:7e0b9fe7f99811d14db2e1f91e3c3090e336d18c148234071298ec9f4ab3f236`

These receipts are comparison evidence. They do not permit stale or new findings.

## Aspen review

Reviewed path: `../aspen/scripts/octet-burndown.scm`

Reviewed BLAKE3: `791fba3ab3c78ededa9406b57a3d90163984541d453f183f632d8326e3aa5322`

Ported invariants:

- take a no-disabled finding baseline;
- run one candidate action;
- validate before acceptance;
- require a strict finding-count decrease;
- restore observed policy bytes before continuation.

Rejected mechanisms:

- the opaque outer loop;
- optional automatic commit and push;
- user-selected shell command text;
- sibling-path runtime loading.

## mc review

Reviewed path: `../../mc/scripts/ralph-drain-cairns.scm`

Reviewed source commit: `7f36751aa4aadcbb048442c06dfbcf2addec4aaa`

Reviewed BLAKE3: `94ff7d3c3c527d272782c1bbd84b79da557fc6392af9dfbbaf7a21cb03f703b9`

The current sibling file has BLAKE3 `1439827952a1f7cd0afd84d61e063d050216056afb4c77fcc460bfc20eb4facc`. Commit `2f9edceed14c5ade5147cbc7b268a8a8b05c8652` changed Cairn paths to `.cairn/changes`. This change does not silently replace the reviewed source identity.

Ported invariants:

- deterministic bounded active-change discovery;
- explicit requested selection or sorted-first selection;
- one selected change per sequential agent call;
- plan-first operator visibility;
- positive and negative deterministic tests.

Rejected mechanisms:

- an opaque Steel loop;
- multiple agent calls inside one Lattice round;
- sibling-path runtime loading;
- semantic-equivalence claims.

## Non-claim

Both scripts are reviewed design references only. Lattice does not vendor them, execute them, track later revisions, or transfer their evidence claims.
