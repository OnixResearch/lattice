## Why

The maintained traceability debt inventory identifies six safe run-inspection and run-store-recovery scenarios with exact maintained evidence, but they intentionally remain missing until their assertions are reviewed and linked precisely. Adding broad file-level markers would overstate evidence, while leaving reviewed exact candidates indefinitely unlinked keeps known evidence invisible.

## What Changes

- Strengthen the maintained inspection and recovery tests where an exact scenario clause is not already asserted directly.
- Add the six reviewed Tracey markers only beside the tests and assertions that establish their complete scenario contracts.
- Add a deterministic positive/negative promotion check proving that exactly the six allowlisted identifiers leave the missing set, no identifiers are added, and dangling remains empty.
- Regenerate the maintained debt inventory for the resulting 361-item missing set and remove promoted identifiers from the review registry.
- Keep Tracey fail-closed and preserve every unrelated missing identifier.

## Impact

- **Files**: safe inspection/recovery tests, one promotion-invariant test and fixtures, typed inventory policy/generated report, repository-validation spec, and lifecycle evidence.
- **Testing**: focused positive/negative tests, exact baseline/current missing-set comparison, inventory checks, repository checks, Cairn gates, Octet, and Nix checks.
