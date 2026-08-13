## Why

Basalt policy DTOs currently expose maps keyed by strings while the mapped values also carry their own stable ids. That duplicates identity in two places and leaves the core responsible for remembering to compare the map key with `ContractPolicy.id` or `ContractBackendPolicy.id` everywhere policy data enters the runtime.

`iddqd` is a good fit for this slice because its ID maps borrow keys from values and force insertion behavior to be explicit. Basalt can use that shape internally while preserving the existing Nickel-exported JSON object shape for policy artifacts and downstream compatibility.

## What Changes

- Add an iddqd-backed policy identity-map core for contracts and backend metadata.
- Preserve the public policy JSON shape and generated Nickel artifact contract.
- Reject duplicate, missing, or mismatched contract/backend ids before enforcement.
- Add positive and negative fixtures for valid policy maps, duplicate ids, key/value mismatches, unknown backend ids, and weakened non-claims.

## Impact

- Policy parsing becomes more explicit about identity drift and duplicate handling.
- Enforcement keeps the same public API and receipt shape unless invalid policy input is rejected earlier.
- Basalt gains a small, focused iddqd pilot without changing UCAN semantics or Steel/Nickel backend authority boundaries.
