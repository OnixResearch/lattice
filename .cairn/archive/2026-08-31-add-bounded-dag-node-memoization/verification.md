# Verification: add-bounded-dag-node-memoization

r[verify lattice.node_result_cache.projection]
r[verify lattice.node_result_cache.decision]
r[verify lattice.node_result_cache.bound]
r[verify lattice.node_result_cache.deferral]
r[verify lattice.node_result_cache.boundary]
r[verify lattice.node_result_cache.verification]

## Implemented boundary

- The pure core projects canonical node requests to BLAKE3 keys.
- The pure core decides LRU, FIFO, lazy promotion, slot use, watermark trimming, and deferred releases.
- The store requires an explicit capacity and valid watermarks.
- The shell releases evicted values only after it releases the mutex guard.
- Cache misses call the supplied executor and preserve its result or error.
- The cache remains advisory and does not own workflow correctness, freshness, scheduling, or authority.

## Positive evidence

- `nix develop -c cargo test node_result_cache`: 10 focused tests passed.
- `nix develop -c cargo clippy --workspace --all-targets -- -D warnings`: passed.
- `nix develop -c cargo fmt --all -- --check`: passed.
- `nix develop -c octet-precommit-check`: passed with zero new and zero stale findings.
- `nix build .#checks.x86_64-linux.nextest --no-link -L --builders ''`: 1,076 tests passed.
- `nix flake check -L --option secret-key-files '' --builders '' --max-jobs 1`: all checks passed.

## Negative and boundary evidence

- Configuration rejects missing capacity, zero capacity, reversed watermarks, and out-of-range thresholds.
- Tests cover one slot, threshold zero, threshold one hundred, and trimming at both watermarks.
- Key tests reject node, contract, input, and capability-context substitution.
- Destructor-order evidence detects any release while the mutex remains locked.
- Miss tests preserve executor errors and do not create an entry.

## Gate stabilization

Parallel Nix checks exhausted child-process capacity in unrelated process tests. The failing test changed between runs.

The Nextest derivation now uses one test thread. The final full check also used one Nix build job.

This change preserves test coverage. It removes resource-dependent process-spawn failures from the release gate.

## Non-claims

These checks establish bounded memoization behavior for supplied values. They do not prove executor correctness, result freshness, scheduler correctness, or external effect success.
