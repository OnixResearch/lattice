## Design

One shared Rust constructor emits a fixed typed Nickel DAG. Planner and generator are production command-backed Agent nodes; the evaluator is a real shell build/test role rather than another model. The selected producer is a finite local rule implementation for one greeting task, not a stubbed runtime or claimed learned generator. It intentionally emits one actual unit-test failure before using the retained verdict, or an explicitly selected always-pass/always-fail control.

Preparation writes an inert, operator-reviewed snapshot. The compiled host retains the exact generated Nickel source in `frozen.json`, source identities for all three scripts, exact runtime recipe and workflow/definition identities. Starting requires explicit local-effect approval. Current standalone operator authority remains the sole effect grant route. Generated output cannot approve orchestration changes; fixed-profile candidate admission refuses any unapproved revision or added topology.

The example host uses the product snapshot executor and existing orchestration fact identities, transitions, durable store, reservation/settlement ledger and hash-linked history. It adds no scheduler, retry counter or model API. A round writes only its active workspace; after the owned terminal result the host retains the actual specification, generated Rust source, evaluator verdict, diagnostics and immutable references under `round-N/`. New run/dispatch identities never reset total accounting.

Duration settlement conservatively charges the admitted upper bound, not invented measured tokens, compute or billing. A monotonic whole-loop deadline includes loop admission/dispatch/evaluation work and a per-round watchdog requests owned cancellation, then awaits observation rather than dropping the executor future. Cancellation intent, actual owned termination and deadline reason are separately retained before settlement. Missing/unobserved terminal evidence blocks automatic retry.

The live completion callsite emits only `RunFinishedObserved`, carrying the exact dispatch/run binding, actual executor cancellation observation, supplied charge and usage basis. It atomically settles before closing ownership. Historical completion actions remain replay-only. Real-command fixtures record cancellation after observed success/failure but before settlement: the actual completion wins, failed-plus-cancel-request cannot retry, and duplicate settlement cannot charge again. A separate real active-process fixture checks Cancelled with an empty reservation ledger and retained total usage.

The base profile selects fixed saved-workflow snapshots, not `SubworkflowNode` invocation or constructor inlining. The dedicated child negative uses the real saved-child runtime to show that referencing an effectful child is not authority for that child. Regenerated source is checked before any subsequent dispatch; rejected candidates retain their bytes and the earlier results stay bound to the original stored revision.

## Optional arm prerequisite

The full Lattice #4 issue and comments were read, along with [inference-lab #1](https://github.com/OnixResearch/inference-lab/issues/1) and [Animus #1](https://github.com/OnixResearch/animus/issues/1). Inference-lab's issue explicitly distinguishes Noble's completed deterministic control from its still-uncompleted real-model comparison and qualification. Neither issue supplies an independently checked qualified masked producer for this example. No model/backend is selected, no adapter API is invented, and the optional masked/diffusion arm remains OFF. The local `repair` control is ordinary finite rule-based correction after a real test, not that optional arm.

## Trust boundary

The script/profile directories and host are operator-controlled. Exact per-node Agent and Process permissions do not establish OS isolation for arbitrary code. These fixtures do not claim remote cancellation, external exactly-once execution, independently witnessed history, model quality or general workflow feedback semantics.
