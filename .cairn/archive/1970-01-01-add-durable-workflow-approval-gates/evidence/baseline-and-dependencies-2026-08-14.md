# Approval-gate dependency and baseline evidence

## A1 orchestration identity prerequisite

Lattice `origin/main` contained archived change `type-orchestration-identities` before this worktree started.

The approval implementation uses its nominal `WorkflowRevision` type. It does not change existing orchestration proposal or approval record types.

## A2 pinned Choregraph suspension evidence

Lattice pins Choregraph revision `0b2000edaf86d5a7a8601246afbc0d526f4a2a07`.

The local ancestor check passed. The pinned revision is an ancestor of the reviewed local Choregraph checkout.

The producer change is archived at:

`choregraph/.cairn/archive/1970-01-01-model-durable-suspension-transitions`

The exact lifecycle profile is:

`choregraph.lifecycle.durable-suspension.v1`

The Lattice product suspension class is:

`lattice/workflow-approval`

BLAKE3 identities from the pinned revision are:

| Artifact | BLAKE3 |
| --- | --- |
| `fixtures/nickel/evaluated/durable-suspension.json` | `e09e52b6e9abd392c1ba84f2617748a90f31ae9d0d3183bc1460a15c796c113d` |
| `evidence/single-step-proof.ncl` | `1c2dd09ed2b955ebda453b3537b3b679bcff56e29667b5f71a8d055e730f611b` |
| `evidence/trace-replay-proof.ncl` | `5b6d6f359fa317c82fa2b0ce8c3bde4c26ff2af79bf17565e8203babc9af56a7` |
| `crates/choregraph-seaglass/tests/lattice-suspension.rs` | `6a869e195d0fdaba9d6c060d22648a92415d48177a55610679c711744993de70` |

The exact pinned downstream command passed:

```text
CARGO_TARGET_DIR=/home/brittonr/git/OnixResearch/choregraph/target/lattice-approval-pin \
  nix develop -c cargo test -p choregraph-seaglass --test lattice-suspension -- --nocapture
```

Result: two tests passed. They cover suspended wait, matching resume, mismatched denial, and deterministic replay.

Choregraph proves only its bounded graph transition and replay properties. It does not prove Lattice authority, operator identity, persistence, expiry, credential handling, downstream effects, or release eligibility.

## A3 pre-change baseline

The focused pre-change baseline passed before approval implementation.

It covered workflow execution, runner behavior, headless runs, store ownership, crash boundaries, recovery, replay, credentials, orchestration, identity compatibility, and TUI behavior.

The pre-change repository command also passed:

```text
CARGO_TARGET_DIR=target/workflow-approval scripts/check.rs
```

No baseline failure was carried into the approval implementation.

## A4 approval-semantic inventory

The review inventory found these distinct approval surfaces:

- Agent orchestration owns `RevisionProposal` approval and rejection.
- Workflow execution now owns `ApprovalRequest`, `ApprovalDecision`, `ResumeCommand`, and approval receipts.
- Choregraph owns only opaque suspension class and reference facts.
- Basalt/UCAN adaptation consumes already verified facts. It does not accept raw tokens.
- The local profile uses an explicit operator command and exact actor policy.
- Redb stores workflow approval requests in `workflow_approval_requests_v1`.
- Redb maps one run to its active gate through `workflow_approval_run_index_v1`.
- Run history, detail records, replay events, and the active approval request change in one write transaction.
- CLI actions are `approvals list`, `approvals show`, `approvals receipt`, `approvals decide`, and `workflow resume`.
- TUI state has safe inspection plus separate typed decision and resume action methods.
- Approval events are `suspended`, `approved`, `rejected`, `expired`, `resume_claimed`, `resume_signalled`, `resumed`, and `resume_uncertain`.
- Choregraph graph events remain separate from product approval events.
- Safe views and receipts omit workflow values, node outputs, reasons, raw tokens, and resolved secrets.

No adapter converts an orchestration revision approval into workflow-gate authority. No workflow-gate decision changes an orchestration proposal record.
