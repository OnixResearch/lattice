## Design

The authority core is a pure Basalt library surface over already-verified UCAN facts and already-loaded Basalt policy values. It does not parse compact tokens, verify signatures, resolve DIDs, read clocks, consult revocation stores, evaluate Nickel, or perform network/storage work.

### Decisions

1. **Inputs are verified facts, not raw tokens.** The core receives verified grant refs, UCAN verification receipt refs, request facts, and policy refs. Raw token verification remains UCAN-owned.

2. **Contract admission is exact.** The request contract ID, resource, ability, and request ref must match the Basalt policy row and the verified UCAN grant facts.

3. **Denials are structured.** Missing grant, wrong resource, wrong ability, stale verification, caveat denial, replay denial, policy denial, and malformed request each have stable diagnostics.

4. **Nickel remains review boundary.** Humans author contract policy in Nickel and export deterministic runtime policy JSON. The authority core consumes checked Rust values and does not run Nickel.

5. **No ambient authority.** Passing authority-core validation admits only the bounded request described by the input DTO. It does not grant transport, storage, persistence, replay, key-resolution, or downstream business authority.

### Validation shape

Positive tests cover a matching contract/resource/ability request with verified UCAN grant refs and Basalt policy refs. Negative tests cover missing verified grants, mismatched resource, mismatched ability, missing policy ref, stale verification ref, caveat denial, replay denial, malformed request ref, and text that treats a content ref or token possession as current authority.
