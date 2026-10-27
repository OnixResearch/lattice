# Standalone Effect Authority Specification

## Purpose

Defines the `standalone-effect-authority` capability.

## Requirements

### Requirement: Honest local authority provenance
r[lattice.standalone_effect_authority.provenance] A standalone local-operator grant MUST be distinct from externally verified UCAN evidence. Profile admission MUST NOT construct verified UCAN grant or verification-receipt facts. The separately reviewed Basalt policy MUST remain an upper bound on local grants.

#### Scenario: Local profile is selected
r[lattice.standalone_effect_authority.provenance.scenario.local]
- GIVEN a valid operator-selected profile
- WHEN a local effect is admitted
- THEN policy enforcement MAY admit the exact local grant without claiming UCAN verification.

#### Scenario: No profile is selected
r[lattice.standalone_effect_authority.provenance.scenario.missing]
- GIVEN no selected operator profile and no host-supplied authority
- WHEN an effect reaches standalone dispatch
- THEN dispatch MUST fail closed without invoking an adapter.

### Requirement: Current finite profile admission
r[lattice.standalone_effect_authority.currentness] The host MUST admit one bounded typed Nickel profile with finite expiry, pin its complete evaluated BLAKE3 identity for a run, and re-evaluate its semantic identity and expiry before every effect. Evaluation, time, expiry, removal, or semantic identity failures MUST deny without a product effect or permissive fallback.

#### Scenario: Imported profile input changes
r[lattice.standalone_effect_authority.currentness.scenario.changed]
- GIVEN a run admitted a profile whose meaning includes an imported input
- WHEN that input changes the evaluated profile before the next effect
- THEN the next effect MUST be denied.

#### Scenario: Expiry equals the observed time
r[lattice.standalone_effect_authority.currentness.scenario.expired]
- GIVEN a selected profile whose deadline is the current observed time
- WHEN admission or dispatch checks currentness
- THEN authority MUST be denied.

### Requirement: Exact local intent and grant binding
r[lattice.standalone_effect_authority.binding] A profile MUST bind one exact workflow revision, concrete machine identity, reviewed policy identity, and explicit node/operation/resource/ability grants. The provider MUST bind the selected run and authority, reject protected collaboration scopes, and preserve complete request/payload binding and reserve-before-effect behavior. Wildcards and graph-authored self-grants MUST NOT authorize effects.

#### Scenario: Another workflow requests the same ability
r[lattice.standalone_effect_authority.binding.scenario.mismatch]
- GIVEN a profile for one workflow revision and machine
- WHEN another workflow requests the same resource and ability
- THEN dispatch MUST deny before adapter invocation.

### Requirement: Operator-owned roots
r[lattice.standalone_effect_authority.roots] Only explicit profile root identifiers and absolute host paths MAY be opened as file capabilities. Existing capability-relative path, no-follow, and adapter policies MUST remain enforced. Unknown roots and escaping paths MUST deny.

#### Scenario: File node names an unconfigured root
r[lattice.standalone_effect_authority.roots.scenario.unknown]
- GIVEN a valid workflow grant and a file node naming an absent root
- WHEN file dispatch resolves its capability
- THEN no file operation MUST occur.

### Requirement: Consistent standalone host selection
r[lattice.standalone_effect_authority.hosts] Standalone entry points with application configuration MUST use its selected provider, including headless, TUI, scheduler, orchestration, and approval-resume paths. Explicit embedding providers MUST remain authoritative. A read-only inspection command MUST expose the saved workflow's canonical identities and effect requirements without granting or executing effects.

#### Scenario: Operator inspects a workflow
r[lattice.standalone_effect_authority.hosts.scenario.inspect]
- GIVEN a saved workflow
- WHEN authority inspection runs
- THEN it MUST report exact identities and grant requirements without dispatch or authority creation.

### Requirement: Repeatable scoped verification
r[lattice.standalone_effect_authority.validation] Maintained checks MUST cover positive and negative profile admission, changed or expired authority, exact binding, root restrictions, existing UCAN/scope/replay behavior, and actual authorized/denied standalone execution. Documentation MUST distinguish local operator grants from token verification, sandboxing, atomic revocation, global freshness, and release authority.

#### Scenario: Local authorized command completes
r[lattice.standalone_effect_authority.validation.scenario.native]
- GIVEN a bounded local workflow and an exact current operator profile
- WHEN native standalone execution completes
- THEN evidence MUST report the observed result and denied counterpart without claiming remote, global, sandbox, or release guarantees.
