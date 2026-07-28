# Durable orchestration examples

`llm-review-loop.ncl` binds the exact checked-in `llm-operating-graph` workflow revision to a durable orchestration policy.

- Every round is one normal acyclic Seaglass-admitted workflow run.
- A successful round waits for a hash-bound successor proposal.
- `operator:local` must approve the exact proposal and current policy/revision epoch.
- Round, agent-invocation, token, and duration budgets are reserved before dispatch.
- Schedule and webhook trigger identities are admitted with scoped idempotency keys.
- Replay folds persisted orchestration facts without rerunning agents or effects.
- One explicit operator-authorized retry is available only after an exact owned terminal failure. It uses a fresh run identity, dispatch identity, and budget.

Regenerate after changing the workflow:

```bash
cargo run --example generate_llm_orchestration
```

This example does not grant ambient mutation authority to agent output.

The retry bound does not authorize these retries:

- automatic
- pending-unobserved
- abandoned
- foreign-owner
- provider

The example does not prove kill safety, remote deduplication, exactly-once effects, provider billing, token counts, model truth, full n8n parity, production readiness, or external event completeness.
