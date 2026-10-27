# Holder observation adoption

## Ownership and source

UCAN owns digest/time binding, signature/proof verification, revocation observation, and original-operation caveat checks. Basalt consumes the published exact revision through its existing reviewed snapshot importer. The importer reads committed source, preserves third-party notices, and produces a BLAKE3 manifest. No ambient sibling worktree becomes a runtime dependency.

## Composition

A method on `LiveUcanEnforcement` borrows its existing holder authorization and forwards explicit compact-token, verification, and non-consuming policy inputs to UCAN. The result is UCAN's borrowed observation. Basalt creates no replacement holder authorization, replay context, copied verifier, or new authority receipt.

The method is named `observe_holder` deliberately. It does not refresh Basalt's policy reference or decision. A consumer must obtain current Basalt policy and application authority separately. Existing final replay admission and process-execution default denial stay unchanged.

## Evidence

Run the original enforcement baseline before changes. Check exact-source parity, snapshot negative controls, generated cohort freshness, default and minimal enforcement tests, compile-fail observation controls, and the complete release verifier. Run Nix checks and inspect Cairn JSON verdicts, not only process exit codes. Record observed results before checking tasks or publishing.

## Non-claims

Borrowed observations do not establish independent custody, continuous revocation freshness, backend authenticity, continuing holder-key possession, workload fencing, atomic release, or effect execution. Never use a receipt or copied timestamp to supply those missing facts.
