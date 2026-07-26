## Why

Lattice's canonical Octet wrapper claims to baseline reviewed debt while every configured Octet lint is `deny`. Deny-level Dylint findings make Cargo exit `101`, so Octet classifies the run as an integration failure before baseline admission can succeed or a replacement baseline can be generated. The Radicle dependency-source change also changed Octet's Cargo.toml-bound config identity, turning 892 reviewed unique findings into 892 new and 892 stale identities; the clean accepted source also contains one pre-existing file-length observation improvement from 676 to 657 lines.

The maintained traceability inventory is independently stale: current repository-policy Cairn coverage reports 12 missing Radicle-cutover identifiers with BLAKE3 `c23fc013acdb1e4ae5d093b1f01ddb8c82e7d744ce8f9989f3841ea56e18c457`, while the typed policy still declares an empty missing set. Both gates therefore fail before reporting the bounded debt they were designed to track.

## What Changes

- Make the canonical Octet debt probe warning-level with no disabled lints, while retaining fail-closed baseline admission in `octet-precommit-check`.
- Generate a current Octet baseline only after reviewing all 1,141 ordered records: 1,140 retain exact lint/group/file/line/column/message/crate facts, while one pre-existing file-length record changes only from 676 to 657 observed lines, in addition to config-derived identities and the intentional severity transition.
- Extend the Nix Octet check with a negative source mutation that introduces a new stable finding and must be rejected by the wrapper.
- Refresh the typed traceability inventory policy and generated report from the exact failing 12-item Cairn receipt without promoting any inventory row into evidence.
- Refresh the separately maintained repository-validation completion fixture for the already accepted Radicle cutover's Cargo, README, and historical test bytes, preserving every classification, rationale, evidence path, and coverage result.
- Document that direct warning-level Octet output is review input and that the canonical wrapper, not compiler severity alone, owns new-finding admission.

## Success Criteria

- `octet-precommit-check` reports 892 accepted/current identities, zero new identities, zero stale identities, and exits successfully.
- A deterministic negative Nix fixture introduces a new `todo!()` finding and the same wrapper rejects it.
- No Octet lint is disabled, malformed, or configured as `allow`.
- Repeated traceability inventory generation is byte-identical for the current 12-item missing set and stale or mutated inputs fail closed.
- The repository-validation completion audit binds the accepted current Cargo, README, and historical test bytes and its mutation test still fails closed without promoting coverage.
- Focused checks, repository checks, Cairn gates, and the bounded full flake gate pass except that Cairn coverage remains intentionally unsuccessful for the explicitly inventoried missing requirements.

## Non-Goals

- Claiming accepted Octet findings are correct, harmless, complete, or release-ready.
- Hiding compiler errors unrelated to Octet, disabling lints, adding suppressions, or accepting an unreviewed finding identity.
- Satisfying, promoting, or deleting the 12 missing Radicle-cutover traceability requirements.
- Changing runtime behavior, dependency source identity, or release authority.

## Impact

- **Validation policy**: existing Octet debt moves from contradictory compiler-deny plus baseline semantics to warning-level observation plus fail-closed stable-ID admission.
- **Evidence**: the baseline, traceability policy, generated inventory, and bounded lifecycle evidence receive new BLAKE3 identities.
- **Runtime**: no Rust runtime implementation changes.
