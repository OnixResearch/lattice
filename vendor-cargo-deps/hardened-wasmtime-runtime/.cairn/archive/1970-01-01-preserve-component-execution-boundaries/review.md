# Producer correction review — execution boundaries

Self-review of the producer mechanism, not independent human completion authority.

- The v4 API exposes initialization yields separately from callback yields and pauses exactly once after successful initialization. Its next poll resumes the same future/store before export lookup or invocation; no input replay, fuel refill or synthetic callback boundary is introduced.
- Checkpoint fuel is the store's exact cumulative readback. Ordinals count only actual yielded polls; automatic post-return remains in Invocation. Failed initialization has no successful checkpoint. Cancellation at the checkpoint drops ownership and releases the existing engine permit without running the export.
- Both memory ceilings are explicit positive inputs. Per-memory StoreLimits and cumulative remaining bytes enforce different supplied meanings; no clamping or implicit multiplication occurs inside the producer. The frozen document reader retains its prior aggregate policy by mapping its existing memory value to both fields.
- Actual module/component executions cover independent cap refusals, two-memory success, asymmetric admission, fixed-reader compatibility, start work, callback fencing, cancellation, exhaustion, cleanup and exact successful synchronous/sliced fuel. The separate public-API smoke exercised checkpoint and allocation behavior outside the test runner.
- Wasmtime remains 45.0.0. No product DTO, effect world, authority rule, guest component async, host timer or local-engine fallback was added. Existing consumer source pins remain immutable until their owners migrate explicitly.
- Molten owns its unchanged admitted profile, initialization-draining policy, callback ledger and canonical receipts. Publication of this experimental producer mechanism enables independent parity work; it does not establish production actor/extension installation, matching Mantle bundles, operated product deadlines, source trust or release readiness.

The consumer handoff requires publishing canonical Radicle main, pinning the exact resulting revision in Cargo/Nix and dependency evidence, regenerating source hashes/plans, and rerunning the frozen Molten corpus without a source-path override. Consumer verification belongs in Molten's active adoption package; this producer archive does not claim it in advance.
