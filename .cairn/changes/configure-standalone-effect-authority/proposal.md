## Why

The standalone CLI and TUI select unavailable effect authority even when an operator intentionally runs a local workflow. Shell, HTTP, and file nodes consequently cannot dispatch. Embedders already have a run-context port, but ordinary operators have no explicit, bounded source for that port.

## What Changes

- Add a typed Nickel local-operator profile, selected explicitly through application configuration, for one exact workflow revision and concrete machine identity.
- Admit only listed node, operation, resource, and ability tuples under the separately reviewed effect policy, with a finite expiry and independently operator-supplied file roots.
- Re-evaluate the profile before each effect and deny if its admitted semantic identity changed, expired, disappeared, or could not be evaluated. Keep unconfigured execution fail-closed.
- Distinguish first-party local-operator evidence from externally verified UCAN evidence. Local admission must never construct or claim UCAN verification receipts.
- Route configured standalone execution entry points through the selected provider and expose read-only workflow authority inspection so operators can author exact grants.
- Keep the published role-protocol runtime profile bound to its original immutable implementation source. Its Nix source-manifest check must not reinterpret the evolving checkout as that historical revision.

## Impact

- **Files**: application configuration/contracts, standalone host entry points, effect authority evaluation and provider modules, authority integration tests, README, the effect-authority guide, and the Nix input/check for the existing published role-protocol cohort.
- **Testing**: positive and negative profile admission/currentness cases, durable attempt and root boundaries, existing UCAN/scope fixtures, actual authorized/denied CLI execution, and maintained repository checks.
- **Consumer and owner**: standalone Lattice CLI/TUI operators are the immediate consumers; Lattice maintainers own the profile, admission, host wiring, and repeatable local checks. Basalt retains reviewed policy enforcement and UCAN verification semantics.
- **Non-claims**: a local operator profile is not a signed UCAN, a sandbox, remote caller authentication, global revocation freshness, atomic revocation, exactly-once execution, or release authority. No remote mutation or dependency publication is authorized.
