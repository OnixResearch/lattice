# UCAN protocol-profile binding evidence

## Baseline

Before source and profile changes, 9 focused UCAN enforcement tests passed.
The repository-owned vendored-source checker accepted revision `277ab768eb532ac0c715a3bbff2a7dc37a702f11` with 677 files.
No existing worktree owned this active Cairn change.

## Upstream cohort

The repository-owned importer selected exact UCAN object `c483c7b58c42ec6636e9b8b8c0f73a2b609bf21e`.
The imported snapshot contains 693 files and passes its deterministic BLAKE3 manifest and license checks.

The reviewed support catalog has BLAKE3 identity `fe77fd63076eaef555b74e702e4c14e483bdba70151ccc955c75b17d97412427`.
The generated Basalt policy retains BLAKE3 identity `5475d5473ec98cbf052a83512836b0739f5dad3e7b9b0ed201eb3094b1cf39bf`.

The imported UCAN snapshot consumes Durable Authority State cores that require Rust 1.96.
Basalt now pins Rust 1.96 through a reviewed immutable `rust-overlay` input.
Nix generated the updated `flake.lock`.

## Implementation

`BasaltUcanProtocolProfile` is a closed application-owned profile type.
The current cohort admits only `onix-compact-ed25519-v1`.
The typed Nickel cohort rejects unknown profiles, stale source, final-adapter promotion, and weakened non-claims.

`UcanEnforcementRequest` requires an explicit profile.
The compact adapter rejects final UCAN `1.0.0` before token parsing and before replay.
It does not auto-select a decoder from credential bytes.

`VerifiedUcanAuthorityInput` is private-construction application data.
It binds profile, source, catalog, verified subject, operation, request, token identity, grant class, and non-claims.
No UCAN wire or codec type enters the Basalt authority core.

Enforcement receipt schema `basalt.ucan-enforcement-receipt.v2` binds protocol profile, source revision, support catalog, policy, operation, token, request, reason, and non-claims.
UCAN semantics fixture schema `basalt.ucan-semantics-fixture.v2` requires the same profile cohort.
Old, missing, unknown, and substituted profiles fail closed.

## Validation

- Focused profile unit test passed.
- UCAN enforcement integration tests passed: 22 of 22.
- UCAN semantics tests passed: 8 of 8.
- Nostr identity tests passed after the exact source refresh: 3 of 3.
- Formatting and Clippy with denied warnings passed.
- The typed Nickel cohort and all negative fixtures passed.
- Vendored source, policy, semantics, license, issuer, and ast-grep freshness checks passed.
- The direct release verifier passed 184 default-feature tests and 197 CLI-feature tests.
- Property, Steel, authority, semantics, token-flow, policy, identity, and documentation checks passed.
- Cairn proposal, design, task, and validation gates passed.
- Cairn traceability reported 9 of 217 accepted identifiers referenced.
- All 8 new profile identifiers are referenced; 208 inherited identifiers remain missing.
- `nix flake check path:$PWD -L` passed with local builders.

## Bounded portfolio search

The exact goal was explicit compact-profile binding without profile inference or authority transfer.
False completion included a string-only receipt field, byte-based decoder guessing, caller-constructible verified facts, an unrefreshed source cohort, final-profile promotion, or replay before profile admission.

Three correlated serial lenses were used because subagents were not authorized:

| Family | Mechanism | Evidence | State |
|---|---|---|---|
| nominal-profile | closed Rust and Nickel profile cohort | matching compact cases pass; substitutions fail | validated |
| byte-inference | inspect credential bytes to select a decoder | rejected because bytes cannot grant nominal profile authority | falsified |
| receipt-only | add profile only after enforcement | rejected because denial must precede parsing and replay | falsified |

The surviving nominal-profile design passed adversarial missing, unknown, stale, final-profile, policy, source, receipt, and replay-order checks.
The declared budget was three lenses, one imported source cohort, one compact adapter, and one full release gate.

## Non-claims

Profile matching does not prove token correctness, key trust, revocation freshness, replay durability, policy completeness, transport security, effect correctness, final-profile compatibility, or release eligibility.
UCAN retains protocol wire and authority semantics.
Basalt retains application policy and action mapping.
Callers retain external credentials, state, clocks, transport, and effects.
