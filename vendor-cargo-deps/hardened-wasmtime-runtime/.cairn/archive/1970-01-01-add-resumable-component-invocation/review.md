# Implementation review — resumable component invocation

Self-review of the producer boundary; not independent or human completion authority.

- `start_sliced` rejects oversized owned input and invalid quanta before retaining work. It reserves the shared engine permit before returning even an unpolled handle. The ordinary synchronous path keeps its borrowed permit and does not allocate an owned guard.
- One pinned future owns prepared state, input and the concurrency guard. Each call polls it once; completion removes the future before returning output. Cancellation drops the same future/store/guard. No guest restart, extra input copy, ambient executor, timer thread or internal slice history was added.
- The store is initialized once with total fuel, aggregate memory/transfer limits and the engine-bound epoch deadline. Resumption does not reset these bounds. Public pending observations report only ordinal and configured quantum; exact fuel comes from final store readback.
- Phase advances before linking, instantiation and export lookup/invocation. Wasmtime's automatic post-return remains part of Invocation; the result does not invent a separately observable cleanup stage or parse diagnostics to infer phase.
- Engine clones share the same atomic concurrency counter. Both borrowed and owned admission use the existing concurrency module's single reservation operation; drop releases ownership on terminal paths and cancellation.
- Actual component regressions and the external public API smoke exercise the intended behavior. The source-policy findings were resolved by qualified owner paths, checked arithmetic, bounded source owners and separating fixture data, not fake assertions or allowances.
- The schema cutover is explicit (`hwr.shell.v3`). External consumers retain their immutable prior dependency until independent parity and product migration pass. README and source inventory describe ownership, epoch driving, cancellation, configured versus measured fuel and non-claims.

No unresolved producer implementation finding remains from this review. The documented failed-run memory/fuel observation remains a consumer compatibility input, not proof of universal synchronous/sliced parity. Molten materialization, authority, scheduler, actor/extension composition and receipt adoption are not implemented or certified by this producer change.
