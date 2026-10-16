## Context

`RunContextProvider` already supplies exact run identities and the durable dispatch shell. The standalone hosts select `UnavailableRunContextSource`. The dispatch shell reserves a complete intent before loading current authority and scope facts; failed reservation cannot execute an adapter. Canonical workflow and machine identities are already produced by orchestration and the Choregraph lowering path.

Basalt's `authority_core` consumes externally verified UCAN grants. Its `Direct`, `Current`, and verification-receipt types must not be populated from local configuration alone. Its generic `enforce` policy boundary can evaluate explicitly operator-admitted capability grants without asserting UCAN provenance.

## Decisions

### Decision: Explicit local-operator provenance

**Choice:** Add a distinct local-operator evidence variant beside the existing externally verified UCAN evidence. Reuse exact intent, policy, scope, attempt, adapter, and outcome validation. Evaluate local capabilities through the reviewed Basalt policy rather than fabricating token-verification facts. Migrate existing verified-UCAN callers to the explicit variant without changing their verification contract.

**Rationale:** A trusted local operator can grant local process authority without pretending to have verified a token. This keeps vendor snapshots unchanged and preserves the stronger UCAN boundary for hosts that supply it.

### Decision: Immutable per-run profile admission and per-effect rechecks

**Choice:** An optional absolute application-configuration path selects a typed Nickel profile. Its canonical evaluated value binds schema, exact workflow revision, concrete machine, reviewed policy identity, finite expiry, bounded explicit node/operation/resource/ability grants, and root identifiers mapped to absolute paths. Pin the evaluated BLAKE3 identity at run-context admission. Re-evaluate imported Nickel inputs and compare the complete semantic identity before every dispatch. Check expiry at admission and each dispatch using a shell-observed time. Missing selection keeps the existing unavailable provider; malformed selected input must not fall back.

**Rationale:** Workflow content cannot choose its own authority. Checking the evaluated value includes the meaning of imports, while harmless source formatting is not a revocation. Removal or semantic edits revoke existing runs at their next authorization check. This is not an atomic check-to-effect revocation protocol.

### Decision: Preserve exact workflow and resource boundaries

**Choice:** Validate machine, workflow, run, authority, unscoped status, node, transition, operation, resource, and ability before constructing local facts. The existing request/payload binding and durable intent reservation remain authoritative. Root directories are opened only by the host from the operator profile; workflow roots never open ambient paths. Protected collaboration requests cannot use the unscoped local provider.

**Rationale:** A profile is a bounded local grant, not permission for a changed graph, another run, a protected scope, a wildcard capability, or an arbitrary root. Existing process/network/file adapters retain their independent limits.

### Decision: One configured provider for standalone hosts

**Choice:** Expose a configuration helper returning the selected run-context provider and use it in CLI, TUI, scheduler, approval-resume, and orchestration host paths that have application configuration. Preserve embedding APIs that explicitly receive providers and fail-closed helpers without configuration. Add read-only inspection of a stored workflow's canonical identities and effect requirements; inspection never creates authority or executes work.

**Rationale:** Operators need a reproducible authoring path, and a configured provider must not disappear at another standalone execution entry point.

### Decision: Preserve published protocol provenance

**Choice:** Verify the published role-protocol source manifest against a separately pinned, non-flake source input at its declared runtime revision `cbb5db0e4844b466dcd1877218eb1f00a2a7caf2`. Keep its contract, profile, generated identities, and historical evidence unchanged. Continue checking the current protocol CLI boundary and current authority behavior independently.

**Rationale:** Shared authority types now change without republishing that protocol profile. Checking its frozen hashes against the mutable checkout conflates two source cohorts. Re-pinning the published profile to an unpublished or fabricated revision would violate its exact-source contract; a read-only immutable source input preserves it.

## Risks / Trade-offs

- Profile/configuration writers are trusted local operators. A process grant permits the selected workflow to run with the existing host environment; this is not an untrusted-workflow sandbox.
- Per-effect re-evaluation adds bounded local Nickel work and has the ordinary check-to-effect race. No atomic or global revocation claim is made.
- Finite expiry relies on the supplied host clock. Equality with the deadline denies; missing time denies.
- Roots are capabilities opened at run admission. Profile edits deny subsequent dispatch, but do not prove external filesystem identity or mount stability.
- Local authority has no UCAN signature, proof, holder, or identity-verification claim. Existing UCAN evidence remains a separate externally verified input.
