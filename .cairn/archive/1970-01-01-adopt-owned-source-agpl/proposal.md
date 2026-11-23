## Why

The operator selected AGPL-3.0-or-later for all new owned source. Lattice's MIT workspace default, budget ledger, workflow contract, transport grants, and owned operation-pack fixture currently contradict that decision.

## What Changes

- Apply AGPL-3.0-or-later to all new first-party code, contracts, schemas, fixtures, templates, documentation, and policy; align package grants and owned pack metadata.
- Preserve historical permissive grants, imported Rats/Animus/Basalt/Host Control/TOON bytes and immutable dependency assertions; do not update Kiln's imported contract or any source pins/locks.

## Impact

- **Files**: root/workspace and contract/transport Cargo metadata, authoritative and historical license texts, owned fixture authoring and consumer expectations, current README/package docs.
- **Testing**: coordinated final Cargo metadata/package inventory, owned operation-pack admission checks, and explicit-policy Cairn validation. No builds, tests, lint, or formatters during preparation.
