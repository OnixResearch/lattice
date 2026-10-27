## Goals

- Express peer-authentication policy as a typed, reviewable contract.
- Evaluate the policy over supplied authenticated-message verdicts.
- Keep enforcement independent from transport, key custody, clock, and replay storage.
- Preserve BLAKE3 identities and reject digest-role and capability promotion.

## Policy boundary

Add a typed peer-auth policy contract. The Nickel contract declares the message facts a node must hold before it acts.

The Rust enforcement consumes an authenticated-message verdict and the caller policy. It returns a typed decision.

The boundary does not enter UCAN token issuance, parsing, proof, invocation, or authorization paths for end users.

## Policy facts

The policy declares these facts over one peer message:

1. The negotiated protocol version.
2. A bound message identity.
3. An explicit freshness disposition.
4. An explicit replay disposition.
5. The enforcing action, with its required authority.

The caller supplies the policy, the verdict, and any expected facts. Message content cannot widen the caller policy.

## Enforcement and output

Only a policy that admits all required facts may authorize the enforcing action.

The decision contains public, bounded fields:

- negotiated protocol version;
- message identity;
- freshness and replay dispositions;
- admitted action and authority;
- implementation and policy identities;
- required non-claims.

The decision excludes raw secrets, transport state, and end-user identity facts.

A peer-auth decision proves only the checked authentication-and-policy statement. It does not authenticate an end user or grant general authority.

## Functional core and shell

The pure enforcement core receives the verdict and policy as values. It reads no clock, key store, or replay store.

The shell owns input reads, output writes, and policy loading. Transport, key custody, clock, and durable replay storage stay consumer-owned.

## Reference

The Celld peer-protocol design is a bounded reference input. It is a comparison source, not a Basalt requirement, parity claim, or equivalence claim.

## Verification

Positive coverage includes a matching version, admitted freshness, exactly-once replay, and required authority.

Negative coverage includes wrong version, denied freshness, replay denial, missing authority, and secret-field leakage.

Boundary coverage rejects digest-role substitution, capability promotion, empty-message drift, ambient clock or key reads, and UCAN end-user path coupling.
