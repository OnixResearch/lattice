## Context

The current Nickel `Policy` record proves only broad field types. Basalt already has stronger knowledge in Rust: supported backends are a closed enum, schema version is exact, policy map keys are expected to equal embedded ids, and enforcement assumes meaningful resource/ability collections. Active Cairns further strengthen runtime identity and UCAN semantic boundaries.

The authoring contract should move obvious invalidity left without becoming another authorization engine.

## Decisions

### 1. Add a small Basalt policy prelude

**Choice:** Add shared contracts for exact schema values, normalized non-empty ids, closed backend ids, unique non-empty arrays, map-entry identity alignment, and deterministic validator issue classes.

**Rationale:** These primitives recur across policy and backend metadata and should not be redefined ad hoc.

### 2. Validate the whole source policy

**Choice:** A top-level validator requires:

- the exact supported policy schema;
- at least one contract;
- map keys equal embedded contract/backend ids;
- unique contract and backend identities;
- supported backend ids and their admitted metadata shape;
- non-empty unique resource-prefix and ability arrays; and
- no exact duplicate normalized entries.

The active iddqd package retains the equivalent runtime check for external JSON.

**Rationale:** Source validation and untrusted-artifact validation are complementary, not competing authorities.

### 3. Keep lexical shape separate from UCAN semantics

**Choice:** Nickel validates resource-prefix and ability values only to the normalized lexical preconditions shared with the Rust parser: non-empty, bounded, no control/whitespace ambiguity, and structurally valid path segments. UCAN wildcard containment, attenuation, caveats, proof traversal, replay, token verification, and authorization remain in UCAN/Rust and their active compatibility package.

**Rationale:** Nickel can improve authoring ergonomics without duplicating security semantics.

### 4. Backend metadata is closed at the source boundary

**Choice:** Backend ids and supported higher-order metadata combinations are validated against Basalt's Rust-supported backend vocabulary. Unknown future backends require a schema/version change plus Rust implementation and fixtures; they do not pass as arbitrary strings.

**Rationale:** Listing backends in the source artifact is a compatibility declaration.

### 5. Freshness includes semantic parity

**Choice:** The existing Nickel export freshness gate remains the source/projection byte check. A pure Rust parity core additionally validates the generated JSON, builds the runtime identity maps, and compares normalized policy meaning. A thin shell owns files and Nickel invocation; runtime enforcement never evaluates Nickel.

**Rationale:** Fresh bytes can still encode a contract/consumer mismatch.

### 6. Fixtures exercise both entry paths

**Choice:** Positive and negative `.ncl` fixtures prove authoring behavior. Corresponding malformed JSON fixtures prove `parse_policy_json` and runtime map conversion fail closed when consumers bypass Nickel.

**Rationale:** A packaged library cannot assume all policies came through its repository workflow.

## Risks / Trade-offs

- Stronger source contracts may expose policy examples that Rust previously tolerated; migrations should use deterministic issue classes.
- Lexical ability validation can accidentally grow into UCAN semantics. The contract intentionally stops before containment or authorization decisions.
- Identity checks overlap the active iddqd package by stage, not ownership: Nickel rejects reviewed source early; Rust rejects every external artifact authoritatively.
- Contract conformance is not policy correctness or authorization evidence.
