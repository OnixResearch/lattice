## Why

Basalt decides bounded contract and UCAN authority facts, but it does not define one portable policy shape for runtime effects such as filesystem access, network connections, process operations, environment access, and named host bindings.

AgentOS provides a useful reference for separate permission scopes, secure defaults, operation-specific rules, and deny-before-effect behavior. Basalt must adapt that clarity without copying its TypeScript schema, trusting client configuration as ambient authority, or claiming that a policy decision enforces a runtime boundary.

## What Changes

- Add a Nickel-authored runtime effect permission contract with closed scope and operation vocabularies.
- Add normalized effect request, subject, rule, profile, decision, and receipt DTOs.
- Use deny-by-default profiles and deterministic rule precedence with ambiguity rejection.
- Keep path normalization, endpoint parsing, executable identity, environment lookup, binding implementation, and effect execution consumer-owned.
- Compose effect permission with existing Basalt contract and verified UCAN authority inputs when configured.
- Add consumer conformance fixtures for Animus and Molten without adding runtime dependencies.

## Impact

- **Files**: Nickel contracts and generated policy, pure Rust policy core, receipts, fixtures, docs, downstream compatibility checks, and `contract-backends` requirements.
- **Testing**: Positive exact and prefix rule cases plus negative unknown scope, unknown operation, ambiguous rule, malformed subject, stale profile, missing capability, and enforcement-overclaim cases.
- **Boundary**: Basalt returns an allow or deny fact over supplied normalized inputs. It does not perform the effect, supply a sandbox, normalize hostile paths, open sockets, spawn processes, or prove current runtime enforcement.
