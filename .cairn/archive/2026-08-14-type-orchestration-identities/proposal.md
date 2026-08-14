# Proposal: Type orchestration identities

## Why

Lattice already gives `RunId` a distinct Rust type. Durable orchestration still carries instance, dispatch, proposal, reservation, fact, revision, authority-policy, idempotency, event, and ledger-head identities as strings in several core models.

These values participate in state transitions, budget accounting, crash recovery, replay, and authority checks. A wrong string can satisfy the Rust signature and reach a later runtime mismatch.

## What Changes

- Inventory orchestration identity and digest strings across the pure core, persistence models, and adapters.
- Add private role-specific identity, revision, and digest types.
- Keep the existing `RunId` contract and integrate it with the new identity vocabulary.
- Split Nickel, CLI, webhook, scheduler, and redb wire DTOs from admitted orchestration core values where needed.
- Migrate pure transition, budget, dispatch, proposal, trigger, event, recovery, replay, and inspection APIs.
- Preserve Nickel, JSON, redb, and event-ledger compatibility.
- Add compile-fail, parser, persistence, crash-recovery, replay, and tamper tests.
- Adopt future Octet nominal-domain enforcement for the migrated orchestration core.

## Impact

- **Core**: `src/orchestration/core.rs` and focused pure identity helpers.
- **Persistence**: Conversion adapters around existing redb keys and values.
- **Shells**: CLI, scheduler, webhook, and trigger parsing.
- **Compatibility**: Existing operator commands, JSON reports, Nickel definitions, and stored records remain readable.

## Non-goals

- Do not change orchestration lifecycle semantics or retry policy.
- Do not change `RunId` allocation or redb ownership rules.
- Do not wrap workflow labels, summaries, diagnostics, or redacted display text.
- Do not claim remote exactly-once effects from typed local identities.
- Do not change agent provider, credential, or execution authority.
