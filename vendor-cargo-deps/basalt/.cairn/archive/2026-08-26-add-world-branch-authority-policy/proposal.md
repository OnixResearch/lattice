## Why

Basalt decides bounded contract and UCAN authority facts. It does not define portable policy for branching a runtime world that references external capabilities.

A Boolean copy decision cannot express attenuation, exclusive transfer, simulation-only use, promotion-gated release, replacement obligations, or non-branchable authority. Consumers need one reviewed policy vocabulary without transferring token, storage, or effect authority into Basalt.

## What Changes

- Add a Nickel-authored world-branch authority contract with closed branch actions and capability modes.
- Define normalized capability, source scope, destination scope, branch action, currentness, policy, decision, obligation, and receipt DTOs.
- Support copyable, attenuated, linear, simulation-only, promotion-gated, replace-before-activation, and non-branchable policy results.
- Use deny-by-default policy and reject unknown, ambiguous, widening, or incomplete rules.
- Compose decisions with supplied verified UCAN and durable-currentness facts when a profile requires them.
- Return obligations only. Basalt does not mint tokens, transfer ownership, bind adapters, activate branches, promote heads, or dispatch effects.
- Add metadata-only BLAKE3 receipts and frozen Molten consumer mappings.

## Dependencies

- Existing Basalt Nickel contract, generated policy, UCAN, runtime-effect permission, and receipt infrastructure.
- Durable Authority State concepts for replay, revocation, generation, and uncertain mutation observations.
- A frozen Molten mapping fixture without a runtime dependency.

## Non-Goals

- Capability storage, secret handling, token issuance, revocation collection, clock access, branch persistence, or runtime enforcement.
- Treating a policy result as proof that linear transfer, attenuation, simulation confinement, promotion, or effect execution occurred.
- A universal capability model for kernel or hardware enforcement.

## Impact

- **Policy**: typed Nickel contracts and generated JSON for world-branch authority profiles.
- **Core**: pure validation, matching, ambiguity rejection, decision, obligation, and safe receipt projection.
- **Consumers**: frozen positive and negative Molten mapping fixtures.
- **Testing**: every supported mode plus negative widening, duplicate linear use, stale facts, unknown mode, ambiguity, secret disclosure, and enforcement-overclaim cases.
