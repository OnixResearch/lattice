## Context

Basalt's vendored UCAN boundary records one exact source revision, source manifest, support catalog, and policy identity. `ucan_enforcement` verifies raw compact credentials through the vendored UCAN shell. Its receipt records the UCAN revision but not the protocol profile.

Final UCAN `1.0.0` has different wire and authority semantics. Basalt must not interpret the same contract, resource, and ability fields as evidence that both formats are interchangeable.

## Decisions

### Decision: Bind source, support catalog, and protocol profile together

**Choice:** Extend the vendored UCAN identity cohort with an exact protocol-profile ID and its source support-catalog BLAKE3. The current cohort selects `onix-compact-ed25519-v1`.

A later final-profile cohort must name the reviewed UCAN revision that implements it. A revision/profile combination absent from the generated Basalt policy is unsupported.

**Rationale:** A source revision alone does not state which credential model is active.

### Decision: Keep profile selection typed and explicit

**Choice:** Raw UCAN enforcement requests and preverified semantics fixtures carry a closed `UcanProtocolProfile` value. Nickel policy declares allowed profile and source cohorts for each contract adapter.

Missing, unknown, unsupported, and mismatched profiles fail before token parsing, prospective authority, or replay. Basalt does not inspect bytes to guess a profile.

**Rationale:** Explicit nominal selection prevents downgrade and parser-confusion behavior.

### Decision: Publish a narrow adapter input contract

**Choice:** The current adapter remains a compact-profile adapter over the vendored compact API. Basalt adds a narrow application-owned `VerifiedUcanAuthorityInput` contract containing exact profile, source cohort, subject or holder reference, operation mapping inputs, request binding, public token identities, and bounded non-claims.

The current compact adapter projects into this contract. A later final-profile adapter must use the same application boundary without bringing UCAN wire or vendor codec types into the Basalt authority core.

**Rationale:** A stable application contract isolates protocol details while Basalt retains policy meaning.

### Decision: Bind receipts to the protocol cohort

**Choice:** Every UCAN enforcement receipt includes and hashes:

- Basalt receipt schema;
- UCAN protocol-profile ID;
- vendored UCAN revision;
- UCAN support-catalog BLAKE3;
- Basalt policy BLAKE3;
- token and request identities allowed by the redaction contract;
- operation profile;
- decision reason and non-claims.

Profile fields cannot be caller-controlled strings after admission. Persisted receipts remain non-authoritative.

**Rationale:** Evidence must distinguish the compact and final protocol paths.

### Decision: Upgrade is explicit and reversible

**Choice:** Basalt keeps the compact profile while a final-profile cohort is introduced. Contract policy selects one profile or a reviewed finite set. Removing compact support requires a separate migration change and evidence.

**Rationale:** Silent replacement would break signatures, stored proofs, consumers, and receipt interpretation.

### Decision: Cross-repository evidence is explicit

**Choice:** Before implementation, pin the archived UCAN profile-catalog receipt and reviewed vendored source revision. The separate final-profile adoption change must require archived UCAN wire and authorization evidence plus a reviewed Basalt snapshot refresh.

Basalt does not inspect an ambient sibling checkout at runtime or release time.

**Rationale:** Cross-repository compatibility needs durable published contracts, not workspace proximity.

## Flow

```text
explicit Basalt protocol cohort
  -> selected UCAN adapter
  -> UCAN-owned wire and authority verification
  -> Basalt-owned verified authority input
  -> pure Basalt policy and action mapping
  -> prospective authority decision
  -> UCAN/consumer replay boundary as configured
  -> live Basalt capability + profile-bound redacted receipt
```

## Risks and trade-offs

- Adding a profile field changes receipt and fixture schemas. A schema version and migration note are required.
- Supporting two profiles increases the test matrix. Cross-profile substitution tests are mandatory.
- A command-to-action map can overgrant when subject or resource facts are lossy. Mapping must fail closed on incomplete facts.
- Final-profile support depends on future UCAN archives. A separate Cairn owns that adoption and its blocked evidence.
