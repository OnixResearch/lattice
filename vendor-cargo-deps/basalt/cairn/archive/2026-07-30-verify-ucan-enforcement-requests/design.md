## Context

`authority_core` is pure and accepts references to already verified facts.
The previous UCAN pin did not provide safe bearer ownership or holder proof.

UCAN commit `34449ba1f1a9ef991225e4e6683b5d6aca8c07cf` completes and archives the required source changes.
This revision is not pushed, so Basalt cannot claim that a remote source can fetch it.
The Basalt README permits an equivalently reviewed vendored boundary.

## Success Contract

Completion requires all of these facts:

- Basalt builds from a checked-in UCAN snapshot at the reviewed revision.
- The repository owns a deterministic import and offline check mechanism.
- The snapshot preserves the UCAN and verified-logic license files.
- A closed Basalt action maps to one contract, ability, operation profile, and exact request digest.
- The adapter rejects caller capability facts and validity booleans.
- Borrowed UCAN verification and non-consuming authorization run before Basalt policy evaluation.
- Basalt fully builds the prospective authority decision and redacted receipt before holder replay.
- Basalt policy denial does not call the replay boundary.
- Fresh holder verification repeats all authoritative checks and calls replay once as its final commit.
- After successful replay, Basalt only moves prebuilt values into the live borrowed capability.
- A failed final verification returns no prospective authority decision.
- Positive, negative, release, and lifecycle rails pass.

False completion includes a workspace-relative dependency, a remote-fetch claim for the unpushed revision, caller-supplied verified facts, or fallible work after replay success.

## Portfolio Search Registry

| Family | Mechanism | State | Reason |
| --- | --- | --- | --- |
| Reviewed vendored source | Import the exact committed Cargo build closure and BLAKE3-check every file. | selected | This mechanism is durable and does not depend on remote publication. |
| Remote Git pin | Pin commit `34449ba` from the authoritative repository. | blocked | The revision is not pushed and is not remotely fetchable. |
| Workspace path | Depend on the sibling UCAN worktree. | rejected | This mechanism is not durable and cannot ship. |
| Reimplement UCAN checks | Copy holder and token semantics into Basalt. | rejected | This mechanism creates a second security implementation and weakens the reviewed boundary. |

The audit budget uses the local committed source, archived UCAN changes, deterministic source parity, focused tests, release rails, and Cairn gates.

## Decisions

### Decision: Use a checked-in reviewed UCAN snapshot

**Choice:** Import the Cargo build closure from UCAN commit `34449ba1f1a9ef991225e4e6683b5d6aca8c07cf` into `vendor/ucan/`.

`tools/sync-vendored-ucan.rs` has three modes:

- `--import SOURCE_REPOSITORY` reads only the pinned commit and replaces the snapshot.
- `--check` checks the checked-in revision, file modes, byte lengths, BLAKE3 identities, and license files.
- `--check-source SOURCE_REPOSITORY` compares the snapshot with the same pinned Git commit.

**Rationale:** The snapshot is a durable repository-owned source boundary.
The mechanism does not depend on a sibling path during normal builds.
It does not claim that the unpushed source revision is available remotely.

### Decision: Use a closed typed action mapping

**Choice:** Define a Basalt action enum for supported downstream operations.
Each variant maps to one contract ID, ability, and BLAKE3 operation-profile identity.

The adapter derives the exact request-binding digest from a versioned Basalt domain, action, contract, resource, and ability.
It rejects a request whose contract or ability differs from the selected variant.

**Rationale:** A typed mapping prevents caller string equality from defining action semantics.
UCAN signs the resulting fixed operation profile and exact request digest.

### Decision: Reject caller authority facts

**Choice:** The raw adapter rejects non-empty `EnforcementRequest::capabilities`.
Its API accepts no verification booleans, verified grant references, or authority decisions.

After UCAN authorizes the exact operation, the adapter creates one internal exact grant.
It then builds `AuthorityEvidence`, calls `adapt_enforcement_request`, and calls `evaluate_authority`.

