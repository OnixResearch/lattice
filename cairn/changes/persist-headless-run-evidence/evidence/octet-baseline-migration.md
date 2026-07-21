# Octet Baseline Migration

The shared run-history refactor and provider-boundary parser initially introduced concrete Octet findings. They were corrected by:

- placing shared run logic under `src/run/history.rs` to preserve searchable module ownership and the root module-file profile;
- moving inline-reasoning parsing into `src/openai/reasoning.rs` to keep `core.rs` below the file-length bound;
- using qualified owner paths instead of non-trait imports;
- decomposing marker predicates, naming booleans positively, reserving output capacity, and restoring assertion density.

After those corrections, semantic comparison with the self-tested migration tool reported:

```text
old=1169 candidate=1166 matched=1166 stale=3 new=0
STALE: assertion_density src/headless.rs:244 function `run_saved_workflow_with_executor_factory` has 0 assertion(s) in 22 lines (need 2+)
STALE: non_trait_imports src/tui/app.rs:58 non-trait import `RunOutputDetail` hides the owner path; prefer the qualified path at the use site
STALE: assertion_density src/tui/app.rs:477 function `run_node_detail_from_step` has 0 assertion(s) in 49 lines (need 2+)
```

Those findings were removed by the implemented refactor rather than accepted as new debt. The migrated baseline passes with 916 accepted/current findings, zero new, zero stale, 1,166 total findings, and hash `b3:1aeb6f42e5bbb4bf92fbb1b16fbc2a05fe03e997ac30a3b7ba2b00b63ffab8f0`.
