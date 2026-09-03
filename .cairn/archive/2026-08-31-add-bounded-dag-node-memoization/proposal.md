## Why

Lattice executes typed DAG workflows and re-derives node results across repeated runs. Repeating immutable node results without a bounded retention contract grows memory without limit.

The stack owns the product-neutral `bounded-cache` retention contract. Lattice should memoize immutable node results under that contract. Retention decisions belong in a pure core. Keys are BLAKE3 digests over canonical node-request projections.

## What Changes

- Add a pure retention decision core: canonical key projection, eviction selection, promotion disposition, watermark trim plans, and deferred-release plans from supplied values.
- Add a bounded node-result store that executes core decisions and releases results after the guard releases.
- Key every entry by BLAKE3 digest. Require an explicit capacity. Add no implicit default bound.
- Record the cross-repo contract: the cache policy follows the `bounded-cache` retention contract, and the store can later adopt a published `bounded-cache` revision.

## Impact

- **Files**: decision core module, bounded node-result store, tests, and this lifecycle package.
- **Testing**: workspace tests, Clippy, formatting, Cairn validation and gates, and Nix checks.

## Dependencies

New sibling component: `bounded-cache` (consumed by an exact published Git revision once available; retained as a policy reference until then).

External reference: `adanil-code/LRUHashTable` under Apache-2.0.

## Non-goals

- Do not change workflow semantics. The cache is advisory only.
- Do not move scheduler authority into the cache.
- Do not add sibling-path dependencies.
- Do not read clocks in the core.
- Do not claim that a cache hit proves node correctness or run freshness.
