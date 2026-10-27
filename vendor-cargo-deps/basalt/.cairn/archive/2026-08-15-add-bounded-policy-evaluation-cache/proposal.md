## Why

Basalt evaluates policy decisions with UCAN capability context. Repeated
evaluation of the same inputs under the same policy recomputes the same
decision without a bounded retention contract.

An unbounded policy-result store grows without limit and creates stale-decision
hazards under high throughput.

Basalt should own a bounded policy-evaluation cache. Retention decisions belong
in a pure deterministic core. Keys are BLAKE3 digests over the policy-source
identity, the normalized decision inputs, and the capability-context
projection.

## What Changes

- Add a pure decision core for the policy-evaluation cache: canonical key
  projection, eviction selection, promotion disposition, watermark trim plans,
  and deferred-release plans from supplied values.
- Add a bounded store that executes core decisions and releases results after
  the guard releases.
- Key every entry by BLAKE3 digest over the policy-source identity, normalized
  inputs, and capability-context projection.
- Require an explicit capacity. Add no implicit default bound.
- Record the cross-repo contract: the retention policy follows the
  `bounded-cache` contract, and the store can later adopt a published
  `bounded-cache` revision.

## Impact

- **Files**: decision core module, bounded store, tests, and this lifecycle
  package.
- **Testing**: workspace tests, policy freshness, Clippy, formatting, Cairn
  validation and gates, and `nix flake check`.

## Dependencies

New sibling component: `bounded-cache` (consumed by an exact published Git
revision once available; retained as a policy reference until then).

Upstream consumer: UCAN verification cache retention contract
(`ucan` repo, `add-bounded-verification-cache`).

External reference: `adanil-code/LRUHashTable`
(<https://github.com/adanil-code/LRUHashTable>) under Apache-2.0.

## Non-goals

- Do not change policy semantics. The cache is advisory only.
- Do not add sibling-path dependencies.
- Do not read clocks in the decision core.
- Do not claim that a cache hit proves policy correctness or revocation
  freshness. Freshness stays consumer-owned.
