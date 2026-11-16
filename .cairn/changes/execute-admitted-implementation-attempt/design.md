## Decisions

A separate implementation-attempt CLI is explicitly selected by the coordinator. A strict bounded request binds the complete existing envelope, exact source snapshot, worker and workspace provider profile. The CLI validates all bindings before persisting intent and invoking the existing bounded OMP route. Persist a one-shot intent by exclusive creation with fsync before worker execution; an existing intent always enters read-only candidate reconciliation. The Store remains owner of candidate receipts/artifacts; the CLI never treats absence after possible dispatch as permission to rerun.

Cancellation is propagated to the existing process group cancellation flag. Unknown teardown retains workspace ownership and cannot yield a candidate. Raw OMP events and compatibility identities are retained separately. Independent acceptance and Git commit materialization belong to Kiln. Configuration is operator-supplied typed Nickel exported to strict request JSON; no prompt text grants authority.

Cairn artifact authoring uses its own versioned source-kind tag and exact native stage context; it is never relabeled as a ready coding task. Existing issue/task identity tags remain unchanged. A native `export-candidate` command validates the frozen request and retained execution/result, captures the bounded ordinary-file snapshot, matches its identity to the actual host receipt, and writes a new role-readable export containing the unchanged native receipt and observation. Replay validates the same completed export; missing, changed or partial state is refused. Independent candidate materialization, verifier authority and acceptance stay in Kiln.

## Verification

Parent runs binding, duplicate intent, crash/recovery and cancellation fixtures after integration. Live OMP needs reviewed executable/provenance, qualified bubblewrap kernel policy, model-service endpoint and explicit operator approval. No live checks or provider actions are run during editing.