**Rationale:** Only adapter-produced facts can reach the authority core.
Caller fields cannot create authority.

### Decision: Keep policy denial before replay

**Choice:** The adapter uses this order:

1. Check token and request bounds.
2. Map the typed action and derive the operation binding.
3. Run borrowed low-level UCAN verification with explicit inputs.
4. Run non-consuming UCAN capability and caveat authorization.
5. Run the complete Basalt authority core and policy decision.
6. Build the prospective live decision and redacted receipt fully in memory.
7. Run fresh holder-signed verification with the same explicit inputs.
8. Let UCAN call replay once as its final state-consuming commit.
9. On success, only move the prebuilt values into the live capability.

The resolver, proof store, revocation checker, profile check, and caveat policies can run twice.
These repeated calls must remain non-consuming.
The final holder verification is authoritative.

**Rationale:** Basalt policy denial cannot burn a nonce.
No Basalt allocation, hash, policy call, formatting, assertion, or fallible function runs after successful replay.

### Decision: Keep final failure isolated

**Choice:** A prospective authority decision stays private until final holder verification succeeds.
If final verification fails, the adapter drops the prospective value and returns only a typed reason with a redacted receipt.

**Rationale:** A changed resolver, proof, revocation, caveat, holder, or replay result cannot leak stale authority.

### Decision: Keep authority live and borrowed

**Choice:** A successful Basalt value borrows the exact `EnforcementRequest`.
It privately owns the UCAN live authorization, Basalt authority decision, and redacted receipt.

The type has no public constructor and no `Clone`, `Serialize`, or `Deserialize` implementation.

**Rationale:** Persisted evidence cannot recreate the live in-process authority capability.

### Decision: Keep receipts redacted and deterministic

**Choice:** Receipts contain stable reason classes and BLAKE3 identities for redacted canonical material.
The receipt hash has a separate domain and frames each variable field with a big-endian `u64` length.
This framing includes the schema, reason, UCAN revision, and each non-claim.
They omit compact credentials, proof bytes, keys, caveat payloads, exact identity text, and exact scope text.

Oversized credentials fail before token hashing.
Their receipts bind the reason and explicit input length, not a credential identity.

**Rationale:** A receipt is bounded evidence only.
It cannot recreate a token, proof, request, or live capability.

### Decision: Preserve functional core and thin shell

**Choice:** Pure helpers own action mapping, canonical request digests, reason mapping, and receipt hash material.
The shell calls injected UCAN services and the existing Basalt authority core.

**Rationale:** The security order remains explicit and the pure logic needs no mocks.

## Adversarial Audit

The test matrix changes one boundary at a time.
It covers malformed and oversized tokens, signatures, proofs, issuers, holders, time, caveats, scope, revocation, replay, limits, policy, and caller authority fields.

Call-count tests prove these order properties:

- Policy denial makes zero replay calls.
- A preflight caveat denial makes zero replay calls.
- Final verification failure exposes no authority decision.
- A valid request makes one replay call.
- A repeated nonce fails at the final replay boundary.
- No observer runs after successful replay.

Source guards reject public live constructors, serde, clone, and post-replay work.

## Non-Claims

- The vendored snapshot does not make the unpushed revision remotely fetchable.
- Holder proof does not prove transport identity or key custody policy.
- Resolver and revocation results do not prove global freshness.
- Replay admission does not prove durability or cross-process coordination.
- The action mapping does not prove application policy truth.
- The request digest does not prove that upstream request construction is correct.
- Authority admission does not prove downstream effect correctness or safe execution.
- Tests and receipts do not prove whole-system security.

## Risks and Trade-offs

- The vendored source adds repository size and requires an explicit reviewed refresh.
- Non-consuming UCAN services run twice before one final replay commit.
- A service that mutates during an observation violates its documented contract.
- Replay durability and crash behavior remain caller-owned.
