let Contracts = import "traceability-inventory-contracts.ncl" in

let core = { path = "tests/test_orchestration_core.rs", blake3 = "6213a979d7a99503cfecb5bdaf58bc8a0384310457bb505c498144c1936eac3b" } in
let store = { path = "tests/test_orchestration_store.rs", blake3 = "98cfa37570536d3232c89005035f9e7ac36e149293df67461854e7338884cd54" } in
let format = { path = "tests/test_orchestration_format.rs", blake3 = "f4bf1879b7da1e633c917c220fbac6f820535d3ffa41e7d93688af7325419c5a" } in
let shell = { path = "tests/test_orchestration_shell.rs", blake3 = "51074190b3ba190631e206776fbcb629b26e98ad672ff50a95188bb4b5a241e1" } in
let cli = { path = "tests/test_orchestration_cli.rs", blake3 = "a57e3674764e3fffa6b83f75169da51ba89c5ca16618d75bdbaee76f89eb4050" } in
let run_store = { path = "tests/test_store.rs", blake3 = "afb9eac879e9e92de19379596f10603715650c3c88dc40de92a0a950ba6cee4d" } in
let exact = "exact_maintained_evidence" in
let partial = "partial_evidence_only" in
let row = fun id decision rationale refs => {
  requirement_id = id,
  classification = decision,
  rationale_code = rationale,
  evidence = refs,
} in
{
  schema_version = "lattice.traceability-debt-inventory.v1",
  hash_algorithm = "BLAKE3",
  expected_missing_count = 261,
  expected_missing_hash = "de87e38b8b3298301d5872c62c6dc887eba0a3bc2ce9f1556996a470fc3ac902",
  reviewed = [
    row "lattice.agent_orchestration.authority.scenario.ambient-denied" partial "gap.inert-authority-shaped-output-unasserted" [core],
    row "lattice.agent_orchestration.authority.scenario.approved" exact "assertions.authorized-successor-lineage" [core, shell],
    row "lattice.agent_orchestration.authority.scenario.stale" exact "assertions.stale-approval-rejected" [core],
    row "lattice.agent_orchestration.authority.scenario.validation" exact "assertions.approval-cannot-bypass-validation" [core],
    row "lattice.agent_orchestration.budgets.scenario.block" exact "assertions.missing-and-exhausted-bounds-block" [core],
    row "lattice.agent_orchestration.budgets.scenario.concurrent" exact "assertions.concurrent-reservation-conflict" [store],
    row "lattice.agent_orchestration.budgets.scenario.overage" partial "gap.overage-breach-and-nonclaim-unasserted" [core],
    row "lattice.agent_orchestration.budgets.scenario.reserve" exact "assertions.reservation-precedes-dispatch" [core, store],
    row "lattice.agent_orchestration.cli.scenario.invalid" exact "assertions.invalid-cli-preserves-state" [cli],
    row "lattice.agent_orchestration.cli.scenario.status" partial "gap.complete-actionable-safe-status-unasserted" [cli],
    row "lattice.agent_orchestration.controls.scenario.cancel" exact "assertions.cancel-request-separate-from-outcome" [core, cli],
    row "lattice.agent_orchestration.controls.scenario.pause" exact "assertions.pause-blocks-until-resume" [core, cli],
    row "lattice.agent_orchestration.core.scenario.complete" exact "assertions.terminal-objective-completes" [core],
    row "lattice.agent_orchestration.core.scenario.terminal-mutation" exact "assertions.terminal-mutation-rejected" [core],
    row "lattice.agent_orchestration.durability.scenario.duplicate" exact "assertions.duplicate-fact-idempotent" [core],
    row "lattice.agent_orchestration.durability.scenario.external-effects" partial "gap.remote-exactly-once-rendered-boundary-unasserted" [shell],
    row "lattice.agent_orchestration.durability.scenario.reconcile" exact "assertions.recovery-reconciles-without-redispatch" [shell],
    row "lattice.agent_orchestration.events.scenario.non-claim" partial "gap.rendered-local-chain-nonclaim-unasserted" [core],
    row "lattice.agent_orchestration.events.scenario.replay" exact "assertions.replay-reconstructs-without-effects" [core],
    row "lattice.agent_orchestration.events.scenario.tampered" exact "assertions.tampered-ledger-rejected" [core, store],
    row "lattice.agent_orchestration.format.scenario.n8n-boundary" exact "assertions.n8n-export-rejected" [format],
    row "lattice.agent_orchestration.format.scenario.round-trip" exact "assertions.typed-nickel-roundtrip" [format],
    row "lattice.agent_orchestration.format.scenario.secret" exact "assertions.plaintext-secret-rejected" [format],
    row "lattice.agent_orchestration.model.scenario.revision-drift" exact "assertions.revision-drift-before-instance" [core, shell],
    row "lattice.agent_orchestration.model.scenario.valid" exact "assertions.valid-definition-binds-identities" [core],
    row "lattice.agent_orchestration.rounds.scenario.execute" exact "assertions.seaglass-round-persists-replay" [shell],
    row "lattice.agent_orchestration.rounds.scenario.hidden-cycle" partial "gap.feedback-cycle-continuation-unasserted" [shell],
    row "lattice.agent_orchestration.run_identity.scenario.concurrent" exact "assertions.concurrent-run-ids-unique" [run_store],
    row "lattice.agent_orchestration.run_identity.scenario.duplicate" exact "assertions.duplicate-run-preserves-first" [run_store],
    row "lattice.agent_orchestration.run_identity.scenario.foreign" exact "assertions.foreign-owner-fails-closed" [shell, run_store],
    row "lattice.agent_orchestration.run_identity.scenario.match" exact "assertions.exact-owner-reconciles" [shell],
    row "lattice.agent_orchestration.triggers.scenario.duplicate" exact "assertions.duplicate-trigger-reuses-instance" [core, store, shell],
    row "lattice.agent_orchestration.triggers.scenario.start" exact "assertions.valid-trigger-starts-once" [core, shell],
    row "lattice.agent_orchestration.unobserved_abandon.scenario.acknowledged" exact "assertions.acknowledged-abandon-consumes-bound" [core],
    row "lattice.agent_orchestration.unobserved_abandon.scenario.authority" exact "assertions.abandon-requires-cancel-and-authority" [core, shell],
    row "lattice.agent_orchestration.unobserved_abandon.scenario.identity" exact "assertions.repeat-and-mismatch-fail-closed" [core],
    row "lattice.agent_orchestration.unobserved_abandon.scenario.observed" exact "assertions.observed-run-uses-recovery" [shell],
    row "lattice.agent_orchestration.unobserved_abandon.scenario.planned" exact "assertions.planned-abandon-releases-reservation" [core],
    row "lattice.agent_orchestration.unobserved_abandon.scenario.replay" exact "assertions.abandonment-replay-preserves-nonclaims" [core, cli],
  ],
  non_claims = [
    "inventory rows do not satisfy requirements or create Tracey evidence",
    "unassessed does not mean behavior or evidence is absent",
    "partial evidence is not a near-complete correctness claim",
    "no exact evidence identified is bounded to the reviewed maintained roots and is not proof of global absence",
    "a valid inventory does not prove semantic correctness, complete traceability, release readiness, or production readiness",
  ],
} | Contracts.ReviewPolicy
