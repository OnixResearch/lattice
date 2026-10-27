# Design: property and fuzz test harness

## Approach

Basalt will add deterministic property-style tests around the functional core. The harness should favor small generators with explicit seeds or checked-in regression fixtures so failures are reproducible through ordinary `cargo test` and the release verifier.

## Invariants

1. Enforcement remains fail-closed: unknown contracts, malformed policy JSON, mismatched contract ids, and unsupported schema versions never allow.
2. Resource boundaries do not collapse prefix siblings into descendants.
3. Multiple capabilities allow only when at least one declared capability exactly authorizes the requested resource and ability.
4. Steel callable descriptors reject opaque/unhashable/effectful shapes unless required metadata and capability declarations are present.
5. Request and receipt hashes are stable for canonical DTOs and change when authority-relevant fields change.

## Verification

The release verifier should run the property tests in the same library rail used by maintainers. Long-running fuzz campaigns may exist as optional local commands, but the required gate must be deterministic and bounded.
