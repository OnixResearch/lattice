# Durable orchestration examples

`llm-review-loop.ncl` binds the exact checked-in `llm-operating-graph` workflow revision to a durable orchestration policy.

- Every round is one normal acyclic Seaglass-admitted workflow run.
- A successful round waits for a hash-bound successor proposal.
- `operator:local` must approve the exact proposal and current policy/revision epoch.
- Round, agent-invocation, token, and duration budgets are reserved before dispatch.
- Schedule and webhook trigger identities are admitted with scoped idempotency keys.
- Replay folds persisted orchestration facts without rerunning agents or effects.

Regenerate after changing the workflow:

```bash
cargo run --example generate_llm_orchestration
```

This example does not grant agent outputs ambient mutation authority and does not claim remote exactly-once effects, provider billing truth, model truth, full n8n parity, or externally anchored event completeness.
