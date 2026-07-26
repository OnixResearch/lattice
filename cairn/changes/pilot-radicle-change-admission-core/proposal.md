## Why

Lattice already has a pure revision-proposal and approval lifecycle with exact proposal identities, base workflow revisions, epochs, successor validation, authority-policy hashes, actors, and terminal decisions. Seaglass has a related preview and commit lifecycle for semantic patches.

The proposed `change-admission-core` may share stale-check and exact-decision rules. Lattice should adopt it only if a disposable adapter reduces real decision logic while preserving orchestration authority, budgets, events, persistence, and proposal identity compatibility.

## What Changes

- Baseline current proposal, identity, successor validation, approval, rejection, replay, persisted event, and CLI behavior.
- Review the producer's two-consumer pilot and require one licensed immutable Radicle source only if extraction was accepted.
- Map Lattice proposals, successor validation, current epoch/revision, and supplied authorization into opaque shared lifecycle facts.
- Preserve all BLAKE3 product identities and persisted schemas unless a separate versioned migration is approved.
- Keep workflow validation, approver sets, authority hashes, actors, budgets, rounds, persistence, event chains, and mutation in Lattice.
- Admit or reject production cutover from measured simplification and complete positive and negative parity.

## Success Criteria

- Exact current proposals can be admitted only after successor validation and Lattice-supplied authorization.
- Stale epochs or revisions, changed candidates, mismatched proposal IDs, failed validation, unauthorized actors, and repeated terminal decisions fail as they do now.
- Shared preview/admission vocabulary does not imply workflow mutation or authority.
- Adapter code removes material duplicate decisions and remains smaller than retained Lattice logic.
- Any production dependency pins one exact Radicle revision without GitHub or sibling fallback.

## Non-Goals

- Extracting orchestration, the agent-session protocol, workflow diffs, authority policy, budget ledger, event chain, persistence, or rollback.
- Changing current proposal or approval identity bytes.
- Treating successor validation as approval or admission as mutation success.
- Adopting a repository that does not simplify Lattice.

## Impact

- **Source**: `src/orchestration/core.rs` and focused shell/store tests only if the pilot is accepted.
- **Dependencies**: conditional exact Radicle source and generated locks.
- **Testing**: identity, stale decision, authority, replay, persisted compatibility, source, Octet, Cairn, and Nix checks.
