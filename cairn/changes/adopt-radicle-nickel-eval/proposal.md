## Why

Lattice constructs `nickel-lang-core` programs directly in configuration and doctor paths, while Seaglass already owns a generic evaluator surface. The proposed independent `nickel-eval` repository can remove duplicate evaluator and diagnostic mechanics, but only after its version cohort, licensing, Radicle publication, and Seaglass compatibility are accepted.

Lattice must retain configuration discovery, product contracts, target decoding, fallback policy, safe operator diagnostics, and all mutation authority.

## What Changes

- Inventory every direct Nickel call, import-path rule, contract input, output target, diagnostic, and fallback path in Lattice.
- Verify the producer's canonical Radicle RID, exact revision, licensing, provenance, package features, and supported Nickel cohort.
- Build a thin Lattice shell adapter over the shared evaluator without moving product contracts or defaults.
- Dual-run positive and negative configuration, workflow, orchestration, template, and doctor fixtures.
- Align Cargo and Nix on one exact Radicle-backed source and regenerate locks only through owning tools.
- Remove duplicate evaluator mechanics only after parity and rollback pass.

## Success Criteria

- Current accepted values and stable error classes remain compatible for every inventoried call site.
- Malformed Nickel, contract failures, missing imports, invalid JSON export, bounded diagnostic redaction, and unsupported evaluator versions fail closed.
- No product contract, workflow type, template policy, secret, or fallback rule enters the shared package.
- Cargo and Nix resolve one immutable Radicle revision without GitHub or sibling fallback.
- Lattice can revert to its prior direct evaluator implementation through one reviewed VCS rollback.

## Non-Goals

- Replacing `nickel-export`, changing authored formats, or moving generated artifact freshness into runtime evaluation.
- Claiming evaluator versions are semantically equivalent.
- Changing current product validation or fallback policy as an unreviewed side effect.
- Publishing or pushing the shared repository from Lattice.

## Impact

- **Source**: configuration and doctor evaluator shells plus focused workflow/orchestration loaders.
- **Dependencies**: exact Radicle source in Cargo and Nix; generated lock updates.
- **Testing**: call-site inventory, dual-run fixtures, redaction checks, source checks, Octet, Cairn, and Nix.
