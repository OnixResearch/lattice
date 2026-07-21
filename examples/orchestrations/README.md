# Durable orchestration examples

`llm-review-loop.ncl` binds the exact checked-in `llm-operating-graph` workflow revision to a durable orchestration policy.

- Every round is one normal acyclic Seaglass-admitted workflow run.
- A successful round waits for a hash-bound successor proposal.
- `operator:local` must approve the exact proposal and current policy/revision epoch.
- Round, agent-invocation, token, and duration budgets are reserved before dispatch.
- Schedule and webhook trigger identities are admitted with scoped idempotency keys.
- Replay folds persisted orchestration facts without rerunning agents or effects.
- One explicit operator-authorized retry is available only after an exact owned terminal failure; it uses fresh run/dispatch identity and fresh budget.

Regenerate after changing the workflow:

```bash
cargo run --example generate_llm_orchestration
```

This example does not grant agent outputs ambient mutation authority. Its retry bound does not authorize automatic, pending-unobserved, abandoned, foreign-owner, or provider retries and does not claim arbitrary kill safety, remote effect deduplication, remote exactly-once effects, provider billing or token truth, model truth, full n8n parity, production readiness, or externally anchored event completeness.
