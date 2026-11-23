## Why

Issue #4 distinguishes finite Nickel construction from evaluator-driven runtime repetition. The old fixture ran one build-only DAG and used AwaitProposal; it did not establish a runtime generate/build/test retry or retain three round handoffs.

## What Changes

- Share a fixed immutable planner/generator/evaluator graph and a RunSucceeded definition with a two-round, four-Agent, four-minute total reservation.
- Add an explicitly approved local example host using existing orchestration durable transitions, store, admission and production snapshot executor. Keep the trusted rule producer distinct from model qualification.
- Retain actual specification, generated Rust source, evaluator verdict, build/test diagnostics and exact round receipts; reject candidate drift before any rejected dispatch.
- Exercise real pass, failed-test retry, iteration/resource exhaustion, invalid next graph, stale revision, denied role/child authority, and owned active cancellation.
- Keep optional masked/diffusion repair disabled without a qualified producer and adapter.

## Impact

Owned surfaces are the bounded loop examples/assets, dedicated integration tests and example documentation. The guarded Failed-to-Exhausted core transition is integrated by the orchestration accounting owner. No worker isolation, generated-topology admission, model-quality, external exactly-once or remote-cancellation claim is added. Verification is pending main-agent integration; this change is not accepted or archived by implementing its fixtures.
